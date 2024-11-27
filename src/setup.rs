use std::sync::Arc;

use either::Either::{self, Left, Right};
use iced::{widget::{button, column, container, pick_list, radio, row, slider, text, tooltip}, Alignment::Center, Element, Length::Fill, Task};
use rfd::FileDialog;
use walkdir::WalkDir;

use crate::{collection::Collection, Action};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WorkMode {
    LearnMode,
    TestMode,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectedCollection(Arc<Collection>),
    SelectedMode(WorkMode),
    SelectedNumberOfWords(u8),
    CollectionsLoaded(Option<Vec<Arc<Collection>>>),
    CollectionDirectoryButton,
    StartButtonClicked,
}

#[derive(Debug, Clone)]
pub struct SetupWidget {
    selected_collection: Option<Arc<Collection>>,
    selected_work_mode: WorkMode,
    selected_number_of_test_words: u8,
    collections: Vec<Arc<Collection>>,
}

impl SetupWidget {
    pub fn new() -> Self {
        Self {
            selected_collection: None,
            selected_work_mode: WorkMode::LearnMode,
            collections: Vec::new(),
            selected_number_of_test_words: 5,
        }
    }
    pub fn update(&mut self, message: Message) -> Either<Task<Message>, Action> {
        match message {
            Message::SelectedCollection(v) => self.selected_collection = Some(v),
            Message::SelectedMode(mode) => self.selected_work_mode = mode,
            Message::SelectedNumberOfWords(num) => self.selected_number_of_test_words = num,
            Message::CollectionsLoaded(colls) => {
                if let Some(colls) = colls {
                    if (!colls.is_empty()) {
                        self.collections = colls.clone();
                        if self.selected_collection.is_none() {
                            self.selected_collection = colls.first().cloned();
                        }
                        // return Right(Action::SendColections(colls.clone()))
                    }
                }
            }
            Message::CollectionDirectoryButton => {
                return Left(Task::perform(
                    async {
                        if let Some(p) = FileDialog::new().set_directory(".").pick_folder() {
                            let mut colls = Vec::new();

                            for (id, file) in WalkDir::new(p)
                                .max_depth(1)
                                .min_depth(1)
                                .into_iter()
                                .flatten()
                                .enumerate()
                            {
                                let a = Arc::new(
                                    Collection::new_from_path(file.into_path(), id + 1).unwrap(),
                                );
                                colls.push(a)
                                // if line.get(0).unwrap() == &'@' {}
                                // println!("{:#?}", file.into_path());
                            }
                            Some(colls)
                        } else {
                            None
                        }
                    },
                    Message::CollectionsLoaded,
                ))
                
            }
            Message::StartButtonClicked => {
                if self.selected_collection.is_some() {
                    return Right(match self.selected_work_mode{
                        WorkMode::LearnMode => Action::StartLearnMode(self.selected_collection.clone().unwrap()),
                        WorkMode::TestMode => Action::StartTestMode(self.selected_collection.clone().unwrap(), self.selected_number_of_test_words.into()),
                    });
                    // return Right(Action::ChangeScreen("learn_screen".into()))
                }
            }
        }
        Left(Task::none())
    }

    pub fn view(&self) -> Element<Message> {
        container(container(column![
            column![
            row![button("Load").on_press(Message::CollectionDirectoryButton),
            pick_list(
                self.collections.clone(),
                self.selected_collection.clone(),
                |v| { Message::SelectedCollection(v) }
            )].padding(5).spacing(2),
            row![
            tooltip(
                radio(
                    "Learn Mode",
                    WorkMode::LearnMode,
                    Some(self.selected_work_mode),
                    Message::SelectedMode
                ),
                "In this mode you will indefinitly shown questions,\n there is no timer, and to stop just quit",
                tooltip::Position::Top
            ).gap(10).style(container::rounded_box),
            radio(
                "Test Mode",
                WorkMode::TestMode,
                Some(self.selected_work_mode),
                Message::SelectedMode
            ),

            ]
                .spacing(10),
                row![
                    tooltip(
                        text!(
                            "Phrase count ({:02})",
                            self.selected_number_of_test_words
                        ),
                        "In Test Mode",
                        tooltip::Position::Top
                    ).gap(10).style(container::rounded_box), 
                    slider(
                        5..=30,
                        self.selected_number_of_test_words,
                        Message::SelectedNumberOfWords
                    )
                ].spacing(10),

                ]
                    .spacing(10)
                    .height(Fill)
                    .width(Fill)
                    .align_x(Center)
                    .padding(10),
                    row![
                        button("Start").on_press(Message::StartButtonClicked),
                    ].padding(5).spacing(5)
                    ]).width(400).height(500).style(container::bordered_box)
                    )
                    .center(Fill).into()
    }
}

impl Default for SetupWidget {
    fn default() -> Self {
        Self::new()
    }
}
