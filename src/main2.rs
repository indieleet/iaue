//! # [Ratatui] Modifiers example
//!
//! The latest version of this example is available in the [examples] folder in the repository.
//!
//! Please note that the examples are designed to be run against the `main` branch of the Github
//! repository. This means that you may not be able to compile with the latest release version on
//! crates.io, or the one that you have installed locally.
//!
//! See the [examples readme] for more information on finding examples that match the version of the
//! library you are using.
//!
//! [Ratatui]: https://github.com/ratatui-org/ratatui
//! [examples]: https://github.com/ratatui-org/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui-org/ratatui/blob/main/examples/README.md

/// This example is useful for testing how your terminal emulator handles different modifiers.
/// It will render a grid of combinations of foreground and background colors with all
/// modifiers applied to them.
use std::{
    error::Error,
    io::{self, Stdout},
    iter::once,
    result,
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

use rodio::source::{SineWave, Source};
use rodio::{dynamic_mixer, OutputStream, Sink};

use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

fn play_auio(sink: &Sink) {
    // Construct a dynamic controller and mixer, stream_handle, and sink.
    let (controller, mixer) = dynamic_mixer::mixer::<f32>(2, 44_100);

    // Create four unique sources. The frequencies used here correspond
    // notes in the key of C and in octave 4: C4, or middle C on a piano,
    // E4, G4, and A4 respectively.
    let source_c = SineWave::new(261.63)
        .take_duration(Duration::from_secs_f32(1.))
        .amplify(0.20);
    let source_e = SineWave::new(329.63)
        .take_duration(Duration::from_secs_f32(1.))
        .amplify(0.20);
    let source_g = SineWave::new(392.0)
        .take_duration(Duration::from_secs_f32(1.))
        .amplify(0.20);
    let source_a = SineWave::new(440.0)
        .take_duration(Duration::from_secs_f32(1.))
        .amplify(0.20);

    // Add sources C, E, G, and A to the mixer controller.
    controller.add(source_c);
    controller.add(source_e);
    controller.add(source_g);
    controller.add(source_a);

    // Append the dynamic mixer to the sink to play a C major 6th chord.
    sink.append(mixer);

    // Sleep the thread until sink is empty.
    sink.sleep_until_end();
}
type Result<T> = result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let res = run_app(&mut terminal, &sink);
Python::with_gil(|py| {
        let custom_manager = PyModule::from_code_bound(
            py,
            r#"
class House(object):
    def __init__(self, address):
        self.address = address
    def __enter__(self):
        print(f"Welcome to {self.address}!")
    def __exit__(self, type, value, traceback):
        if type:
            print(f"Sorry you had {type} trouble at {self.address}")
        else:
            print(f"Thank you for visiting {self.address}, come again soon!")

        "#,
            "house.py",
            "house",
        )
        .unwrap();

        let house_class = custom_manager.getattr("House").unwrap();
        let house = house_class.call1(("123 Main Street",)).unwrap();

        house.call_method0("__enter__").unwrap();

        let result = py.eval_bound("undefined_variable + 1", None, None);

        // If the eval threw an exception we'll pass it through to the context manager.
        // Otherwise, __exit__  is called with empty arguments (Python "None").
        match result {
            Ok(_) => {
                let none = py.None();
                house
                    .call_method1("__exit__", (&none, &none, &none))
                    .unwrap();
            }
            Err(e) => {
                house
                    .call_method1(
                        "__exit__",
                        (
                            e.get_type_bound(py),
                            e.value_bound(py),
                            e.traceback_bound(py),
                        ),
                    )
                    .unwrap();
            }
        }
    });
    restore_terminal(terminal)?;
    if let Err(err) = res {
        eprintln!("{err:?}");
    }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, sink: &Sink) -> io::Result<()> {
    loop {
        terminal.draw(ui)?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
                if let KeyCode::Char(' ') = key.code {
                    play_auio(&sink);
                }
            }
        }
    }
}

fn ui(frame: &mut Frame) {
    let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
    let [text_area, main_area] = vertical.areas(frame.size());
    frame.render_widget(
        Paragraph::new("Note: not all terminals support all modifiers")
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        text_area,
    );
    let layout = Layout::vertical([Constraint::Length(1); 50])
        .split(main_area)
        .iter()
        .flat_map(|area| {
            Layout::horizontal([Constraint::Percentage(20); 5])
                .split(*area)
                .to_vec()
        })
        .collect_vec();

    let colors = [
        Color::Black,
        Color::DarkGray,
        Color::Gray,
        Color::White,
        Color::Red,
    ];
    let all_modifiers = once(Modifier::empty())
        .chain(Modifier::all().iter())
        .collect_vec();
    let mut index = 0;
    for bg in colors.iter() {
        for fg in colors.iter() {
            for modifier in &all_modifiers {
                let modifier_name = format!("{modifier:11?}");
                let padding = (" ").repeat(12 - modifier_name.len());
                let paragraph = Paragraph::new(Line::from(vec![
                    modifier_name.fg(*fg).bg(*bg).add_modifier(*modifier),
                    padding.fg(*fg).bg(*bg).add_modifier(*modifier),
                    // This is a hack to work around a bug in VHS which is used for rendering the
                    // examples to gifs. The bug is that the background color of a paragraph seems
                    // to bleed into the next character.
                    ".".black().on_black(),
                ]));
                frame.render_widget(paragraph, layout[index]);
                index += 1;
            }
        }
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
