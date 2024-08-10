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
use std::io::{stdout, Result};

#[derive(Debug, Default)]
pub struct App {
    counter: u16,
    cursor: u8,
    items: Vec<Constraint>,
    columns: u8,
    rows: Vec<Vec<String>>,
    constrains: Vec<Constraint>,
}

#[derive(Debug, Default)]
struct AppCursor {
    x: u16,
    y: u16,
}

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    let mut cursor = AppCursor::default();
    let mut app = App {
        counter: 1,
        cursor: 0,
        columns: 1,
        items: vec![Constraint::Percentage(30), Constraint::Fill(1)],
        rows: vec![
            vec!["7/9 1/2 1".into(), "1/2 3/5 1/2".into()],
            vec!["3/2 1 1".into()],
        ],
        constrains: vec![Constraint::Max(10)],
    };
    loop {
        terminal.draw(|f| {
            let size_x = ratatui::layout::Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(f.size());
            f.render_widget(
                Table::new(
                    app.rows.to_owned().into_iter().map(Row::new),
                    app.constrains.to_owned(),
                )
                .block(Block::bordered()),
                size_x[0],
            );
            f.render_widget(
                Paragraph::new(format!("Hello {}", app.counter)).block(Block::bordered()),
                size_x[1],
            );
            f.render_widget(
                " ".on_red(),
                layout::Rect {
                    x: cursor.x,
                    y: cursor.y,
                    width: 1,
                    height: 1,
                },
            )
        })?;
        // TODO main loop
        let match_event = event::read()?;
        match match_event {
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('q'),
                ..
            }) => 
                    {
                        break;
                    }
                   // KeyCode::Char('h') => {
                   //     cursor.x = cursor.x.saturating_sub(1);
                   // }
                   // KeyCode::Char('j') => {
                   //     cursor.y = cursor.y.saturating_add(1);
                   // }
                   // KeyCode::Char('k') => {
                   //     cursor.y = cursor.y.saturating_sub(1);
                   // }
                   // KeyCode::Char('l') => {
                   //     cursor.x = cursor.x.saturating_add(1);
                   // }
                   // KeyCode::Char('+') => {
                   //     app.counter = app.counter.saturating_add(1);
                   //     app.rows[0].push("7/9 1/2 1".into());
                   //     app.constrains.push(Constraint::Max(10));
                   // }
                   // KeyCode::Char('-') => {
                   //     app.counter = app.counter.saturating_sub(1);
                   //     app.rows[0].pop();
                   //     app.constrains.pop();
                   // }
                    _ => (),
                }
            }
        }
    }
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
