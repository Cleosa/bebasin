use std::{error::Error, fs, io};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    Terminal,
    text::{Span, Spans, Text}, widgets::{Block, Borders, List, ListItem, Paragraph},
};
use tui::layout::{Alignment, Rect};
use tui::style::Color::Rgb;
use tui::widgets::{Cell, Clear, Row, Table, Wrap};
use unicode_width::UnicodeWidthStr;

use crate::{CURRENT_VERSION, HOSTS_BEBASIN, HOSTS_HEADER, REPOSITORY_URL, updater};
use crate::app::{App, InputMode, Status};
use crate::error::ErrorKind;
use crate::helpers::AppendableMap;
use crate::os::{HOSTS_BACKUP_PATH, HOSTS_PATH};
use crate::parser::{parse_from_file, parse_from_str, write_to_file};
use crate::updater::{backup, is_backed};

use std::collections::{HashMap, HashSet};

struct HostsData<'a> {
    hosts_path: Option<&'a str>,
    hosts_bebasin: Option<HashMap<String, HashSet<String>>>,
    hosts_header: Option<&'a str>,
}

impl<'a> HostsData<'a> {
    fn new() -> Self {
        Self {
            hosts_path: None,
            hosts_bebasin: None,
            hosts_header: None,
        }
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {

    let mut hosts_data = HostsData::new();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::Esc => {
                        if let Some(v) = &app.status {
                            if let Status::Success(_) = v {
                                app.status = None;
                                fs::remove_file(HOSTS_BACKUP_PATH);
                            }
                        }
                    }
                    KeyCode::Enter => {

                        if let Some(v) = &app.status {
                            if let Status::Success(_) = v {
                                let hosts_bebasin = if let Some(v) = &hosts_data.hosts_bebasin {
                                    v
                                }
                                else {
                                    panic!()
                                };
                                match write_to_file(&hosts_data.hosts_path.unwrap(), hosts_bebasin, &hosts_data.hosts_header.unwrap()) {
                                    Err(err) => {
                                        app.status = Some(Status::Error(err));
                                    }
                                    Ok(_) => {
                                        app.installed = true;
                                    }
                                }
                                app.status = None;
                                app.items = vec![
                                    vec!["Uninstall"],
                                    vec!["Update"],
                                    vec!["Repository"],
                                ]
                            }
                        }
                        else {
                            if !is_backed() {
                                let backup_result = backup();
                                if backup_result.is_err() {
                                    return Ok(());
                                }
                            }

                            if !app.installed {
                                match app.state.selected() {
                                    Some(selection) => {
                                        match selection {
                                            0 => {
                                                match parse_from_str(HOSTS_BEBASIN) {
                                                    Ok(mut hosts_bebasin) => {
                                                        match parse_from_file(HOSTS_BACKUP_PATH) {
                                                            Ok(hosts_backup) => {
                                                                hosts_bebasin.append(hosts_backup);
                                                                app.status = Some(Status::Success(String::from("Are you sure that you want to install bebasin?")));
                                                                hosts_data.hosts_path = Some(HOSTS_PATH);
                                                                hosts_data.hosts_bebasin = Some(hosts_bebasin);
                                                                hosts_data.hosts_header = Some(HOSTS_HEADER);
                                                            }
                                                            Err(err) => {
                                                                app.status = Some(Status::Error(err));
                                                            }
                                                        }
                                                    }
                                                    Err(err) => {
                                                        app.status = Some(Status::Error(err));
                                                    }
                                                }
                                            }
                                            1 => {
                                                app.input_mode = InputMode::Editing
                                            }
                                            2 => {
                                                webbrowser::open("https://github.com/mochidaz/bebasin");
                                            }
                                            _ => {}
                                        }
                                    }
                                    None => {}
                                }
                            }
                            else {

                            }
                        }

                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {}
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
                .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
                .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn confirmation<B: Backend>(f: &mut Frame<B>, msg: &String) {
    let block = Block::default().title("Confirmation").borders(Borders::ALL)
        .style(Style::default().bg(Rgb(0,0,0)));
    let paragraph = Paragraph::new(msg.to_string())
        .block(block.clone())
        .alignment(Alignment::Center);
    let area = centered_rect(60, 20, f.size());
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
    f.render_widget(block, area);
}

fn error<B: Backend>(f: &mut Frame<B>, error: &ErrorKind) {
    let block = Block::default().title(format!("An error occured!")).borders(Borders::ALL)
        .style(Style::default().bg(Rgb(0,0,0)));
    let text = vec![
        Spans::from(format!("Error: {}", error))
    ];
    let paragraph = Paragraph::new(text)
        .block(block.clone())
        .alignment(Alignment::Center);
    let area = centered_rect(60, 20, f.size());
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
    f.render_widget(block, area);
}

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(3)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Length(4),
                Constraint::Length(4),
            ]
                .as_ref(),
        )
        .split(f.size());


    let wrapper =
        Block::default().borders(Borders::ALL)
            .style(Style::default())
            .title_alignment(Alignment::Center)
            .title("Bebasin");

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Rgb(144, 238, 144));
    let header_cells = ["Installation Menu"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Rgb(0, 0, 0))));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(*c));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    let t = Table::new(rows)
        .header(header)
        .block(wrapper)
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);
    f.render_stateful_widget(t, chunks[0], &mut app.state);

    let status = if app.installed {
        "Installed"
    } else {
        "Not Installed"
    };

    let text = vec![
        Spans::from(format!("Status: {}", status))
    ];

    let create_block = |title| {
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
    };

    let paragraph = Paragraph::new(text.clone())
        .block(create_block("Status"))
        .alignment(Alignment::Left);
    f.render_widget(paragraph, chunks[1]);

    match &app.status {
        Some(v) => {
            match v {
                Status::Error(e) => {
                    error(f, e)
                }
                Status::Success(m) => {
                    confirmation(f, m)
                }
            }
        }
        None => {}
    }

    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Custom Host Path"));
    f.render_widget(input, chunks[2]);
    match app.input_mode {
        InputMode::Normal =>
            {}

        InputMode::Editing => {
            f.set_cursor(
                chunks[1].x + app.input.width() as u16 + 1,
                chunks[1].y + 1,
            )
        }
    }
}