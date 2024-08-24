use ratatui::layout::Direction;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        cursor,
        event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    style::{Color, Stylize},
    widgets::*,
    Terminal,
};
use std::process::{Command, Stdio};
use std::
    io::{stdout, Result};
use style::Styled;

use tinyaudio::prelude::*;

#[derive(Debug, Default)]
pub struct App<'a> {
    counter: u16,
    cursor: u8,
    items: Vec<Constraint>,
    current_times: String,
    columns: u8,
    x_bound: u16,
    y_bound: u16,
    cols: Vec<Vec<Vec<Span<'a>>>>,
    constrains: Vec<Constraint>,
}

#[derive(Debug)]
enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
}

#[derive(Debug, Default)]
struct NormalCursor {
    x: u16,
    y: u16,
}

#[derive(Debug, Default)]
struct InsertCursor {
    x: u16,
}

#[derive(Debug, Default)]
struct VisualCursor {
    x: u16,
    y: u16,
}

struct TableWithCells<'a> {
    content: Vec<Vec<Vec<Span<'a>>>>,
}

impl Widget for TableWithCells<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let constr_x = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Max(14); self.content.len()])
            .split(area);
        for (col_i, col) in self.content.iter().enumerate() {
            let constr_y = ratatui::layout::Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Max(1); col.len()])
                .split(constr_x[col_i]);
            for (i, el) in col.iter().enumerate() {
                let constr_c = layout::Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Max(4); el.len()])
                    .split(constr_y[i]);
                for (ci, c) in el.iter().enumerate() {
                    buf.set_span(constr_c[ci].x, constr_c[ci].y, c, 12);
                    match ci {
                        0 | 2 | 4 => {
                            buf.set_span(constr_c[ci].x + 1, constr_c[ci].y, &Span::from("/"), 1);
                        }
                        1 | 3 | 5 => {
                            buf.set_span(constr_c[ci].x + 1, constr_c[ci].y, &Span::from(" "), 1);
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    let mut normal_cursor = NormalCursor::default();
    let mut insert_cursor = InsertCursor::default();
    let mut visual_cursor = VisualCursor::default();
    let mut command_buf = String::new();
    let mut current_mode = Mode::Normal;
    let mut app = App {
        counter: 1,
        cursor: 0,
        columns: 1,
        x_bound: 0,
        y_bound: 0,
        current_times: "".into(),
        items: vec![Constraint::Percentage(30), Constraint::Fill(1)],
        cols: vec![
            vec![vec![Span::from("1").to_owned(); 7]; 3],
            vec![vec![Span::from("1").to_owned(); 7]; 1],
        ],
        constrains: vec![Constraint::Max(3); 6],
    };
    let mut fn_status = String::new();
    let cur_path = std::env::var("HOME").unwrap();
    let cur_filename = "rust_lib.rs";
    let audio_params = OutputDeviceParameters {
        channels_count: 1,
        sample_rate: 44100,
        channel_sample_count: 4410,
    };
    //let mut state = TableState::new();
    loop {
        let full_path = cur_path.clone() + "/" + cur_filename;
        let mut table_cols = app.cols.to_owned();
        let (max_vx, min_vx) = if normal_cursor.x >= visual_cursor.x {
            (normal_cursor.x, visual_cursor.x)
        } else {
            (visual_cursor.x, normal_cursor.x)
        };
        let (max_vy, min_vy) = if normal_cursor.y >= visual_cursor.y {
            (normal_cursor.y, visual_cursor.y)
        } else {
            (visual_cursor.y, normal_cursor.y)
        };
        let mut rand_iter = core::iter::repeat_with(|| fastrand::u8(0..9));
        for (i_col, c_col) in table_cols.iter_mut().enumerate() {
            for (i_el, el) in c_col.iter_mut().enumerate() {
                match current_mode {
                    Mode::Normal
                        if (normal_cursor.y as usize, normal_cursor.x as usize)
                            == (i_el, i_col) =>
                    {
                        *el = el
                            .clone()
                            .into_iter()
                            .map(|it| it.add_modifier(Modifier::REVERSED))
                            .collect();
                    }
                    Mode::Visual
                        if (i_el >= min_vy as usize && i_el <= max_vy as usize)
                            && (i_col >= min_vx as usize && i_col <= max_vx as usize) =>
                    {
                        *el = el
                            .clone()
                            .into_iter()
                            .map(|it| it.add_modifier(Modifier::REVERSED))
                            .collect();
                    }
                    Mode::Insert
                        if (normal_cursor.y as usize, normal_cursor.x as usize)
                            == (i_el, i_col) =>
                    {
                        *el = el
                            .clone()
                            .into_iter()
                            .enumerate()
                            .map(|(i, it)| {
                                if i == insert_cursor.x as usize {
                                    it.add_modifier(Modifier::REVERSED)
                                } else {
                                    it
                                }
                            })
                            .collect();
                    }
                    _ => (),
                }
            }
        }
        //table_rows[normal_cursor.y as usize][normal_cursor.x as usize] = cur_cell_text;
        //let y_bound = core::iter::repeat_with(|| &app.rows.iter().next().unwrap_or(&Vec::<Span>::new()).get(normal_cursor.x as usize)).count();
        let y_bound: u16 = app.cols[normal_cursor.x as usize].len() as u16;
        // for el in &app.rows[normal_cursor.x as usize] {
        //     if el.get(normal_cursor.x as usize).is_some() {
        //         y_bound += 1;
        //     }
        //     else {
        //         break;
        //     }
        // }
        //let x = Row::new(vec![vec![Span::from("1"), Span::from("1")]].iter().flatten().collect::Vec<Span>());
        let mode_str = match current_mode {
            Mode::Normal => Span::from("Normal").bg(Color::Blue),
            Mode::Visual => Span::from("Visual").bg(Color::Magenta),
            Mode::Insert => Span::from("Insert").bg(Color::Green),
            Mode::Command => Span::from("Command").bg(Color::Yellow),
        };
        let mode_str_width = mode_str.to_string().len() as u16;
        terminal.draw(|f| {
            app.x_bound = f.size().width;
            app.y_bound = f.size().height;
            let size_x = ratatui::layout::Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(f.size());
            f.render_widget(Block::new().bg(Color::Rgb(0x22, 0x24, 0x36)), f.area());
            f.render_widget(
                Table::new(
                    table_cols.clone().into_iter().enumerate().map(|(i, row)| {
                        if i == normal_cursor.y as usize {
                            Row::new(row.into_iter().flatten().collect::<Vec<Span>>())
                                .bg(Color::Rgb(0x2f, 0x33, 0x4d))
                        } else {
                            Row::new(row.into_iter().flatten().collect::<Vec<Span>>())
                        }
                    }),
                    app.constrains.to_owned(),
                )
                .block(Block::bordered()),
                size_x[0],
            );

            //for (i_row, c_row) in app.rows.iter().enumerate() {
            //    for (i_el, c_el) in c_row {
            //        f.render_widget(c_el, app.constrains[i_row][i_el])
            //    }
            //}
            f.render_widget(Block::bordered(), size_x[1]);
            f.render_widget(
                TableWithCells {
                    content: table_cols,
                },
                size_x[1],
            );
            f.render_widget(
                format!(
                    "{}:{}:{}",
                    normal_cursor.x, normal_cursor.y, insert_cursor.x
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
                &command_buf,
                layout::Rect {
                    x: 0,
                    y: app.y_bound - 1,
                    width: command_buf.len() as u16,
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
                    x: f.size().width - ct.len() as u16,
                    y: f.size().height - 1,
                    width: ct.len() as u16,
                    height: 1,
                },
            );
        })?;
        let match_event = event::read()?;
        match match_event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => match current_mode {
                Mode::Visual | Mode::Normal | Mode::Insert => {
                    break;
                }
                Mode::Command => {
                    command_buf.push('q');
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char(matched_code @ '0'..='9'),
                ..
            }) => match current_mode {
                Mode::Normal | Mode::Visual => {
                    app.current_times.push(matched_code);
                }
                Mode::Insert => {
                    let temp_span = app.cols[normal_cursor.x as usize][normal_cursor.y as usize]
                        [insert_cursor.x as usize]
                        .clone();
                    app.cols[normal_cursor.x as usize][normal_cursor.y as usize]
                        [insert_cursor.x as usize]
                        .content =
                        (temp_span.content.to_string() + &matched_code.to_string()).into();
                }
                Mode::Command => {
                    command_buf.push(matched_code);
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => {
                current_mode = Mode::Normal;
                let _ = &command_buf.clear();
                let _ = &app.current_times.clear();
            }

            Event::Key(KeyEvent {
                code: KeyCode::Char('i'),
                ..
            }) => match current_mode {
                Mode::Normal | Mode::Visual => {
                    current_mode = Mode::Insert;
                    let _ = &app.current_times.clear();
                }
                Mode::Command => {
                    command_buf.push('i');
                }
                Mode::Insert => {}
            },
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Char('r'),
                ..
            }) => match current_mode {
                Mode::Insert => {
                    let temp_cell = app.cols[normal_cursor.x as usize][normal_cursor.y as usize]
                        [insert_cursor.x as usize]
                        .clone();
                    app.cols[normal_cursor.x as usize][normal_cursor.y as usize]
                        [insert_cursor.x as usize] =
                        temp_cell.content(format!("{}", rand_iter.next().unwrap()));
                }
                Mode::Normal => {}
                Mode::Command => {
                    command_buf.push('r');
                }
                _ => (),
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char('h') | KeyCode::Left,
                ..
            }) => {
                let count: u16 = app.current_times.parse().unwrap_or(1);
                let _ = &app.current_times.clear();
                match current_mode {
                    Mode::Normal | Mode::Visual => {
                        //let x_bound = app.rows[normal_cursor.y as usize].len() as u16;
                        normal_cursor.x = normal_cursor.x.saturating_sub(count);
                    }
                    Mode::Insert => {
                        // 5 - 7
                        let new_cursor = insert_cursor.x as isize - count as isize;
                        insert_cursor.x = if new_cursor >= 0 {
                            new_cursor as u16
                        } else {
                            (7 + new_cursor % 7) as u16
                        };
                        normal_cursor.x = normal_cursor
                            .x
                            .saturating_sub(((new_cursor - 6) / 7).unsigned_abs() as u16);
                    }
                    Mode::Command => {
                        command_buf.push('h');
                    }
                }
                //normal_cursor.x = normal_cursor.x.saturating_sub(count);
            }
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('j') | KeyCode::Down,
                ..
            }) => {
                let count: u16 = app.current_times.parse().unwrap_or(1);
                let _ = &app.current_times.clear();
                //cursor.y = cursor.y.saturating_add(count);
                //let new_y = normal_cursor.y.saturating_add(count);
                match current_mode {
                    Mode::Normal | Mode::Visual | Mode::Insert => {
                        //let y_bound = app.rows[normal_cursor.y as usize].len() as u16;
                        let new_y = normal_cursor.y.saturating_add(count);
                        normal_cursor.y = if new_y > y_bound - 1 {
                            y_bound - 1
                        } else {
                            new_y
                        };
                    }
                    Mode::Command => {
                        command_buf.push('j');
                    }
                }
                //normal_cursor.y = if new_y > app.y_bound - 1 { app.y_bound - 1 }
                //    else { new_y };
            }
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('k') | KeyCode::Up,
                ..
            }) => {
                let count: u16 = app.current_times.parse().unwrap_or(1);
                let _ = &app.current_times.clear();
                match current_mode {
                    Mode::Normal | Mode::Visual | Mode::Insert => {
                        normal_cursor.y = normal_cursor.y.saturating_sub(count);
                    }
                    Mode::Command => {
                        command_buf.push('k');
                    }
                }
                //normal_cursor.y = normal_cursor.y.saturating_sub(count);
            }
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('l') | KeyCode::Right,
                ..
            }) => {
                let count: u16 = app.current_times.parse().unwrap_or(1);
                let _ = &app.current_times.clear();
                match current_mode {
                    Mode::Normal | Mode::Visual => {
                        let x_bound = app.cols.len() as u16;
                        let new_x = normal_cursor.x.saturating_add(count);
                        normal_cursor.x = if new_x > x_bound - 1 {
                            x_bound - 1
                        } else {
                            new_x
                        };
                    }
                    Mode::Insert => {
                        let new_cursor = insert_cursor.x + count as u16;
                        insert_cursor.x = (new_cursor) % 7;
                        normal_cursor.x = normal_cursor.x.saturating_add(new_cursor / 7);
                    }

                    Mode::Command => {
                        command_buf.push('l');
                    }
                }
                //cursor.x = if new_x > app.x_bound - 1 { app.x_bound - 1 }
                //    else { new_x };
            }
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('G'),
                ..
            }) => {
                let count: u16 = app.current_times.parse().unwrap_or(y_bound - 1);
                let _ = &app.current_times.clear();
                match current_mode {
                    Mode::Normal | Mode::Visual => {
                        normal_cursor.y = if count <= y_bound - 1 {
                            count
                        } else {
                            y_bound - 1
                        };
                    }
                    Mode::Command => {
                        command_buf.push('G');
                    }
                    Mode::Insert => todo!(),
                }
                //cursor.x = if new_x > app.x_bound - 1 { app.x_bound - 1 }
                //    else { new_x };
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('+'),
                ..
            }) => {
                match current_mode {
                    Mode::Insert | Mode::Normal | Mode::Visual => {
                        app.cols[normal_cursor.x as usize].push(vec![Span::from("1"); 7]);
                    }
                    Mode::Command => {
                        command_buf.push('+');
                    }
                }
                // let len_rows = app.rows[y_bound as usize - 1].len();
                // if (y_bound as usize) < app.rows.len() {
                //     app.rows[y_bound as usize].extend(vec![vec![Span::from("1/1");3]; (normal_cursor.x as usize + 1).saturating_sub(len_rows - 1)]);
                // }
                // else {
                //     app.rows.push(vec![vec![Span::from("1/1"); 3]; normal_cursor.x as usize + 1]);
                // }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('='),
                ..
            }) => match current_mode {
                Mode::Insert | Mode::Normal | Mode::Visual => {
                    app.cols.push(vec![vec![Span::from("1"); 7]]);
                }
                Mode::Command => {
                    command_buf.push('=');
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                ..
            }) => match current_mode {
                Mode::Insert | Mode::Normal | Mode::Visual => {
                    if y_bound - 2 < normal_cursor.y {
                        normal_cursor.y = normal_cursor.y.saturating_sub(1);
                    }
                    app.cols[normal_cursor.x as usize].remove(normal_cursor.y as usize);
                }
                Mode::Command => {
                    command_buf.push('d');
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char('-'),
                ..
            }) => match current_mode {
                Mode::Insert | Mode::Normal | Mode::Visual => {
                    app.cols.remove(normal_cursor.x as usize);
                    normal_cursor.x = if app.cols.len() < normal_cursor.x as usize {
                        app.cols.len() as u16
                    } else {
                        normal_cursor.x
                    };
                }
                Mode::Command => {
                    command_buf.push('-');
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char('v'),
                ..
            }) => match current_mode {
                Mode::Insert | Mode::Normal | Mode::Visual => {
                    current_mode = Mode::Visual;
                    (visual_cursor.x, visual_cursor.y) = (normal_cursor.x, normal_cursor.y);
                }
                Mode::Command => {
                    command_buf.push('v');
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) => match current_mode {
                Mode::Insert => {
                    let temp_span = app.cols[normal_cursor.x as usize][normal_cursor.y as usize]
                        [insert_cursor.x as usize]
                        .clone();
                    let slice_len = if temp_span.content.is_empty() { 0 } else { temp_span.content.len() - 1 };
                    let new_line = &mut temp_span.content.to_string()[..slice_len];
                    app.cols[normal_cursor.x as usize][normal_cursor.y as usize]
                        [insert_cursor.x as usize]
                        .content = (String::from(new_line)).into();
                }
                 Mode::Normal | Mode::Visual => {
                }
                Mode::Command => {
                    command_buf.pop();
                    if command_buf.is_empty() { current_mode = Mode::Normal; }
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char(':'),
                ..
            }) => match current_mode {
                Mode::Insert | Mode::Normal | Mode::Visual => {
                    current_mode = Mode::Command;
                    command_buf.push(':');
                }
                Mode::Command => {
                    command_buf.push(':');
                }
            },
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('e'),
                ..
            }) => {
                stdout().execute(LeaveAlternateScreen)?;
                disable_raw_mode()?;
                Command::new("nvim").arg(full_path).status()?;
                stdout().execute(EnterAlternateScreen)?;
                enable_raw_mode()?;
                let _ = terminal.clear();
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                code: KeyCode::Char(matched_code @ ' '..='~'),
                ..
            }) => {
                command_buf.push(matched_code);
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('r'),
                ..
            }) => {
                let lib_name = "./librust_lib.so";
                //let mut file = std::fs::File::create(cur_filename).expect("can't find file");
                //let mut file = std::fs::File::open(&full_path).expect("can't find file");
                // use std::io::Write;
                // file.write_all(
                //     r#"#[no_mangle] pub extern "C" fn f0(f: f32, l: f32, v: f32, t: usize, p: &[f32]) -> Vec<f32> {
                //          let freq = t as f32 / f;
                //          let length = l * t as f32;
                //          (0..freq as usize)
                //          .map(|it| it as f32 / f - 0.5)
                //          .map(|it| it * v)
                //          .cycle()
                //          .take(length as usize)
                //          .collect()
                //     }
                //     "#
                //         .as_bytes(),
                // )?;
                // drop(file);
                let comp_status = std::process::Command::new("rustc")
                    .arg("-C")
                    .arg("target-feature=-crt-static")
                    .arg("--crate-type")
                    .arg("cdylib")
                    .arg(full_path)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()?;
                let mut output: Vec<Vec<Vec<f32>>> = vec![Vec::new(); app.cols.len()];
                let (mut ft, mut lt, mut vt) = (440.0, 1.0, 1.0);
                let mut unique_fn: Vec<String> = Vec::new();
                for col in &app.cols {
                    for el in col {
                        if !unique_fn.contains(&el[6].to_string()) {
                            unique_fn.push(el[6].to_string().clone());
                        }
                    }
                }
                let mut fns = std::collections::HashMap::new();
                fn f1(f: f32, l: f32, v: f32, t: usize, p: &[f32]) -> Vec<f32>
                {
                    vec![0.0; (l * t as f32) as usize]
                }
                if comp_status.success() {
                    unsafe {
                        let lib = libloading::Library::new(lib_name).unwrap();
                        for el in unique_fn {
                        let f0 = lib.get::<libloading::Symbol<unsafe extern "C" fn(f32, f32, f32, usize, &[f32]) -> Vec<f32>>>(("f".to_string() + &el).as_bytes());
                       // let f0: FnType = match f0_probe {
                       //     Ok(val) => FnType::FnLib(val),
                       //     Err(_) => FnType::Fn(f1)
                       // };
                            fns.insert(el.clone(), f0);
                        }
                        //let f0_probe = lib.get(b"f0");

                        for (i, col) in app.cols.iter().enumerate() {
                            let (mut fs, mut ls, mut vs) = (440.0, 1.0, 1.0);
                            for el in col {
                                //      let temp_args: Vec<Vec<String>> = el
                                //          .iter()
                                //          .map(|it| it.to_string().split("/").map(String::from).collect::<Vec<String>>())
                                //          .collect();
                                //.map(|(x, _, y)| str::parse::<usize>(x).unwrap() as f32 / str::parse::<usize>(y).unwrap() as f32))
                                let elems: Vec<_> = el.iter().take(6).clone().collect();
                                let mut vec_args = Vec::with_capacity(3);
                                for indx in 0..3 {
                                    vec_args.push(
                                        str::parse::<usize>(&elems[indx * 2].content).unwrap()
                                            as f32
                                            / str::parse::<usize>(&elems[indx * 2 + 1].content)
                                                .unwrap()
                                                as f32,
                                    );
                                }
                                // let mut vec_args = Vec::with_capacity(3);
                                // for param in temp_args {
                                //     vec_args.push(str::parse::<usize>(&param[0]).unwrap() as f32 / str::parse::<usize>(&param[1]).unwrap() as f32);
                                //}
                                
                                let (f, l, v) = (vec_args[0], vec_args[1], vec_args[2]);
                                (fs, ls, vs) = (fs * f, ls * l, v * vs);
                                (ft, lt, vt) = (fs * f, ls * l, v * vs); // TODO: REMOVE THIS
                                let pushed_fn = &fns[&el[6].content.to_string()];
                                match pushed_fn {
                                    Ok(val) => {output[i].push(val(fs, ls, vs, 44100, &[]));},
                                    Err(_) => {output[i].push(f1(fs, ls, vs, 44100, &[]));},
                                }
                            }
                        }
                        let max_len = output
                            .iter()
                            .map(|it| it.iter().flatten().count())
                            .max()
                            .unwrap_or(0);
                        let mut out_vec = vec![0.0; max_len];
                        for column in output {
                            for (i, el) in column.iter().flatten().enumerate() {
                                out_vec[i] += *el;
                            }
                        }
                        let mut out_vec_iter = out_vec.into_iter();
                        fn_status = format!("{}, {}, {}, {}", ft, lt, vt, (max_len / 44100) as f32);
                        //let mut out_vec = output.into_iter().flatten().chain(core::iter::once(0.0).cycle());
                        std::thread::spawn(move || {
                            let _aud = run_output_device(audio_params, move |data| {
                                for samples in data {
                                    *samples = out_vec_iter.next().unwrap_or(0.0);
                                }
                            })
                            .unwrap();
                            std::thread::sleep(std::time::Duration::from_secs(100));
                        });
                    }
                }
            }
            _ => (),
        }
    }
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
