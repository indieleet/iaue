use std::io::{stdout, Result};

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    style::Stylize,
    prelude::*,
    widgets::*,
    Terminal,
};

#[derive(Debug, Default)]
pub struct App {
    counter: u16,
    cursor: u8,
}

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    let mut app = App { counter: 1, cursor: 0 };
    loop { 
        terminal.draw(|frame| {
            let area = frame.size();
            for i in 0..app.counter as usize { 
                let cur_width = area.width / app.counter;
                frame.render_widget(
                    Block::new()
                    .title(format!("Hello Ratatui!, {} (press 'q' to quit)", app.counter))
                    .borders(Borders::LEFT)
                    .white()
                    .on_blue(),
                    ratatui::layout::Rect{
                        x: area.x + cur_width * i as u16, 
                        y: area.y,
                        width: cur_width,
                        height: area.height },
                );}
        })?;   // TODO main loop
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('+') {
                    app.counter = app.counter.saturating_add(1);
                }
            }
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('-') {
                    app.counter = app.counter.saturating_sub(1);
                }
            }
        }
    }
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
