use std::{
    alloc::Layout,
    io::{stdout, Result},
};

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    style::Stylize,
    widgets::*,
    Terminal,
};

#[derive(Debug, Default)]
pub struct App<'a> {
    counter: u16,
    cursor: u8,
    items: Vec<Constraint>,
    rows: Vec<Row<'a>>,
    constrains: Vec<Constraint>,
}

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    let mut app = App {
        counter: 1,
        cursor: 0,
        items: vec![Constraint::Percentage(30), Constraint::Fill(1)],
        rows: vec![Row::new(vec!["7/9 1/2 1"])],
        constrains: vec![Constraint::Percentage(30)],
    };
    loop {
        terminal.draw(|f| {
            let size_x = ratatui::layout::Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(f.size());
            let rows = [Row::new(vec!["7/9 1/2 1"])];
            let constr = vec![Constraint::Percentage(15)];
            f.render_widget(
                Table::new(app.rows.clone(), app.constrains.clone()).block(Block::bordered()),
                size_x[0],
            );
            f.render_widget(Paragraph::new("Hello").block(Block::bordered()), size_x[1]);
        })?;
        // TODO main loop
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                break;
            }
        }
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('+') {
                app.counter = app.counter.saturating_add(1);
                app.rows.push(Row::new(vec!["7/9 1/2 1"]));
                app.constrains.push(Constraint::Percentage(30));
            }
        }
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('-') {
                app.counter = app.counter.saturating_sub(1);
                app.rows.pop();
                app.constrains.pop();
            }
        }
    }
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
