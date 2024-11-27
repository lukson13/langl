#![allow(dead_code, unused)]
mod collection;
mod learn;
mod modal;
mod setup;
mod testing;

use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Read},
    sync::Arc,
    thread,
};

use collection::Collection;
use either::Either::{Left, Right};
use iced::{
    keyboard::{key::Named, Key},
    widget::{
        button, center, column, combo_box, container, mouse_area, opaque, pick_list, radio, row,
        slider, stack, text, tooltip, Container,
    },
    Alignment::Center,
    Color, Element,
    Length::{self, Fill},
    Subscription, Task,
};
use learn::LearnWidget;
use modal::modal_base;
use rfd::{AsyncFileDialog, FileDialog};
use setup::SetupWidget;
use testing::TestWidget;
use walkdir::WalkDir;

// @ lang(en,pl, ja, kr, so on)
// $ name=name_value //only required variable will be displayed in a gui
// # - first in line indicates that the whole line is comment
// word | meaning1 / meaning2/.../ meaning_n // white spaces around word will be removed
// word2 | meaning1 / meaning2/.../ meaning_n // white spaces around meaning will be remowed.
//
//
//

// pub trait OwnWidget<T> {
//     fn update(&mut self, message: T) -> Task<T>;
//     fn view(&self) -> Element<T>;
// }
//

pub trait KeyAcceptor {
    fn key_pressed(&mut self, key: Named);
}

#[derive(Debug)]
pub enum AppScreen {
    LearnScreen(LearnWidget),
    SetupScreen(SetupWidget),
    TestingScreen(TestWidget),
    None,
}
#[derive(Debug)]
pub enum Action {
    ChangeScreen(String),
    // SendColections(Vec<Arc<Collection>>),
    StartLearnMode(Arc<Collection>),
    StartTestMode(Arc<Collection>, usize),
}

#[derive(Debug)]
pub enum Message {
    SetupMessage(String, setup::Message),
    LearnMessage(String, learn::Message),
    TestingMessage(String, testing::Message),
    KeyPressed(Named),
}

#[derive(Debug)]
pub struct App {
    active_screen: String, // selection_dialog: bool,
    screens: HashMap<String, AppScreen>,
    // collections: Vec<Arc<Collection>>,
}

impl Default for App {
    fn default() -> Self {
        let mut screens = HashMap::new();
        screens.insert(
            "learn_screen".into(),
            AppScreen::LearnScreen(LearnWidget::new()),
        );
        screens.insert(
            "setup_screen".into(),
            AppScreen::SetupScreen(SetupWidget::new()),
        );
        screens.insert(
            "testing_screen".into(),
            AppScreen::TestingScreen(TestWidget::new()),
        );
        Self {
            screens,
            active_screen: "setup_screen".into(),
            // collections: Vec::new(),
        }
    }
}

impl App {
    fn perform_action(&mut self, action: Action) {
        match action {
            Action::ChangeScreen(screen_name) => self.active_screen = screen_name,
            // Action::SendColections(colls) => self.collections = colls,
            Action::StartLearnMode(coll) => {
                if let AppScreen::LearnScreen(ls) = self.screens.get_mut("learn_screen").unwrap() {
                    ls.update(learn::Message::Start(coll));
                };
                self.active_screen = "learn_screen".into()
            }
            Action::StartTestMode(coll, num) => {
                if let AppScreen::TestingScreen(ls) =
                    self.screens.get_mut("testing_screen").unwrap()
                {
                    ls.update(testing::Message::Start(coll, num));
                };
                self.active_screen = "testing_screen".into()
            }
        }
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SetupMessage(key, sm) => {
                if let AppScreen::SetupScreen(ss) = self.screens.get_mut(&key).unwrap() {
                    match ss.update(sm) {
                        Left(l) => {
                            return l.map(|v| Message::SetupMessage("setup_screen".into(), v))
                        }
                        Right(action) => self.perform_action(action),
                    };
                };
            }
            Message::LearnMessage(key, lm) => {
                if let AppScreen::LearnScreen(ls) = self.screens.get_mut(&key).unwrap() {
                    match ls.update(lm) {
                        Left(l) => {
                            return l.map(|v| Message::LearnMessage("learn_screen".into(), v))
                        }
                        Right(action) => self.perform_action(action),
                    };
                };
            }
            Message::TestingMessage(key, tm) => {
                if let AppScreen::TestingScreen(ts) = self.screens.get_mut(&key).unwrap() {
                    match ts.update(tm) {
                        Left(l) => {
                            return l.map(|v| Message::TestingMessage("testing_screen".into(), v))
                        }
                        Right(action) => self.perform_action(action),
                    };
                };
            }
            Message::KeyPressed(k) => match self.screens.get_mut(&self.active_screen) {
                Some(AppScreen::LearnScreen(ls)) => {
                    match ls.update(learn::Message::KeyPressed(k)) {
                        Left(l) => {
                            return l.map(|v| Message::LearnMessage("learn_screen".into(), v))
                        }
                        Right(action) => self.perform_action(action),
                    };
                }
                Some(AppScreen::SetupScreen(ls)) => (),
                Some(AppScreen::TestingScreen(ls)) => {
                    match ls.update(testing::Message::KeyPressed(k)) {
                        Left(l) => {
                            return l.map(|v| Message::TestingMessage("testing_screen".into(), v))
                        }
                        Right(action) => self.perform_action(action),
                    };
                }
                Some(AppScreen::None) => (),
                None => (),
            },
        };
        Task::none()
    }
    pub fn view(&self) -> Element<Message> {
        match &self
            .screens
            .get(&self.active_screen)
            .unwrap_or(&AppScreen::None)
        {
            AppScreen::LearnScreen(ls) => ls
                .view()
                .map(|v| Message::LearnMessage("learn_screen".into(), v)),
            AppScreen::SetupScreen(ss) => ss
                .view()
                .map(|v| Message::SetupMessage("setup_screen".into(), v)),
            AppScreen::TestingScreen(ts) => ts
                .view()
                .map(|v| Message::TestingMessage("testing_screen".into(), v)),
            AppScreen::None => container(text("Smth is wrong")).center(Fill).into(),
        }
    }

    pub fn subscribe(&self) -> Subscription<Message> {
        iced::keyboard::on_key_press(|key, mods| {
            match key {
                Key::Named(Named::Enter) => Some(Message::KeyPressed(Named::Enter)),
                Key::Named(Named::Escape) => Some(Message::KeyPressed(Named::Escape)),
                _ => None,
            }
            // if Key::Named(Named::Enter) == key {
            //     Some()
            //     // Some(Message::LearnMessage(
            //     //     "learn_screen".into(),
            //     //     learn::Message::EnterPressed,
            //     // ))
            // } else {
            //     None
            // }
        })
    }
}

fn main() -> anyhow::Result<()> {
    Ok(iced::application("LangL", App::update, App::view)
        .subscription(App::subscribe)
        .run()?)
    // Ok(())
    // Ok(iced::run("LangL", App::update, App::view)?)
}
