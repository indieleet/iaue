use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        cursor, event::{self, Event, KeyCode, KeyEvent, KeyModifiers}, style::Color, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand
    },
    prelude::*,
    style::Stylize, widgets::*, Terminal,
};
use style::Styled;
use std::io::{stdout, Result};
use std::process::{Command, Stdio};

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
    rows: Vec<Vec<Span<'a>>>,
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
    y: u16,
}

#[derive(Debug, Default)]
struct VisualCursor {
    x: u16,
    y: u16,
}

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    let mut normal_cursor = NormalCursor::default();
    let mut insert_cursor = InsertCursor::default();
    let mut visual_cursor = VisualCursor::default();
    let mut current_mode = Mode::Normal;
    let mut app = App {
        counter: 1,
        cursor: 0,
        columns: 1,
        x_bound: 0,
        y_bound: 0,
        current_times: "".into(),
        items: vec![Constraint::Percentage(30), Constraint::Fill(1)],
        rows: vec![
            vec![Span::from("1/8 4/1 1/1").to_owned(), Span::from("1/2 3/5 1/2").to_owned()],
            vec![Span::from("3/2 1/1 1/2").to_owned()],
            vec![Span::from("3/2 1/1 1/2").to_owned()],
            vec![Span::from("3/2 1/1 2/1").to_owned()],
            vec![Span::from("3/2 1/1 2/1").to_owned()],
        ],
        constrains: vec![Constraint::Max(12); 2],
    };
    let mut fn_status: f32 = 0.0;
    let audio_params = OutputDeviceParameters {
    channels_count: 1,
    sample_rate: 44100,
    channel_sample_count: 4410,
};
    //let mut state = TableState::new();
    loop {
        let mut table_rows = app.rows.to_owned();
        let mut cur_cell_text = Span::default();
        let (max_vx, min_vx) = if normal_cursor.x >= visual_cursor.x { (normal_cursor.x, visual_cursor.x) }
        else { (visual_cursor.x, normal_cursor.x) };
        let (max_vy, min_vy) = if normal_cursor.y >= visual_cursor.y { (normal_cursor.y, visual_cursor.y) }
        else { (visual_cursor.y, normal_cursor.y) };
        for (i_row, c_row) in table_rows.iter_mut().enumerate() {
            for (i_el, el) in c_row.iter_mut().enumerate() {
                match current_mode {
                    Mode::Normal if (normal_cursor.x as usize, normal_cursor.y as usize) == (i_el, i_row) => {
                        *el = el.clone().add_modifier(Modifier::REVERSED);
                    },
                    Mode::Visual if (i_el >= min_vx as usize && i_el <= max_vx as usize) && (i_row >= min_vy as usize && i_row <= max_vy as usize) => {
                        *el = el.clone().add_modifier(Modifier::REVERSED);
                    },
                    Mode::Insert if (normal_cursor.x as usize, normal_cursor.y as usize) == (i_el, i_row) => {
                        cur_cell_text = el.clone().add_modifier(Modifier::REVERSED);
                    },
                    _ => ()
                }
            }
        }
        //table_rows[normal_cursor.y as usize][normal_cursor.x as usize] = cur_cell_text;
        //let y_bound = core::iter::repeat_with(|| &app.rows.iter().next().unwrap_or(&Vec::<Span>::new()).get(normal_cursor.x as usize)).count();
        let mut y_bound: u16 = 0;
        for el in &app.rows {
            if el.get(normal_cursor.x as usize).is_some() {
                y_bound += 1;
            }
            else {
                break;
            }
        }
        let mode_str = match current_mode { 
            Mode::Normal => "Normal",
            Mode::Visual => "Visual",
            Mode::Insert => "Insert",
            Mode::Command => "Command",
            };
        terminal.draw(|f| {
            app.x_bound = f.size().width;
            app.y_bound = f.size().height;
            let size_x = ratatui::layout::Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(f.size());
            f.render_widget(
                Table::new(
                    table_rows.into_iter().enumerate().map(|(i, row)| if i == normal_cursor.y as usize { Row::new(row).bg(Color::Grey) }
                    else { Row::new(row) }),
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
            f.render_widget(
                Paragraph::new(format!("Hello {}", app.counter)).block(Block::bordered()),
                size_x[1],
            );
            f.render_widget(mode_str, layout::Rect {
                x: 0,
                y: app.y_bound - 1,
                width: mode_str.len() as u16,
                height: 1
            }
            );
            f.render_widget(format!("{}", fn_status).set_style(Modifier::REVERSED), layout::Rect {
                x: 0,
                y: app.y_bound - 2,
                width: mode_str.len() as u16,
                height: 1
            }
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
            //f.render_widget(
            //    " ".on_red(),
            //    layout::Rect {
            //        x: cursor.x,
            //        y: cursor.y,
            //        width: 1,
            //        height: 1,
            //    },
            //)
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
                    },
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char(matched_code @ '0'..='9'),
                ..
            }) => 
                    {
                app.current_times.push(matched_code)
                    },
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Esc,
                ..
            }) => 
            {
                current_mode = Mode::Normal;
                let _ = &app.current_times.clear();
            },
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('h') | KeyCode::Left,
                ..
            }) => 
            {
                let count: u16 = app.current_times.parse().unwrap_or(1);
                let _ = &app.current_times.clear();
                match current_mode {
                    Mode::Normal | Mode::Visual => { 
                        //let x_bound = app.rows[normal_cursor.y as usize].len() as u16;
                        normal_cursor.x = normal_cursor.x.saturating_sub(count);
                    },
                    Mode::Insert => todo!(),
                    Mode::Command => todo!(),
                }
                //normal_cursor.x = normal_cursor.x.saturating_sub(count);
            },
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('j') | KeyCode::Down,
                ..
            }) => 
            {
                let count: u16 = app.current_times.parse().unwrap_or(1);
                let _ = &app.current_times.clear();
                //cursor.y = cursor.y.saturating_add(count);
                //let new_y = normal_cursor.y.saturating_add(count);
                match current_mode {
                    Mode::Normal | Mode::Visual => { 
                        //let y_bound = app.rows[normal_cursor.y as usize].len() as u16;
                        let new_y = normal_cursor.y.saturating_add(count);
                        normal_cursor.y = if new_y > y_bound - 1 { y_bound - 1 }
                            else { new_y };
                    },
                    Mode::Insert => todo!(),
                    Mode::Command => todo!(),
                }
                //normal_cursor.y = if new_y > app.y_bound - 1 { app.y_bound - 1 }
                //    else { new_y };
            },
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('k') | KeyCode::Up,
                ..
            }) => 
            {
                let count: u16 = app.current_times.parse().unwrap_or(1);
                let _ = &app.current_times.clear();
                match current_mode {
                    Mode::Normal | Mode::Visual => { 
                        normal_cursor.y = normal_cursor.y.saturating_sub(count);
                    },
                    Mode::Insert | Mode::Command => todo!(),
                }
                //normal_cursor.y = normal_cursor.y.saturating_sub(count);
            },
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('l') | KeyCode::Right,
                ..
            }) => 
            {
                let count: u16 = app.current_times.parse().unwrap_or(1);
                let _ = &app.current_times.clear();
                match current_mode {
                    Mode::Normal | Mode::Visual => { 
                        let x_bound = app.rows[normal_cursor.y as usize].len() as u16;
                        let new_x = normal_cursor.x.saturating_add(count);
                        normal_cursor.x = if new_x > x_bound - 1 { x_bound - 1 }
                            else { new_x };
                    },
                    Mode::Insert | Mode::Command => todo!(),
                }
                //cursor.x = if new_x > app.x_bound - 1 { app.x_bound - 1 }
                //    else { new_x };
            },
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('G'),
                ..
            }) => 
            {
                let count: u16 = app.current_times.parse().unwrap_or(y_bound-1);
                let _ = &app.current_times.clear();
                match current_mode {
                    Mode::Normal | Mode::Visual => { 
                        normal_cursor.y = if count <= y_bound - 1 { count } else { y_bound - 1 };
                    },
                    Mode::Insert | Mode::Command => todo!(),
                }
                //cursor.x = if new_x > app.x_bound - 1 { app.x_bound - 1 }
                //    else { new_x };
            },
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('+'),
                ..
            }) => 
            {
                let len_rows = app.rows[y_bound as usize - 1].len();
                if (y_bound as usize) < app.rows.len() {
                    app.rows[y_bound as usize].extend(vec![Span::from("1/1 1/1 1/1"); (normal_cursor.x as usize + 1).saturating_sub(len_rows - 1)]); 
                }
                else {
                    app.rows.push(vec![Span::from("1/1 1/1 1/1"); normal_cursor.x as usize + 1]); 
                }
            },
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('v'),
                ..
            }) => 
            {
                current_mode = Mode::Visual;
                (visual_cursor.x, visual_cursor.y) = (normal_cursor.x, normal_cursor.y);
            },
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('e'),
                ..
            }) => 
            { 
                stdout().execute(LeaveAlternateScreen)?;
                disable_raw_mode()?;
//.arg("/tmp/a.txt")
                Command::new("nvim").status()?;
                stdout().execute(EnterAlternateScreen)?;
                enable_raw_mode()?;
                let _ = terminal.clear();
                //let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
            },
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('r'),
                ..
            }) =>
            {

                let lib_name = "./librust_lib.so";
                let code_name = "rust_lib.rs";
                let mut file = std::fs::File::create(code_name).expect("can't find file");
                use std::io::Write;
                file.write_all(
                    r#"#[no_mangle] pub extern "C" fn f0(f: f32, l: f32, v: f32, t: usize, p: &[f32]) -> Vec<f32> {
                         let freq = t as f32 / f;
                         let length = l * t as f32;
                         (0..freq as usize)
                         .map(|it| it as f32 / f - 0.5).map(|it| it * v)
                         .cycle()
                         .take(length as usize)
                         .collect()
                    }
                    "#
                        .as_bytes(),
                )?;
                drop(file);
                let comp_status = std::process::Command::new("rustc")
                    .arg("-C")
                    .arg("target-feature=-crt-static")
                    .arg("--crate-type")
                    .arg("cdylib")
                    .arg(code_name)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()?;
                let mut output: Vec<Vec<Vec<f32>>> = vec![Vec::with_capacity(app.rows.len()); app.rows.get(0).unwrap_or(&vec![]).iter().count()];
                let (mut fs, mut ls, mut vs) = (440.0, 1.0, 1.0);
                if comp_status.success() {
                    unsafe {
                        let lib = libloading::Library::new(lib_name).unwrap();
                        let f0: libloading::Symbol<
                        unsafe extern "C" fn(f32, f32, f32, usize, &[f32]) -> Vec<f32>,
                        > = lib.get(b"f0").unwrap();
                        for row in &app.rows {
                            for (i, el) in row.iter().enumerate() {
                                let temp_args: Vec<Vec<String>> = el
                                    .to_string()
                                    .split_whitespace()
                                    .map(|it| it.split("/").map(String::from).collect::<Vec<String>>())
                                    .collect();
                                        //.map(|(x, _, y)| str::parse::<usize>(x).unwrap() as f32 / str::parse::<usize>(y).unwrap() as f32))
                                let mut vec_args = Vec::with_capacity(3);
                                for param in temp_args {
                                    vec_args.push(str::parse::<usize>(&param[0]).unwrap() as f32 / str::parse::<usize>(&param[1]).unwrap() as f32);
                                }
                                let (f, l, v) = (vec_args[0], vec_args[1], vec_args[2]);
                                (fs, ls, vs) = (fs*f, ls*l, v*vs);
// TODO: fix parsing of rows
                                output[i].push(f0(fs, ls, vs, 44100, &[]));
                            }
                        }
                        //fn_status = (output.iter().flatten().count() / 44100) as f32;
                        let max_len = output.iter().map(|it| it.iter().flatten().count()).max().unwrap_or(0);
                        let mut out_vec = vec![0.0; max_len];
                        for column in output {
                            for (i, el) in column.iter().flatten().enumerate(){
                                out_vec[i] += *el;
                            }
                        }
                        let mut out_vec_iter = out_vec.into_iter();
                    //let mut out_vec = output.into_iter().flatten().chain(core::iter::once(0.0).cycle());
                    std::thread::spawn(move || {let _aud = run_output_device(audio_params, move |data| {
                            for samples in data {
                                    *samples = out_vec_iter.next().unwrap_or(0.0);

                            }
                        }
                        ).unwrap();
                        std::thread::sleep(std::time::Duration::from_secs(100));});
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
