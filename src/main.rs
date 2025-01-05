mod help;
mod init_config;
mod app;
use crate::app::{App, Mode, NormalCursor, VisualCursor, InsertCursor};
mod sdl_event;
use crate::sdl_event::{sdl_event, SdlBackend};
mod fns;
mod synths;
//#[cfg(not(feature = "sdl"))]
//mod crossterm_event;

#[cfg(not(feature = "sdl"))]
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

#[cfg(not(feature = "sdl"))]
use ratatui::backend::CrosstermBackend;

use ratatui::{layout::Direction, prelude::*, style::Stylize, widgets::*, Terminal};

#[cfg(feature = "sdl")]
use ratatui::backend::Backend;

use style::Styled;
// TODO: #[cfg(not(feature = "sdl"))]
use tinyaudio::prelude::*;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{stdout, Result};
use std::{
    fs,
    io::{Read, Write},
    process::{Command, Stdio},
};



#[derive(Serialize, Deserialize)]
struct JsonColor(String);

impl From<JsonColor> for ratatui::style::Color {
    fn from(item: JsonColor) -> Self {
        ratatui::style::Color::from_u32(u32::from_str_radix(&item.0[1..], 16).unwrap())
    }
}

struct TableWithCells<'a> {
    app: &'a App<'a>,
}

impl Widget for TableWithCells<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let constr_col = Layout::horizontal(vec![
            Constraint::Max(1),
            Constraint::Min(1),
            Constraint::Max(1),
        ])
        .split(area);
        let constr_rows = Layout::vertical(vec![
            Constraint::Max(1),
            Constraint::Min(1),
            Constraint::Max(3),
        ])
        .split(constr_col[1]);
        let temp_bound = self.app.count_bound(); //TODO: don't call this fn every time
        let constr_x = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    vec![Constraint::Max(4)],
                    vec![Constraint::Max(14); self.app.normal_cursor.x.saturating_sub(1) as usize],
                    vec![Constraint::Max(temp_bound as u16)],
                    vec![
                        Constraint::Max(14);
                        self.app.cols.len() - self.app.normal_cursor.x as usize - 1
                    ],
                ]
                .concat(),
            )
            .split(constr_rows[1]);

        let (max_vx, min_vx) = if self.app.normal_cursor.x >= self.app.visual_cursor.x {
            (self.app.normal_cursor.x, self.app.visual_cursor.x)
        } else {
            (self.app.visual_cursor.x, self.app.normal_cursor.x)
        };
        let (max_vy, min_vy) = if self.app.normal_cursor.y >= self.app.visual_cursor.y {
            (self.app.normal_cursor.y, self.app.visual_cursor.y)
        } else {
            (self.app.visual_cursor.y, self.app.normal_cursor.y)
        };

        //Highlight line
        match self.app.current_mode {
            Mode::Normal | Mode::Insert => {
                buf.set_span(
                    constr_col[1].x,
                    self.app.normal_cursor.y + constr_rows[1].y,
                    &Span::from(" ".repeat(area.width as usize)).bg(self.app.theme["bg_highlight"]),
                    area.width - 2,
                );
            }
            _ => {}
        }
        for (col_i, col) in self.app.cols.iter().enumerate() {
            let constr_y = ratatui::layout::Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Max(1); col.len()])
                .split(constr_x[col_i]);
            for (i, el) in col.iter().enumerate() {
                let curr_len = el.len();
                let line_bound = if curr_len > 7 { 7 } else { curr_len };
                let bounded_el = if (col_i == self.app.normal_cursor.x as usize)
                    && (i == self.app.normal_cursor.y as usize)
                {
                    &el[..]
                } else {
                    &el[..line_bound]
                };
                let constr_c = if col_i != 0 {
                    layout::Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            bounded_el
                                .iter()
                                .map(|it| Constraint::Max(it.content.len() as u16 + 1))
                                .collect::<Vec<_>>(),
                        )
                        .split(constr_y[i])
                } else {
                    layout::Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(vec![Constraint::Max(3)])
                        .split(constr_y[i])
                };
                for (ci, c) in bounded_el.iter().enumerate() {
                    let (cell_style, inside_style) = match self.app.current_mode {
                        Mode::Visual
                            if (i >= min_vy as usize && i <= max_vy as usize)
                                && (col_i >= min_vx as usize && col_i <= max_vx as usize) =>
                        {
                            (Modifier::REVERSED, Modifier::REVERSED)
                        }
                        Mode::Normal
                            if (col_i == self.app.normal_cursor.x as usize
                                && i == self.app.normal_cursor.y as usize) =>
                        {
                            (Modifier::REVERSED, Modifier::REVERSED)
                        }
                        Mode::Insert
                            if (col_i == self.app.normal_cursor.x as usize
                                && i == self.app.normal_cursor.y as usize
                                && ci == self.app.insert_cursor.x as usize) =>
                        {
                            (Modifier::REVERSED, Modifier::default())
                        }
                        _ => (Modifier::default(), Modifier::default()),
                    };
                    let c_len = if c.content.is_empty() {
                        1
                    } else {
                        c.content.len() as u16
                    };
                    let printed_cell = if !c.content.is_empty() {
                        &c.clone().patch_style(cell_style)
                    } else {
                        &Span::from(" ").patch_style(cell_style)
                    };
                    buf.set_span(constr_c[ci].x, constr_c[ci].y, printed_cell, c_len);
                    match ci {
                        0 | 2 | 4 if (i > 1) && (col_i > 0) => {
                            buf.set_span(
                                constr_c[ci].x + c_len,
                                constr_c[ci].y,
                                &Span::from("/").style(inside_style),
                                1,
                            );
                        }
                        ci if (i > 1) && (col_i > 0) => {
                            buf.set_span(
                                constr_c[ci].x + c_len,
                                constr_c[ci].y,
                                &Span::from(" ").set_style(inside_style),
                                1,
                            );
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    path: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    ///Render file to wav format
    Render {
        file_path: Option<String>,
        output_path: Option<String>,
    },
}


#[cfg(not(feature = "sdl"))]
fn init_panic_hook() {
    use std::panic::{set_hook, take_hook};
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        let _ = restore_tui();
        original_hook(panic_info);
    }));
}

use std::io;
#[cfg(not(feature = "sdl"))]
fn restore_tui() -> io::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

#[cfg(not(feature = "sdl"))]
pub fn crossterm_event(
    app: &mut App,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<()> {
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



//fn sdl_event(app: &mut App) -> Result<()> {
//for event in sdl_context.event_pump()?.poll_iter() {
//            match event {
//                Event::KeyDown {
//                    keycode: Some(Keycode::Escape),
//                    ..
//                }
//                | Event::Quit { .. } => break 'mainloop,
//                _ => {}
//            }
//        }
//
//}

fn start_app(working_file: &str) -> Result<()> {
    #[cfg(not(feature = "sdl"))]
    init_panic_hook();
    let mut config_raw_text = String::new();
    let config_path = home::home_dir()
        .unwrap()
        .join(".config")
        .join("iaue")
        .join("theme.json");
    if !home::home_dir().unwrap().exists() {
        let _ = fs::create_dir(home::home_dir().unwrap());
    };
    if !config_path.parent().unwrap().parent().unwrap().exists() {
        let _ = fs::create_dir(config_path.parent().unwrap().parent().unwrap());
    };
    if !config_path.parent().unwrap().exists() {
        let _ = fs::create_dir(config_path.parent().unwrap());
    };
    if !config_path.exists() {
        let mut new_config = fs::File::create_new(&config_path).unwrap();
        let _ = new_config.write_all(init_config::INIT_CONFIG.as_bytes());
    };
    let mut config_file = std::fs::File::open(config_path).unwrap();
    let _ = config_file.read_to_string(&mut config_raw_text);
    let config_file: HashMap<String, style::Color> =
        serde_json::from_str::<HashMap<String, JsonColor>>(&config_raw_text)
            .unwrap()
            .into_iter()
            .map(|(key, val)| (key, val.into()))
            .collect();

    #[cfg(not(feature = "sdl"))]
    stdout().execute(EnterAlternateScreen)?;
    #[cfg(not(feature = "sdl"))]
    enable_raw_mode()?;
    #[cfg(not(feature = "sdl"))]
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    #[cfg(feature = "sdl")]
    let mut terminal = Terminal::new(SdlBackend::new())?;

    terminal.clear()?;
    let mut app = App {
        normal_cursor: NormalCursor { x: 1, y: 1 },
        visual_cursor: VisualCursor { x: 1, y: 1 },
        insert_cursor: InsertCursor::default(),
        current_mode: Mode::Normal,
        audio_params: OutputDeviceParameters {
            channels_count: 2,
            sample_rate: 44100,
            channel_sample_count: 4410,
        },
        command_buf: String::new(),
        //file_path: std::env::current_dir().unwrap().to_str().unwrap_or("/").to_string(),
        file_name: working_file.to_string(),
        theme: config_file,
        x_bound: 0,
        y_bound: 0,
        current_times: String::new(),
        cols: vec![
            vec![vec![Span::from("1").to_owned()]; 3],
            vec![
                vec![Span::from("name").to_owned()],
                vec![
                    Span::from("440").to_owned(),
                    Span::from("1").to_owned(),
                    Span::from("1").to_owned(),
                ],
                vec![Span::from("1").to_owned(); 7],
                vec![Span::from("1").to_owned(); 7],
            ],
            vec![
                vec![Span::from("name").to_owned()],
                vec![
                    Span::from("440").to_owned(),
                    Span::from("1").to_owned(),
                    Span::from("1").to_owned(),
                ],
                vec![Span::from("1").to_owned(); 7],
            ],
        ],
        yank_buf: Vec::new(),
        //constrains: vec![Constraint::Max(3); 6],
        help_page: 0,
        is_help: false,
        should_leave: false,
        x_active: false
    };
    let fn_status = String::new();
    // let full_path_file =
    //     std::path::Path::new(&std::env::current_dir().unwrap().to_str().unwrap_or("/"))
    //         .join("project.tr");
    app.y_bound = app.cols[app.normal_cursor.x as usize].len() as u16;
    app.count_lines();
    loop {
        //let table_cols = app.cols.to_owned();
        //table_rows[app.normal_cursor.y as usize][app.normal_cursor.x as usize] = cur_cell_text;
        //let y_bound = core::iter::repeat_with(|| &app.rows.iter().next().unwrap_or(&Vec::<Span>::new()).get(app.normal_cursor.x as usize)).count();
        // for el in &app.rows[app.normal_cursor.x as usize] {
        //     if el.get(app.normal_cursor.x as usize).is_some() {
        //         y_bound += 1;
        //     }
        //     else {
        //         break;
        //     }
        // }
        //let x = Row::new(vec![vec![Span::from("1"), Span::from("1")]].iter().flatten().collect::Vec<Span>());
        let mode_str = match app.current_mode {
            Mode::Normal => Span::from("Normal").bg(app.theme["blue"]),
            Mode::Visual => Span::from("Visual").bg(app.theme["magenta"]),
            Mode::Insert => Span::from("Insert").bg(app.theme["green"]),
            Mode::Command => Span::from("Command").bg(app.theme["orange"]),
        };
        let mode_str_width = mode_str.to_string().len() as u16;
        terminal.draw(|f| {
            app.x_bound = f.area().width;
            app.y_bound = f.area().height;
            // let size_x = ratatui::layout::Layout::default()
            //     .direction(Direction::Vertical)
            //     .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            //     .split(f.area());
            f.render_widget(Block::new().bg(app.theme["bg"]), f.area());
            // f.render_widget(
            //     Table::new(
            //         table_cols.clone().into_iter().enumerate().map(|(i, row)| {
            //             if i == app.normal_cursor.y as usize {
            //                 Row::new(row.into_iter().flatten().collect::<Vec<Span>>())
            //                     .bg(Color::Rgb(0x2f, 0x33, 0x4d))
            //             } else {
            //                 Row::new(row.into_iter().flatten().collect::<Vec<Span>>())
            //             }
            //         }),
            //         app.constrains.to_owned(),
            //     )
            //     .block(Block::bordered()),
            //     size_x[0],
            // );
            f.render_widget(
                Block::bordered(),
                layout::Rect {
                    x: f.area().x,
                    y: f.area().y,
                    width: f.area().width,
                    height: f.area().height - 2,
                },
            );
            f.render_widget(TableWithCells { app: &app }, f.area());
            f.render_widget(
                format!(
                    "{}:{}:{}",
                    app.normal_cursor.x, app.normal_cursor.y, app.insert_cursor.x
                ),
                layout::Rect {
                    x: app.x_bound - 9,
                    y: app.y_bound - 2,
                    width: 9,
                    height: 1,
                },
            );
            f.render_widget(
                mode_str,
                layout::Rect {
                    x: 0,
                    y: app.y_bound - 2,
                    width: mode_str_width,
                    height: 1,
                },
            );
            f.render_widget(
                &app.command_buf,
                layout::Rect {
                    x: 0,
                    y: app.y_bound - 1,
                    width: app.command_buf.len() as u16,
                    height: 1,
                },
            );
            f.render_widget(
                fn_status.clone().set_style(Modifier::REVERSED),
                layout::Rect {
                    x: 0,
                    y: app.y_bound - 3,
                    width: f.area().width,
                    height: 1,
                },
            );
            let ct = &app.current_times;
            f.render_widget(
                ct,
                layout::Rect {
                    x: f.area().width - ct.len() as u16,
                    y: f.area().height - 1,
                    width: ct.len() as u16,
                    height: 1,
                },
            );
            let lines_count = help::TEXT[app.help_page].lines().count() as u16 + 2;
            let width = help::TEXT[app.help_page]
                .lines()
                .map(|it| it.len())
                .max()
                .unwrap_or(0) as u16
                + 2;
            if app.is_help {
                f.render_widget(
                    Paragraph::new(help::TEXT[app.help_page]).block(
                        Block::bordered()
                            .title_alignment(Alignment::Center)
                            .title("Help"),
                    ),
                    layout::Rect {
                        x: f.area().width / 2 - width / 2,
                        y: f.area().height / 2 - lines_count / 2,
                        width,
                        height: lines_count,
                    },
                );
            }
        })?;

        #[cfg(not(feature = "sdl"))]
        let _ = crossterm_event(&mut app, &mut terminal);
        #[cfg(feature = "sdl")]
        sdl_event(&mut app, &mut terminal);

        if app.should_leave {
            break;
        };

        #[cfg(feature = "sdl")]
        ::std::thread::sleep(core::time::Duration::new(0, 1_000_000_000u32 / 30));
    }

    #[cfg(not(feature = "sdl"))]
    stdout().execute(LeaveAlternateScreen)?;
    #[cfg(not(feature = "sdl"))]
    disable_raw_mode()?;
    Ok(())
}
fn main() {
    let cli = Cli::parse();

    use std::path::Path;
    let mut working_file: &str = "project.tr";
    if let Some(path) = cli.path.as_deref() {
        if std::path::Path::new(path).is_dir() {
            let _ = std::env::set_current_dir(path);
        } else {
            let path = Path::new(path);
            working_file = path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            let _ = std::env::set_current_dir(path.parent().unwrap_or(Path::new("/")));
        };
    }
    let _ = start_app(working_file);
}
