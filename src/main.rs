extern crate crossterm;
extern crate cursive;
#[cfg(any(target_os = "linux", target_os = "macos"))]
extern crate nix;
extern crate pest;
extern crate webbrowser;
#[cfg(target_os = "windows")]
extern crate winapi;
#[macro_use]
extern crate pest_derive;
extern crate itertools;
extern crate zip;

mod app;
mod error;
mod helpers;
mod os;
mod parser;
mod ui;
mod updater;

#[allow(clippy::unreadable_literal)]
const CURRENT_VERSION: u64 = 202011120913;
const REPOSITORY_URL: &str = "https://github.com/bebasid/bebasin";
const LATEST_VERSION_URL: &str =
    "https://raw.githubusercontent.com/bebasid/bebasin/tree/master/hosts";
const UPDATE_URL: &str = "https://api.github.com/repos/bebasid/bebasid/releases/latest";
const HOSTS_HEADER: &str = include_str!("../misc/header-hosts");
const HOSTS_BEBASIN: &str = include_str!("../misc/hosts");
const DEFAULT_HOSTS: &str = include_str!("../misc/default-hosts");

use std::io;
use tui::{backend::CrosstermBackend, Terminal};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}
