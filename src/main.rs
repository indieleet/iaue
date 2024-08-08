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
        constrains: vec![Constraint::Min(1)],
    };
    loop {
        terminal.draw(|f| {
            let size_x = ratatui::layout::Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(f.size());
            f.render_widget(
                Table::new(app.rows.to_owned(), app.constrains.to_owned()).block(Block::bordered()),
                size_x[0],
            );
            f.render_widget(Paragraph::new("Hello").block(Block::bordered()), size_x[1]);
        })?;
        // TODO main loop
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('+') => {
                        app.counter = app.counter.saturating_add(1);
                        app.rows.push(Row::new(vec!["7/9 1/2 1"]));
                        //app.constrains.push(Constraint::Percentage(30));
                    }
                    KeyCode::Char('-') => {
                        app.counter = app.counter.saturating_sub(1);
                        app.rows.pop();
                        //app.constrains.pop();
                    }
                    _ => (),
                }
            }
        }
    }
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
