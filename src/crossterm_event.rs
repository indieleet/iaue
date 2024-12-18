#[cfg(not(feature = "sdl"))]
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use ratatui::prelude::*;

use tinyaudio::prelude::*;

use std::io::{stdout, Result};
use std:: process::Command;

#[cfg(not(feature = "sdl"))]
pub fn crossterm_event(app: &mut App) -> Result<()> {
    let match_event = event::read()?;
    let y_bound: u16 = app.cols[app.normal_cursor.x as usize].len() as u16;

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
            }) => match app.current_mode {
                Mode::Visual | Mode::Normal | Mode::Insert => {
                    app.is_help = false;
                    app.command_buf = "to quit type :q and hit enter".to_string();
                }
                Mode::Command => {
                    app.command_buf.push('q');
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char(matched_code @ '0'..='9'),
                ..
            }) => match app.current_mode {
                Mode::Normal | Mode::Visual => {
                    app.current_times.push(matched_code);
                }
                Mode::Insert => {
                    let temp_span = app.cols[app.normal_cursor.x as usize]
                        [app.normal_cursor.y as usize][app.insert_cursor.x as usize]
                        .clone();
                    app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                        [app.insert_cursor.x as usize]
                        .content =
                        (temp_span.content.to_string() + &matched_code.to_string()).into();
                }
                Mode::Command => {
                    app.command_buf.push(matched_code);
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char('.'),
                ..
            }) => match app.current_mode {
                Mode::Normal | Mode::Visual => {}
                Mode::Insert => {
                    let temp_span = app.cols[app.normal_cursor.x as usize]
                        [app.normal_cursor.y as usize][app.insert_cursor.x as usize]
                        .clone();
                    app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                        [app.insert_cursor.x as usize]
                        .content = (temp_span.content.to_string() + ".").into();
                }
                Mode::Command => {
                    app.command_buf.push('.');
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char(','),
                ..
            }) => match app.current_mode {
                Mode::Normal | Mode::Visual => {}
                Mode::Insert => {
                    let temp_span = app.cols[app.normal_cursor.x as usize]
                        [app.normal_cursor.y as usize][app.insert_cursor.x as usize]
                        .clone();
                    app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                        [app.insert_cursor.x as usize]
                        .content = (temp_span.content.to_string() + ",").into();
                }
                Mode::Command => {
                    app.command_buf.push(',');
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char('/'),
                ..
            }) => match app.current_mode {
                Mode::Normal | Mode::Visual => {}
                Mode::Insert => {
                    let temp_span = app.cols[app.normal_cursor.x as usize]
                        [app.normal_cursor.y as usize][app.insert_cursor.x as usize]
                        .clone();
                    app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                        [app.insert_cursor.x as usize]
                        .content = (temp_span.content.to_string() + "/").into();
                }
                Mode::Command => {
                    app.command_buf.push('/');
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => {
                app.is_help = false;
                app.current_mode = Mode::Normal;
                let _ = &app.command_buf.clear();
                let _ = &app.current_times.clear();
            }

            Event::Key(KeyEvent {
                code: KeyCode::Char('i'),
                ..
            }) => match app.current_mode {
                Mode::Normal | Mode::Visual => {
                    app.current_mode = Mode::Insert;
                    let _ = &app.current_times.clear();
                }
                Mode::Command => {
                    app.command_buf.push('i');
                }
                Mode::Insert => {}
            },
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Char('r'),
                ..
            }) => match app.current_mode {
                Mode::Insert => {
                    let temp_cell = app.cols[app.normal_cursor.x as usize]
                        [app.normal_cursor.y as usize][app.insert_cursor.x as usize]
                        .clone();
                    app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                        [app.insert_cursor.x as usize] =
                        temp_cell.content(format!("{}", rand_iter.next().unwrap()));
                }
                Mode::Normal => {}
                Mode::Command => {
                    app.command_buf.push('r');
                }
                _ => (),
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char('h') | KeyCode::Left,
                ..
            }) => {
                let count: u16 = app.current_times.parse().unwrap_or(1);
                let _ = &app.current_times.clear();
                match app.current_mode {
                    Mode::Normal | Mode::Visual => {
                        //let x_bound = app.rows[app.normal_cursor.y as usize].len() as u16;
                        let final_cursor = app.normal_cursor.x.saturating_sub(count);
                        if ((app.normal_cursor.y as usize) < app.cols[final_cursor as usize].len()) && (final_cursor > 0) { app.normal_cursor.x = final_cursor };
                    }
                    Mode::Insert => {
                        let new_cursor_insert = app.insert_cursor.x as isize - count as isize;
                        let insert_bound = app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize].len() as isize;
                       // let insert_bound = if app.normal_cursor.y == 0 {
                       //     1
                       // } else if app.normal_cursor.y == 1 {
                       //     3
                       // } else {
                       //     7
                       // };
                        let new_cursor_normal = app.normal_cursor.x as isize
                            - (((new_cursor_insert - insert_bound + 1) / insert_bound).abs());
                        if app.normal_cursor.y >= app.cols[new_cursor_normal as usize].len() as u16 {
                        }
                        else if new_cursor_normal > 0 {
                            app.insert_cursor.x = if new_cursor_insert >= 0 {
                                new_cursor_insert as u16
                            } else {
                                (insert_bound + new_cursor_insert % insert_bound) as u16
                            };
                            app.normal_cursor.x = new_cursor_normal as u16;
                        } else {
                            app.normal_cursor.x = 1;
                            app.insert_cursor.x = 0;
                        }
                    }
                    Mode::Command => {
                        app.command_buf.push('h');
                    }
                }
                //app.normal_cursor.x = app.normal_cursor.x.saturating_sub(count);
                if app.is_help {
                    if let Mode::Normal | Mode::Insert | Mode::Visual = app.current_mode {
                        app.help_page = app.help_page.saturating_sub(count as usize);
                    }
                }
            }
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('j') | KeyCode::Down,
                ..
            }) => {
                let count: u16 = app.current_times.parse().unwrap_or(1);
                let _ = &app.current_times.clear();
                //cursor.y = cursor.y.saturating_add(count);
                //let new_y = app.normal_cursor.y.saturating_add(count);
                match app.current_mode {
                    Mode::Normal | Mode::Visual | Mode::Insert => {
                        //let y_bound = app.rows[app.normal_cursor.y as usize].len() as u16;
                        let new_y = app.normal_cursor.y.saturating_add(count);
                        app.normal_cursor.y = if new_y > y_bound - 1 {
                            y_bound - 1
                        } else {
                            new_y
                        };
                        app.count_lines();
                    }
                    Mode::Command => {
                        app.command_buf.push('j');
                    }
                }
                //app.normal_cursor.y = if new_y > app.y_bound - 1 { app.y_bound - 1 }
                //    else { new_y };
            }
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('k') | KeyCode::Up,
                ..
            }) => {
                let count: u16 = app.current_times.parse().unwrap_or(1);
                let _ = &app.current_times.clear();
                match app.current_mode {
                    Mode::Normal | Mode::Visual | Mode::Insert => {
                        let new_cursor = app.normal_cursor.y.saturating_sub(count);
                        if new_cursor > 0 { app.normal_cursor.y = new_cursor };
                        app.count_lines();
                    }
                    Mode::Command => {
                        app.command_buf.push('k');
                    }
                }
                //app.normal_cursor.y = app.normal_cursor.y.saturating_sub(count);
            }
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('l') | KeyCode::Right,
                ..
            }) => {
                let count: u16 = app.current_times.parse().unwrap_or(1);
                let _ = &app.current_times.clear();
                match app.current_mode {
                    Mode::Normal | Mode::Visual => {
                        let x_bound = app.cols.len() as u16;
                        let new_x = app.normal_cursor.x.saturating_add(count);
                        app.normal_cursor.x = if new_x > x_bound - 1 {
                            x_bound - 1
                        } else if (app.normal_cursor.y as usize) < app.cols[new_x as usize].len() {
                            new_x
                        }
                            else { app.normal_cursor.x };
                    }
                    Mode::Insert => {
                        let insert_bound = app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize].len() as u16;
                        let new_cursor_insert = app.insert_cursor.x + count as u16;
                        let new_cursor_normal = app
                            .normal_cursor
                            .x
                            .saturating_add(new_cursor_insert / insert_bound);
                        if new_cursor_normal < app.cols.len() as u16 && app.normal_cursor.y >= app.cols[new_cursor_normal as usize].len() as u16 {
                        }
                        else if new_cursor_normal < app.cols.len() as u16 {
                            app.insert_cursor.x = (new_cursor_insert) % insert_bound;
                            app.normal_cursor.x = new_cursor_normal;
                        } else {
                            app.insert_cursor.x = insert_bound - 1;
                            app.normal_cursor.x = app.cols.len() as u16 - 1;
                        }
                    }

                    Mode::Command => {
                        app.command_buf.push('l');
                    }
                }
                //cursor.x = if new_x > app.x_bound - 1 { app.x_bound - 1 }
                if app.is_help {
                    if let Mode::Normal | Mode::Insert | Mode::Visual = app.current_mode {
                        let new_page = app.help_page.saturating_add(count as usize);
                        app.help_page = if new_page >= help::TEXT.len() { help::TEXT.len() - 1 } else { new_page }; 
                    }
                }
                //    else { new_x };
            }
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('G'),
                ..
            }) => {
                let count: u16 = app.current_times.parse().unwrap_or(y_bound - 1);
                let _ = &app.current_times.clear();
                match app.current_mode {
                    Mode::Normal | Mode::Visual => {
                        app.normal_cursor.y = if count < y_bound {
                            count
                        } else {
                            y_bound - 1
                        };
                        app.count_lines();
                    }
                    Mode::Command => {
                        app.command_buf.push('G');
                    }
                    Mode::Insert => {}
                }
                //cursor.x = if new_x > app.x_bound - 1 { app.x_bound - 1 }
                //    else { new_x };
            }
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('g'),
                ..
            }) => {
                let count: u16 = app.current_times.parse().unwrap_or(0);
                let _ = &app.current_times.clear();
                match app.current_mode {
                    Mode::Normal | Mode::Visual => {
                        app.normal_cursor.y = if count < y_bound {
                            count
                        } else {
                            y_bound - 1
                        };
                        app.count_lines();
                    }
                    Mode::Command => {
                        app.command_buf.push('G');
                    }
                    Mode::Insert => {}
                }
                //cursor.x = if new_x > app.x_bound - 1 { app.x_bound - 1 }
                //    else { new_x };
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('+'),
                ..
            }) => {
                match app.current_mode {
                    Mode::Insert | Mode::Normal | Mode::Visual => {
                        app.cols[app.normal_cursor.x as usize].push(vec![Span::from("1"); 7]);
                        app.count_lines();
                    }
                    Mode::Command => {
                        app.command_buf.push('+');
                    }
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('t'),
                ..
            }) => {
                match app.current_mode {
                    Mode::Insert | Mode::Normal | Mode::Visual => {
                        app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize].extend(vec![Span::from("0"), Span::from("0")]);
                        app.count_lines();
                    }
                    Mode::Command => {
                        app.command_buf.push('t');
                    }
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('T'),
                ..
            }) => {
                match app.current_mode {
                    Mode::Normal | Mode::Visual => {},
                    Mode::Insert => {
                        if app.insert_cursor.x % 2 == 0 {
                            app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize].remove(app.insert_cursor.x as usize);
                            app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize].remove(app.insert_cursor.x as usize - 1 );
                        }
                        else {
                            app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize].remove(app.insert_cursor.x as usize + 1);
                            app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize].remove(app.insert_cursor.x as usize);
                        };
                    }
                    Mode::Command => {
                        app.command_buf.push('T');
                    }
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('='),
                ..
            }) => match app.current_mode {
                Mode::Insert | Mode::Normal | Mode::Visual => {
                    app.cols.push(vec![
                        vec![Span::from("name")],
                        vec![Span::from("440"), Span::from("1"), Span::from("1")],
                        vec![Span::from("1"); 7],
                    ]);
                }
                Mode::Command => {
                    app.command_buf.push('=');
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                ..
            }) => match app.current_mode {
                Mode::Insert | Mode::Normal => {
                    app.cols[app.normal_cursor.x as usize].remove(app.normal_cursor.y as usize);
                    if y_bound - 2 < app.normal_cursor.y {
                        app.normal_cursor.y = app.normal_cursor.y.saturating_sub(1);
                    }
                    app.count_lines();
                    app.current_mode = Mode::Normal;
                }
                Mode::Visual => {
                    let (min_x, max_x) = minmax_x(&app);
                    let (min_y, max_y) = minmax_y(&app);
                    for i in min_x..=max_x {
                        app.cols[i as usize].drain((min_y as usize)..=(max_y as usize));
                    }
                    let new_bound = app.cols[app.normal_cursor.x as usize].len() as u16 - 1;
                    if new_bound < app.normal_cursor.y {
                        app.normal_cursor.y = new_bound; 
                    }
                    app.count_lines();
                    app.current_mode = Mode::Normal;
                }
                Mode::Command => {
                    app.command_buf.push('d');
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char('y'),
                ..
            }) => match app.current_mode {
                Mode::Insert | Mode::Normal  => {
                    app.yank_buf = vec![vec![app.cols[app.normal_cursor.x as usize]
                        [app.normal_cursor.y as usize]
                        .clone()]];
                    app.current_mode = Mode::Normal;
                }
                Mode::Visual => {
                    let (min_x, max_x) = minmax_x(&app);
                    let (min_y, max_y) = minmax_y(&app);
                    app.yank_buf = app.cols[(min_x as usize)..=(max_x as usize)].to_vec()
                        .iter()
                        .map(|it|
                        it[(min_y as usize)..=(max_y as usize)].to_vec()
                        )
                    .collect::<Vec<_>>();
                    app.current_mode = Mode::Normal;
                }
                Mode::Command => {
                    app.command_buf.push('y');
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char('p'),
                ..
            }) => {
                match app.current_mode {
                    Mode::Insert | Mode::Normal | Mode::Visual => {
                        if !app.yank_buf.is_empty() {
                            app.cols[app.normal_cursor.x as usize]
                                .insert(app.normal_cursor.y as usize + 1, app.yank_buf[0][0].clone());
                            app.count_lines();
                        }
                    }
                    Mode::Command => {
                        app.command_buf.push('p');
                    }
                }
                // let len_rows = app.rows[y_bound as usize - 1].len();
                // if (y_bound as usize) < app.rows.len() {
                //     app.rows[y_bound as usize].extend(vec![vec![Span::from("1/1");3]; (app.normal_cursor.x as usize + 1).saturating_sub(len_rows - 1)]);
                // }
                // else {
                //     app.rows.push(vec![vec![Span::from("1/1"); 3]; app.normal_cursor.x as usize + 1]);
                // }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('P'),
                ..
            }) => {
                match app.current_mode {
                    Mode::Insert | Mode::Normal | Mode::Visual => {
                        if !app.yank_buf.is_empty() {
                            app.cols[app.normal_cursor.x as usize]
                                .insert(app.normal_cursor.y as usize, app.yank_buf[0][0].clone());
                            app.count_lines();
                        }
                    }
                    Mode::Command => {
                        app.command_buf.push('P');
                    }
                }
                // let len_rows = app.rows[y_bound as usize - 1].len();
                // if (y_bound as usize) < app.rows.len() {
                //     app.rows[y_bound as usize].extend(vec![vec![Span::from("1/1");3]; (app.normal_cursor.x as usize + 1).saturating_sub(len_rows - 1)]);
                // }
                // else {
                //     app.rows.push(vec![vec![Span::from("1/1"); 3]; app.normal_cursor.x as usize + 1]);
                // }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('-'),
                ..
            }) => match app.current_mode {
               Mode::Normal  => {
                    app.cols.remove(app.normal_cursor.x as usize);
                    app.normal_cursor.x = if app.cols.len() - 1 < app.normal_cursor.x as usize {
                        app.cols.len() as u16 - 1
                    } else {
                        app.normal_cursor.x
                    };
                    app.count_lines();
                }
                Mode::Visual => {
                    let (min_x, max_x) = minmax_x(&app);
                    app.cols.drain((min_x as usize)..=(max_x as usize));
                    app.normal_cursor.x = if app.cols.len() - 1 < app.normal_cursor.x as usize {
                        app.cols.len() as u16 - 1
                    } else {
                        app.normal_cursor.x
                    };
                    app.current_mode = Mode::Normal;
                    app.count_lines();
                }
                Mode::Insert => {
                    let temp_span = app.cols[app.normal_cursor.x as usize]
                        [app.normal_cursor.y as usize][app.insert_cursor.x as usize]
                        .clone();
                    app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                        [app.insert_cursor.x as usize]
                        .content = (temp_span.content.to_string() + "-").into();
                }
                Mode::Command => {
                    app.command_buf.push('-');
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char('v'),
                ..
            }) => match app.current_mode {
                Mode::Insert | Mode::Normal | Mode::Visual => {
                    app.current_mode = Mode::Visual;
                    (app.visual_cursor.x, app.visual_cursor.y) =
                        (app.normal_cursor.x, app.normal_cursor.y);
                }
                Mode::Command => {
                    app.command_buf.push('v');
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) => match app.current_mode {
                Mode::Insert => {
                    let temp_span = app.cols[app.normal_cursor.x as usize]
                        [app.normal_cursor.y as usize][app.insert_cursor.x as usize]
                        .clone();
                    let slice_len = if temp_span.content.is_empty() {
                        0
                    } else {
                        temp_span.content.len() - 1
                    };
                    let new_line = &mut temp_span.content.to_string()[..slice_len];
                    app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                        [app.insert_cursor.x as usize]
                        .content = (String::from(new_line)).into();
                }
                Mode::Normal | Mode::Visual => {}
                Mode::Command => {
                    app.command_buf.pop();
                    if app.command_buf.is_empty() {
                        app.current_mode = Mode::Normal;
                    }
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char(':'),
                ..
            }) => match app.current_mode {
                Mode::Insert | Mode::Normal | Mode::Visual => {
                    app.current_mode = Mode::Command;
                    app.command_buf.clear();
                    app.command_buf.push(':');
                }
                Mode::Command => {
                    app.command_buf.push(':');
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char('?'),
                ..
            }) => match app.current_mode {
                Mode::Insert | Mode::Normal | Mode::Visual => {
                    app.is_help = !app.is_help;
                }
                Mode::Command => {}
            },
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('e'),
                ..
            }) => {
                stdout().execute(LeaveAlternateScreen)?;
                disable_raw_mode()?;
                Command::new(&editor).arg(&full_path_lib).status()?;
                stdout().execute(EnterAlternateScreen)?;
                enable_raw_mode()?;
                //let _ = terminal.clear();
            }
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
            }) => match app.current_mode {
                Mode::Command => exec_command(app),
                Mode::Normal | Mode::Insert | Mode::Visual => {}
            },
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                code: KeyCode::Char(matched_code @ ' '..='~'),
                ..
            }) => match app.current_mode {
                Mode::Command => {
                    app.command_buf.push(matched_code);
                }
                Mode::Normal | Mode::Visual | Mode::Insert => (),
            },
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('r'),
                ..
            }) => {
                let out_vec = render(app);
                let max_len = out_vec.len();
                let mut out_vec_iter = out_vec.into_iter();
            let audio_params = app.audio_params;
                        std::thread::spawn(move || {
                            let _aud = run_output_device(audio_params, move |data| {
                                for samples in data {
                                    *samples = out_vec_iter.next().unwrap_or(0.0);
                                }
                            })
                            .unwrap();
                            std::thread::sleep(std::time::Duration::from_secs(
                                max_len as u64 / 44100,
                            ));
                        });
                    }
            _ => (),
        }
    Ok(())
}

