use crate::app::App;
use crate::fns::*;
use ratatui::prelude::*;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
#[cfg(not(feature = "sdl"))]
pub fn crossterm_event(
    app: &mut App,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> std::io::Result<()> {
    let match_event = event::read()?;
    app.y_bound = app.cols[app.normal_cursor.x as usize].len() as u16;

    let editor = std::env::var("EDITOR").unwrap_or("nvim".to_string());
    let full_path_lib =
        std::path::Path::new(&std::env::current_dir().unwrap().to_str().unwrap_or("/"))
            .join("cargolib/")
            .join("src/")
            .join("lib.rs");
    let mut rand_iter = core::iter::repeat_with(|| fastrand::u8(0..=9));

    match match_event {
        Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            ..
        }) => false_quit(app),
        Event::Key(KeyEvent {
            code: KeyCode::Char(matched_code @ '0'..='9'),
            ..
        }) => num_to_buf(app, matched_code),
        Event::Key(KeyEvent {
            code: KeyCode::Char('.'),
            ..
        }) => dot(app),
        Event::Key(KeyEvent {
            code: KeyCode::Char(','),
            ..
        }) => comma(app),
        Event::Key(KeyEvent {
            code: KeyCode::Char('/'),
            ..
        }) => slash(app),
        Event::Key(KeyEvent {
            code: KeyCode::Esc, ..
        }) => escape(app),
        Event::Key(KeyEvent {
            code: KeyCode::Char('i'),
            ..
        }) => enter_insert_mode(app),
        Event::Key(KeyEvent {
            modifiers: KeyModifiers::NONE,
            code: KeyCode::Char('r'),
            ..
        }) => rand_interal(app, &mut rand_iter),
        Event::Key(KeyEvent {
            code: KeyCode::Char('h') | KeyCode::Left,
            ..
        }) => move_left(app),
        Event::Key(KeyEvent {
            //modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('j') | KeyCode::Down,
            ..
        }) => move_down(app),
        Event::Key(KeyEvent {
            //modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('k') | KeyCode::Up,
            ..
        }) => move_up(app),
        Event::Key(KeyEvent {
            //modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('l') | KeyCode::Right,
            ..
        }) => move_right(app),
        Event::Key(KeyEvent {
            //modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('G'),
            ..
        }) => goto_end(app),
        Event::Key(KeyEvent {
            //modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('g'),
            ..
        }) => goto_start(app),
        Event::Key(KeyEvent {
            code: KeyCode::Char('+'),
            ..
        }) => add_line(app),
        Event::Key(KeyEvent {
            code: KeyCode::Char('t'),
            ..
        }) => add_fx(app),
        Event::Key(KeyEvent {
            code: KeyCode::Char('T'),
            ..
        }) => remove_fx(app),
        Event::Key(KeyEvent {
            code: KeyCode::Char('='),
            ..
        }) => add_column(app),
        Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            ..
        }) => remove_line(app),
        Event::Key(KeyEvent {
            code: KeyCode::Char('y'),
            ..
        }) => yank(app),
        Event::Key(KeyEvent {
            code: KeyCode::Char('p'),
            ..
        }) => paste_down(app),
        Event::Key(KeyEvent {
            code: KeyCode::Char('P'),
            ..
        }) => paste_up(app),
        Event::Key(KeyEvent {
            code: KeyCode::Char('-'),
            ..
        }) => remove_column(app),
        Event::Key(KeyEvent {
            code: KeyCode::Char('v'),
            ..
        }) => enter_visual_mode(app),
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            ..
        }) => backspace(app),
        Event::Key(KeyEvent {
            code: KeyCode::Char(':'),
            ..
        }) => enter_command_mode(app),
        Event::Key(KeyEvent {
            code: KeyCode::Char('?'),
            ..
        }) => toggle_help(app),
        Event::Key(KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('e'),
            ..
        }) => open_editor(&editor, &full_path_lib, terminal)?,
        Event::Key(KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('s'),
            ..
        }) => {
            save_file(app, "".to_string())
            //use std::fs::File;
            //use std::io::{BufWriter, Write};
            //let file_cloned = &app
            //    .cols
            //    .clone()
            //    .into_iter()
            //    .map(|col| {
            //        col.into_iter()
            //            .map(|el| el.into_iter().map(|c| c.content).collect::<Vec<_>>())
            //            .collect::<Vec<_>>()
            //    })
            //    .collect::<Vec<_>>();
            //let file = File::create(&full_path_file).unwrap();
            //let mut writer = BufWriter::new(file);
            //serde_json::to_writer(&mut writer, &file_cloned).unwrap();
            //writer.flush().unwrap();
        }
        Event::Key(KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('o'),
            ..
        }) => {
            //use std::fs::File;
            //use std::io::Read;
            //let mut file = File::open(&full_path_file).unwrap();
            //let mut data = String::new();
            //file.read_to_string(&mut data).unwrap();
            //app.cols = serde_json::from_str::<Vec<Vec<Vec<String>>>>(&data)
            //    .unwrap()
            //    .into_iter()
            //    .map(|col| {
            //        col.into_iter()
            //            .map(|el| el.into_iter().map(Span::from).collect::<Vec<_>>())
            //            .collect::<Vec<_>>()
            //    })
            //    .collect::<Vec<_>>();
            //app.normal_cursor.x = 0;
            //app.normal_cursor.y = 0;
            //app.visual_cursor.x = 0;
            //app.visual_cursor.y = 0;
            //app.insert_cursor.x = 0;
            open_file(app, "".to_string());
        }
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            ..
        }) => child_execute_command(app),
        Event::Key(KeyEvent {
            modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            code: KeyCode::Char(matched_code @ ' '..='~'),
            ..
        }) => insert_symbol_to_cmd(app, matched_code),
        Event::Key(KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            code: KeyCode::Char('r'),
            ..
        }) => child_render(app),
        _ => (),
    }
    Ok(())
}
