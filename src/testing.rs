use std::{fmt::format, fs::File, io::Write, path::PathBuf, sync::Arc};

use chrono::Utc;
use either::Either::{self, Left, Right};
use iced::{
    keyboard::key::Named,
    widget::{button, column, container, row, scrollable, text, text_input},
    Alignment::Center,
    Element,
    Length::{self, Fill},
    Task,
};
use rand::Rng;
use rfd::FileDialog;

use crate::{collection::Collection, Action};

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    InputSubmit,
    SaveResults,
    Start(Arc<Collection>, usize),
    KeyPressed(Named),
    SaveFile(Option<PathBuf>),
}

#[derive(Debug, Clone)]
enum Answer {
    Correct,
    Incorrect,
    None,
}

#[derive(Debug, Clone)]
pub struct TestWidget {
    selected_collection: Option<Arc<Collection>>,
    selected_collection_words: Option<Vec<String>>,
    answers: Vec<(String, String, bool, i64)>,
    answer: Answer,
    word_index: usize,
    inputed: String,
    words_number: usize,
    start_message: bool,
    end_message: bool,
    start_time: i64,
}
impl TestWidget {
    pub fn new() -> Self {
        Self {
            selected_collection: None,
            word_index: 0,
            answer: Answer::None,
            answers: Vec::new(),
            inputed: String::new(),
            selected_collection_words: None,
            words_number: 0,
            start_message: true,
            end_message: false,

            start_time: 0,
        }
    }
    pub fn update(&mut self, message: Message) -> Either<Task<Message>, Action> {
        match message {
            Message::InputChanged(input) => self.inputed = input,
            Message::InputSubmit => {
                let sub_time = Utc::now().timestamp();
                let word = self
                    .selected_collection_words
                    .as_ref()
                    .unwrap()
                    .get(self.word_index)
                    // .map(|v| v.as_str())
                    .unwrap();
                let is_correct = self
                    .selected_collection
                    .clone()
                    .unwrap()
                    .words()
                    .get(word)
                    .unwrap()
                    .contains(&self.inputed);
                self.answers
                    .push((word.clone(), self.inputed.clone(), is_correct, sub_time));

                if self.word_index < self.selected_collection_words.as_ref().unwrap().len() - 1 {
                    self.word_index += 1
                } else {
                    self.end_message = true
                }
                self.inputed.clear();
            }
            Message::Start(c, n) => {
                self.selected_collection = Some(c.clone());
                self.words_number = n;
                self.answer = Answer::None;
                self.inputed.clear();
                self.answers.clear();
                self.word_index = 0;
                if c.words().len() < n {
                    println!("there is not enough words in collection");
                    self.words_number = c.words().len();
                }
                let mut words = Vec::new();
                let keys = c.words().keys().collect::<Vec<_>>();
                let mut used = Vec::new();
                while words.len() < self.words_number {
                    let rnd = rand::thread_rng().gen_range(0..c.words().len());
                    if used.contains(&rnd) {
                        continue;
                    }
                    used.push(rnd);
                    words.push(keys[rnd].clone())
                }
                self.selected_collection_words = Some(words);
                return Left(text_input::focus("testing_input_id"));
            }
            Message::KeyPressed(k) => match k {
                Named::Escape => return Right(Action::ChangeScreen("setup_screen".into())),
                Named::Enter => {
                    if self.end_message {
                        self.end_message = false;
                        return Right(Action::ChangeScreen("setup_screen".into()));
                    }
                    if self.start_message {
                        self.start_message = false;
                        self.start_time = Utc::now().timestamp();
                    }
                }
                _ => {}
            },
            Message::SaveResults => {
                return Left(Task::perform(
                    async {
                        let filename = format!("test_save-{}.txt", Utc::now().timestamp());
                        FileDialog::new()
                            .add_filter("Text file", &["txt"])
                            .set_file_name(filename)
                            .save_file()
                    },
                    Message::SaveFile,
                ))
            }
            Message::SaveFile(p) => {
                if let Some(p) = p {
                    let mut file = File::create(p).unwrap();
                    writeln!(file, "Start time");
                    writeln!(file, "{}", self.start_time);
                    writeln!(file, "\n");
                    let correct = self.answers.iter().filter(|v| v.2).count();
                    writeln!(file, "Correct\tAll");
                    writeln!(file, "{correct}\t{}", self.answers.len());
                    writeln!(file, "\n");
                    for (word, answer, correct, ts) in self.answers.iter() {
                        writeln!(
                            file,
                            "{}\t{}\t{}\t{}",
                            word,
                            if answer.is_empty() { "-" } else { answer },
                            if *correct { 1 } else { 0 },
                            ts
                        )
                        .unwrap();
                    }
                }
            }
        }
        Left(Task::none())
    }

    pub fn view(&self) -> Element<Message> {
        if self.selected_collection.is_none() {
            return container(text("Smthing not right")).center(Fill).into();
        }
        let word = self
            .selected_collection_words
            .as_ref()
            .unwrap()
            .get(self.word_index)
            .map(|v| v.as_str())
            .unwrap();
        if self.end_message {
            return container(
                column![
                    text("Test finished").size(64),
                    text("Press ENTER to exit").size(12),
                    scrollable(column(self.answers.iter().map(|v| {
                        text!(
                            "| {:^30} | {:^30} | {:^12} |",
                            v.0,
                            if v.1.is_empty() { "-" } else { &v.1 },
                            if v.2 { "Correct" } else { "Incorrect" }
                        )
                        .size(16)
                        .into()
                    })))
                    .height(100),
                    button("Save results").on_press(Message::SaveResults)
                ]
                .spacing(10)
                .align_x(Center),
            )
            .center(Fill)
            .into();
        }
        if self.start_message {
            container(
                column![text("Test").size(64), text("Press ENTER to start").size(12)]
                    .align_x(Center),
            )
            .center(Fill)
            .into()
        } else {
            container(
                column![
                    text(word).size(48),
                    text_input("Type...", &self.inputed)
                        .id("testing_input_id")
                        .on_input(Message::InputChanged)
                        .on_submit(Message::InputSubmit),
                ]
                .spacing(30)
                .align_x(Center)
                .width(400),
            )
            .center(Fill)
            .into()
        }
    }
}

impl Default for TestWidget {
    fn default() -> Self {
        Self::new()
    }
}
