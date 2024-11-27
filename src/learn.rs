use std::sync::Arc;

use either::Either::{self, Left, Right};
use iced::{
    keyboard::key::Named,
    widget::{
        button::{self},
        column, container, text, text_input, TextInput,
    },
    Alignment::Center,
    Background, Border, Color, Element,
    Length::Fill,
    Task,
};

use crate::{collection::Collection, Action, KeyAcceptor};

#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    InputSubmit,
    Start(Arc<Collection>),
    KeyPressed(Named),
}

#[derive(Debug, Clone)]
enum Answer {
    Correct,
    Incorrect,
    None,
}

#[derive(Debug, Clone)]
pub struct LearnWidget {
    selected_collection: Option<Arc<Collection>>,
    selected_collection_words: Option<Vec<String>>,
    answer: Answer,
    word_index: usize,
    inputed: String,
}
impl LearnWidget {
    pub fn new() -> Self {
        Self {
            selected_collection: None,
            word_index: 0,
            answer: Answer::None,
            inputed: String::new(),
            selected_collection_words: None,
        }
    }
    pub fn update(&mut self, message: Message) -> Either<Task<Message>, Action> {
        match message {
            Message::InputChanged(input) => self.inputed = input,
            Message::InputSubmit => {
                if self.inputed.is_empty() {
                    return Left(Task::none());
                }
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

                if is_correct {
                    self.answer = Answer::Correct;
                } else {
                    self.answer = Answer::Incorrect;
                }
                if self.word_index < self.selected_collection_words.as_ref().unwrap().len() - 1 {
                    self.word_index += 1
                } else {
                    self.word_index = 0;
                }
                self.inputed.clear();
            }
            Message::Start(c) => {
                self.selected_collection = Some(c.clone());
                self.answer = Answer::None;
                self.inputed.clear();
                self.word_index = 0;
                self.selected_collection_words =
                    Some(c.words().keys().map(|v| v.to_owned()).collect())
            }
            Message::KeyPressed(k) => match k {
                Named::Escape => return Right(Action::ChangeScreen("setup_screen".into())),
                Named::Enter => match self.answer {
                    Answer::Correct | Answer::Incorrect => {
                        self.answer = Answer::None;
                        return Left(text_input::focus("learn_input_id"));
                    }
                    Answer::None => (),
                },
                _ => {}
            },
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
        match self.answer {
            Answer::Correct => container(text("Correct").size(60))
                .center(Fill)
                .style(|v| container::background(Color::new(0., 1., 0., 1.)))
                .into(),
            Answer::Incorrect => container(text("Incorrect").size(60))
                .center(Fill)
                .style(|v| container::background(Color::new(1., 0., 0., 1.)))
                .into(),
            Answer::None => container(
                column![
                    text(word).size(48),
                    text_input("Type...", &self.inputed)
                        .id("learn_input_id")
                        .on_input(Message::InputChanged)
                        .on_submit(Message::InputSubmit),
                ]
                .spacing(30)
                .align_x(Center)
                .width(400),
            )
            .center(Fill)
            .into(),
        }
    }
}

impl Default for LearnWidget {
    fn default() -> Self {
        Self::new()
    }
}
