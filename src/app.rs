use tui::widgets::TableState;
use crate::error::ErrorKind;

use crate::updater::is_installed;

pub enum InputMode {
    Normal,
    Editing,
}

pub enum Status {
    Error(ErrorKind),
    InstallSuccess(String),
    RemoveSuccess(String),
}

pub struct App<'a> {
    pub state: TableState,
    pub input: String,
    pub input_mode: InputMode,
    pub items: Vec<Vec<&'a str>>,
    pub installed: bool,
    pub status: Option<Status>,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let items = if !is_installed() {
            vec![
                vec!["Install"],
                vec!["Custom Install"],
                vec!["Repository"],
            ]
        } else {
            vec![
                vec!["Uninstall"],
                vec!["Update"],
                vec!["Repository"],
            ]
        };

        App {
            state: TableState::default(),
            input: String::new(),
            input_mode: InputMode::Normal,
            items,
            installed: is_installed(),
            status: None,
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
