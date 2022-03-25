use std::{error::Error, io};

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
use tui::layout::Alignment;
use tui::style::Color::Rgb;
use tui::widgets::{Cell, Row, Table, Wrap};
use unicode_width::UnicodeWidthStr;

use crate::{CURRENT_VERSION, HOSTS_BEBASIN, HOSTS_HEADER, REPOSITORY_URL, updater};
use crate::app::{App, InputMode};
use crate::error::ErrorKind;
use crate::helpers::AppendableMap;
use crate::os::HOSTS_BACKUP_PATH;
use crate::parser::{parse_from_file, parse_from_str, write_to_file};

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
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
                    KeyCode::Enter => {
                        match app.state.selected() {
                            Some(selection) => {
                                match selection {
                                    0 => {

                                    }
                                    1 => {
                                        app.input_mode = InputMode::Editing
                                    }
                                    2 => {

                                    }
                                    _ => {}
                                }
                            }
                            None => {}
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

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(3)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Length(4),
            ]
                .as_ref(),
        )
        .split(f.size());


    let wrapper =
        Block::default().borders(Borders::ALL)
            .title_alignment(Alignment::Center)
            .title("Bebasin");

    let create_block = |title: String| {
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::White).fg(Color::Black))
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
    };

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

    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Custom Host Path"));
    f.render_widget(input, chunks[1]);
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