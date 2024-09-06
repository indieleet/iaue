mod help;
mod init_config;

use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::layout::Direction;
use ratatui::{backend::CrosstermBackend, prelude::*, style::Stylize, widgets::*, Terminal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{stdout, Result};
use std::{
    fs,
    io::{Read, Write},
    process::{Command, Stdio},
};
use style::Styled;
use tinyaudio::prelude::*;

#[derive(Serialize, Deserialize)]
struct JsonColor(String);

impl From<JsonColor> for ratatui::style::Color {
    fn from(item: JsonColor) -> Self {
        ratatui::style::Color::from_u32(u32::from_str_radix(&item.0[1..], 16).unwrap())
    }
}

pub struct App<'a> {
    normal_cursor: NormalCursor,
    insert_cursor: InsertCursor,
    visual_cursor: VisualCursor,
    current_times: String,
    current_mode: Mode,
    audio_params: OutputDeviceParameters,
    command_buf: String,
    //file_path: String,
    file_name: String,
    theme: HashMap<String, style::Color>,
    x_bound: u16,
    y_bound: u16,
    cols: Vec<Vec<Vec<Span<'a>>>>,
    yank_buf: Vec<Span<'a>>,
    //constrains: Vec<Constraint>,
    is_help: bool,
    should_leave: bool,
}

impl App<'_> {
    fn count_lines(&mut self) {
        let max_y = self.cols[1..].iter().map(|it| it.len()).max().unwrap_or(0);
        let mut cols = (0..max_y as isize)
            .map(|it| (it - self.normal_cursor.y as isize).abs())
            .map(|it| vec![Span::from(it.to_string()).style(self.theme["fg_dark"])])
            .collect::<Vec<_>>();
        cols[self.normal_cursor.y as usize][0] =
            Span::from(self.normal_cursor.y.to_string()).style(self.theme["orange"]);
        self.cols[0] = cols;
    }
    fn count_bound(&self) -> usize {
        let bound = self.cols[self.normal_cursor.x as usize][self.normal_cursor.y as usize].iter().map(|it| it.content.len() + 1).sum::<usize>();
        if bound < 14 { 14 } else { bound }
    }
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
                    vec![Constraint::Max(temp_bound as u16)] ,
                    vec![Constraint::Max(14); self.app.cols.len() - self.app.normal_cursor.x as usize - 1],
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
                let bounded_el = if (col_i == self.app.normal_cursor.x as usize) && (i == self.app.normal_cursor.y as usize) { &el[..] } else { &el[..line_bound] };
                let constr_c = if col_i != 0 {
                    layout::Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(bounded_el.iter().map(|it| Constraint::Max(it.content.len() as u16 + 1)).collect::<Vec<_>>())
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
                    let c_len = if c.content.is_empty() { 1 } else { c.content.len() as u16 };
                    let printed_cell = if !c.content.is_empty() { &c.clone().patch_style(cell_style) } else { &Span::from(" ").patch_style(cell_style) };
                    buf.set_span(
                        constr_c[ci].x,
                        constr_c[ci].y,
                        printed_cell,
                        c_len,
                    );
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

fn render(app: &mut App) -> Vec<f32> {
    let mut out_vec: Vec<(f32, f32)> = vec![];
    let cur_dir = std::env::current_dir().unwrap();
    let mut lib_name = std::path::PathBuf::new();
    let comp_status = if cur_dir.join("cargolib").exists() {
//cargo run --release --manifest-path=iaue/Cargo.toml
    let full_path_lib = std::env::current_dir()
        .unwrap()
        .join("cargolib");
    lib_name = std::path::Path::new(&full_path_lib)
            .join("target")
            .join("release")
            .join("libcargolib.rlib")
            .canonicalize()
            .unwrap();
    std::process::Command::new("cargo")
        .arg("build")
        .arg("--manifest-path=iaue/Cargo.toml")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        //.inspect_err(|e| app.command_buf = e.to_string())
    .unwrap()
    }
else {
    let full_path_lib = std::env::current_dir()
        .unwrap()
        .join(app.file_name.clone() + ".rs");
    lib_name = std::path::Path::new(&("lib".to_string().to_owned() + &app.file_name + ".so"))
        .canonicalize()
        .unwrap();
    std::process::Command::new("rustc")
        .arg("-C")
        .arg("target-feature=-crt-static")
        .arg("--crate-type")
        .arg("cdylib")
        .arg(&full_path_lib)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        //.inspect_err(|e| app.command_buf = e.to_string())
    .unwrap()
    };
    let err_out = std::str::from_utf8(&comp_status.stderr).unwrap_or("meh");
    app.command_buf = err_out.to_string();
      
    let mut output: Vec<Vec<(f32, f32)>> = vec![Vec::new(); app.cols.len()];
    let mut unique_fn: Vec<String> = Vec::new();
    for col in &app.cols[1..] {
        for el in &col[2..] {
            if !unique_fn.contains(&el[6].to_string()) {
                unique_fn.push(el[6].to_string().clone());
            }
        }
    }
    let mut unique_fx: Vec<String> = Vec::new();
    for col in &app.cols[1..] {
        for el in &col[1][3..] {
            if !unique_fx.contains(&el.to_string()) {
                unique_fx.push(el.to_string().clone());
            }
        }
    }
    let mut fns = std::collections::HashMap::new();
    let mut fxes_fns = std::collections::HashMap::new();
    fn f1(_f: f32, l: f32, _v: f32, t: usize, _p: &[f32]) -> Vec<(f32, f32)> {
        vec![(0.0, 0.0); (l * t as f32) as usize]

    }
    if comp_status.status.success() {
        unsafe {
            let lib = libloading::Library::new(lib_name).unwrap();
            for el in unique_fn {
                let f0 = lib.get::<libloading::Symbol<
                    unsafe extern "C" fn(f32, f32, f32, usize, &[f32]) -> Vec<(f32, f32)>,
                >>(("f".to_string() + &el).as_bytes());
                fns.insert(el.clone(), f0);
            }
            for el in unique_fx {
                let f0 = lib.get::<libloading::Symbol<
                    unsafe extern "C" fn(&[(f32, f32)], usize, &[f32]) -> Vec<(f32, f32)>,
                >>(("fx".to_string() + &el).as_bytes());
                fxes_fns.insert(el.clone(), f0);
            }
            for (i, col) in app.cols[1..].iter().enumerate() {
                let (mut fs, mut ls, mut vs) = (440.0, 1.0, 1.0);
                let mut fxes = Vec::new();
                let mut fx_params = Vec::new();
                for (i_el, el) in col[1..].iter().enumerate() {
                    if i_el == 0 {
                        let el_iter = &mut el.iter();
                        let elems: Vec<_> = el_iter
                            .take(3)
                            .map(|it| str::parse::<f32>(&it.content).unwrap_or(0.0))
                            .collect();
                        (fs, ls, vs) = (elems[0], elems[1], elems[2]);
                        let fx_and_params = el_iter.map(|it| &it.content).collect::<Vec<_>>();
                        for fx in fx_and_params.chunks(2)
                        {
                            fxes.push(fx[0]);
                            fx_params.push(fx[1].split(',').map(|it| it.parse::<f32>().unwrap_or(0.0)).collect::<Vec<f32>>());
                        }
                    } else {
                        let mut pushed_args = Vec::new();
                        let el_iter = &mut el.iter();
                        let elems: Vec<_> = el_iter.take(6).collect();
                        let mut vec_args = Vec::with_capacity(3);
                        for indx in 0..3 {
                            vec_args.push(
                                str::parse::<f32>(&elems[indx * 2].content).unwrap_or(0.0) / str::parse::<f32>(&elems[indx * 2 + 1].content).unwrap_or(0.0),
                            );
                        }
                        let (f, l, v) = (vec_args[0], vec_args[1], vec_args[2]);
                        let (old_f, old_l, old_v) = (fs, ls, vs); 
                        (fs, ls, vs) = (fs * f, ls * l, v * vs);
                        let (mut new_f, mut new_l, mut new_v) = (fs, ls, vs); 
                        let (mut fc, mut lc,  mut vc) = (new_f, new_l, new_v);
                        let pushed_fn = &fns[&el_iter.next().unwrap_or(&Span::from("0")).content.to_string()];
                        let mut note_repeat = 1;
                        let mut slice_param = 1.0;
                        let mut fx_params_slice = Vec::new();
                        for note_param in el_iter.as_slice().chunks(2) {
                            let note_fx = &note_param[0].content;
                            let fx_args = &note_param[1].content.split(',').map(|it| it.to_string()).collect::<Vec<_>>();
                            match note_fx.to_string().as_str() {
                                // 0: Layer new Notes relative to previous
                                // 1: Layer new note Additive
                                // 2: use Constant Frequency for one line
                                // 3: use Constant Duration for one line
                                // 4: use Constant Velocity for one line
                                // 5: Repeat Note
                                // 6: Send Parameters
                                // 7: Override current Frequency with constant value
                                // 8: Override current Duration with constant value
                                // 9: Override current Velocity with constant value
                                // 10: Don't override current values
                                // 11: Slice current note
                                "0" => {
                                    (fc, lc, vc) = ( 
                                    fc * fx_args.first().unwrap_or(&"1".to_string()).split("/").map(|it| it.parse::<f32>().unwrap_or(1.0)).reduce(|x, y| x / y).unwrap_or(1.0), 
                                    lc, 
                                    vc * fx_args.get(1).unwrap_or(&"1".to_string()).split("/").map(|it| it.parse::<f32>().unwrap_or(1.0)).reduce(|x, y| x / y).unwrap_or(1.0));
                                    pushed_args.push((fc, lc, vc));
                                },
                                "1" => {
                                    pushed_args.push((
                                    fs * fx_args.first().unwrap_or(&"1".to_string()).split("/").map(|it| it.parse::<f32>().unwrap_or(1.0)).reduce(|x, y| x / y).unwrap_or(1.0), 
                                    ls, 
                                    vs * fx_args.get(1).unwrap_or(&"1".to_string()).split("/").map(|it| it.parse::<f32>().unwrap_or(1.0)).reduce(|x, y| x / y).unwrap_or(1.0)))
                                },
                                "2" => { fs = 44100.0 / fx_args.first().unwrap_or(&fs.to_string()).parse::<f32>().unwrap_or(fs);},

                                "3" => { ls = fx_args.first().unwrap_or(&ls.to_string()).parse::<f32>().unwrap_or(ls);},

                                "4" => { vs = fx_args.first().unwrap_or(&vs.to_string()).parse::<f32>().unwrap_or(vs);}

                                "5" => { note_repeat *= fx_args.first().unwrap_or(&"1".to_string()).parse::<usize>().unwrap_or(1); }

                                "6" => { fx_params_slice.extend(fx_args.iter().map(|it| it.parse::<f32>().unwrap_or(0.0))); },

                                "7" => { new_f = 44100.0 / fx_args.first().unwrap_or(&fs.to_string()).parse::<f32>().unwrap_or(fs);
                                    fs = new_f;
                                },

                                "8" => { new_l = fx_args.first().unwrap_or(&ls.to_string()).parse::<f32>().unwrap_or(ls);
                                    ls = new_l;
                                },

                                "9" => { new_v = fx_args.first().unwrap_or(&vs.to_string()).parse::<f32>().unwrap_or(vs);
                                    vs = new_v;
                                },

                                "10" => { (new_f, new_l, new_v) = (old_f, old_l, old_v); },
                                "11" => { 
                                    note_repeat *= fx_args.first().unwrap_or(&"1".to_string()).parse::<usize>().unwrap_or(1); 
                                    slice_param = if note_repeat == 0 { 1.0 } else { note_repeat as f32 };
                                },
                                "12" => {
                                    let bound = fx_args.first().unwrap_or(&"1".to_string()).parse::<usize>().unwrap_or(1);
                                    let mut rand_iter = core::iter::repeat_with(|| fastrand::usize(1..bound));
                                    fs *= rand_iter.next().unwrap_or(1) as f32 / rand_iter.next().unwrap_or(1) as f32;
                                },
                                "13" => {
                                    let bound = fx_args.first().unwrap_or(&"1".to_string()).parse::<usize>().unwrap_or(1);
                                    let mut rand_iter = core::iter::repeat_with(|| fastrand::usize(1..bound));
                                    ls *= rand_iter.next().unwrap_or(1) as f32 / rand_iter.next().unwrap_or(1) as f32;
                                },
                                "14" => {
                                    let bound = fx_args.first().unwrap_or(&"1".to_string()).parse::<usize>().unwrap_or(1);
                                    let mut rand_iter = core::iter::repeat_with(|| fastrand::usize(1..bound));
                                    vs *= rand_iter.next().unwrap_or(1) as f32 / rand_iter.next().unwrap_or(1) as f32;
                                },
                                "15" => {
                                    let bound = fx_args.first().unwrap_or(&"1".to_string()).parse::<usize>().unwrap_or(1);
                                    let mut rand_iter = core::iter::repeat_with(|| fastrand::usize(1..bound));
                                    fs *= rand_iter.next().unwrap_or(1) as f32 / rand_iter.next().unwrap_or(1) as f32;
                                    ls *= rand_iter.next().unwrap_or(1) as f32 / rand_iter.next().unwrap_or(1) as f32;
                                    vs *= rand_iter.next().unwrap_or(1) as f32 / rand_iter.next().unwrap_or(1) as f32;
                                },
                                "16" => {
                                    let bound = fx_args.first().unwrap_or(&"1".to_string()).parse::<usize>().unwrap_or(1);
                                    let mut rand_iter = core::iter::repeat_with(|| fastrand::usize(1..bound));
                                    fs *= rand_iter.next().unwrap_or(1) as f32 / rand_iter.next().unwrap_or(1) as f32;
                                    new_f = fs;
                                },
                                "17" => {
                                    let bound = fx_args.first().unwrap_or(&"1".to_string()).parse::<usize>().unwrap_or(1);
                                    let mut rand_iter = core::iter::repeat_with(|| fastrand::usize(1..bound));
                                    ls *= rand_iter.next().unwrap_or(1) as f32 / rand_iter.next().unwrap_or(1) as f32;
                                    new_l = ls;
                                },
                                "18" => {
                                    let bound = fx_args.first().unwrap_or(&"1".to_string()).parse::<usize>().unwrap_or(1);
                                    let mut rand_iter = core::iter::repeat_with(|| fastrand::usize(1..bound));
                                    vs *= rand_iter.next().unwrap_or(1) as f32 / rand_iter.next().unwrap_or(1) as f32;
                                    new_v = vs;
                                },
                                "19" => {
                                    let bound = fx_args.first().unwrap_or(&"1".to_string()).parse::<usize>().unwrap_or(1);
                                    let mut rand_iter = core::iter::repeat_with(|| fastrand::usize(1..bound));
                                    fs *= rand_iter.next().unwrap_or(1) as f32 / rand_iter.next().unwrap_or(1) as f32;
                                    ls *= rand_iter.next().unwrap_or(1) as f32 / rand_iter.next().unwrap_or(1) as f32;
                                    vs *= rand_iter.next().unwrap_or(1) as f32 / rand_iter.next().unwrap_or(1) as f32;
                                    new_f = fs;
                                    new_l = ls;
                                    new_v = vs;
                                },

                                _ => {}
                            }
                        }
                        pushed_args.push((fs, ls, vs));
                        (fs, ls, vs) = (new_f, new_l, new_v);
                        let mut temp_vec: Vec<Vec<(f32, f32)>> = Vec::new();
                        for (fs, ls, vs) in pushed_args {
                            match pushed_fn {
                                Ok(val) => {
                                    let out_tuple = val(fs, ls / slice_param, vs, 44100, fx_params_slice.as_slice());
                                    temp_vec.push(out_tuple);
                                }
                                Err(_) => {
                                    let out_tuple = f1(fs, ls / slice_param, vs, 44100, fx_params_slice.as_slice());
                                    temp_vec.push(out_tuple);
                                }
                            }
                        }
                        let len_of_note = temp_vec[0].len();
                        let mut sum_vec = vec![(0.0, 0.0); len_of_note];
                            for el in temp_vec {
                                for (i, sample) in el.iter().enumerate() {
                                    sum_vec[i].0 += sample.0;
                                    sum_vec[i].1 += sample.1;
                                }
                            }
                        let out_note = sum_vec.into_iter().cycle().take(len_of_note * note_repeat);
                        output[i].extend(out_note);
                    }
                }
                for (idx, fx) in fxes.iter().enumerate() {
                    let cur_fx = &fxes_fns[&fx.to_string()];
                    match cur_fx {
                        Ok(val) => {
                            let out_tuple = val(output[i].as_slice(), 44100, fx_params[idx].as_slice());
                            output[i] = out_tuple;
                        }
                        Err(_) => {
                        }
                    }
                }
            }
            let max_len = output
                .iter()
                .map(|it| it.len())
                .max()
                .unwrap_or(0);
            out_vec = vec![(0.0, 0.0); max_len];
            for column in output {
                for (i, el) in column.iter().enumerate() {
                    out_vec[i].0 += el.0;
                    out_vec[i].1 += el.1;
                }
            }
            //let mut out_vec_iter = out_vec.into_iter();
            //fn_status = format!("{}, {}, {}, {}", ft, lt, vt, (max_len / 44100) as f32);
        }
    }
    out_vec.iter().map(|&(it, y)| if it == f32::INFINITY { (f32::MAX, y) }
        else if it == f32::NEG_INFINITY { (f32::MIN, y) }
        else if it.is_nan() { (0.0, y) }
        else { (it, y) }
    )
        .map(|(x, it)| if it == f32::INFINITY { (x, f32::MAX) }
            else if it == f32::NEG_INFINITY { (x, f32::MIN) }
            else if it.is_nan() { (x, 0.0) }
            else { (x, it) }
        )
    .flat_map(|(x, y)| [x, y])
    .collect::<Vec<_>>()
}
fn render_and_save_file(app: &mut App, file_name: String) {
    use std::fs::File;
    use std::path::absolute;
    use std::path::Path;
    let out_file = render(app);
    let new_file_name = if file_name.is_empty() { app.file_name.clone() + ".wav" } else { file_name };
    let full_path = absolute(Path::new(&new_file_name)).unwrap().to_path_buf();
    let mut file = File::create(full_path).unwrap();
    let header = wav_io::new_mono_header();
    let _ = wav_io::write_to_file(&mut file, &header, &out_file);
    app.command_buf = format!("Saved to {}", new_file_name); //TODO:
}
fn open_file(app: &mut App, file_name: String) {
    use std::fs::File;
    use std::io::Read;
    //use std::env::current_dir;
    use std::path::Path;
    let new_file = match Path::new(&file_name).canonicalize() {
        Ok(value) => value,
        Err(_) => {
            app.command_buf = format!("Can't find file {}.", file_name);
            return;
        }
    };
    let full_path = match new_file.is_file() {
        true => new_file,
        false => new_file.join(&app.file_name),
    };
    let _ = std::env::set_current_dir(full_path.parent().unwrap());
    app.file_name = full_path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .split(".")
        .next()
        .unwrap()
        .to_string();
    let mut file = File::open(full_path);
    match file {
        Ok(ref mut val) => {
            let mut data = String::new();
            val.read_to_string(&mut data).unwrap();
            app.cols = serde_json::from_str::<Vec<Vec<Vec<String>>>>(&data)
                .unwrap()
                .into_iter()
                .map(|col| {
                    col.into_iter()
                        .map(|el| el.into_iter().map(Span::from).collect::<Vec<_>>())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            app.normal_cursor.x = 0;
            app.normal_cursor.y = 0;
            app.visual_cursor.x = 0;
            app.visual_cursor.y = 0;
            app.insert_cursor.x = 0;
            app.command_buf.clear();
        }
        Err(_) => {
            app.command_buf = format!("Can't find file {}.", file_name);
        }
    }
}

fn save_file(app: &mut App, file_name: String) {
    use std::fs::File;
    use std::io::{BufWriter, Write};
    use std::path::absolute;
    use std::path::Path;
    let new_file = absolute(Path::new(&file_name)).unwrap().to_path_buf();
    let full_path = match new_file.is_file() {
        true => new_file.to_path_buf(),
        false => new_file.join(&app.file_name),
    };
    let _ = std::env::set_current_dir(full_path.parent().unwrap());
    app.file_name = full_path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .split(".")
        .next()
        .unwrap()
        .to_string();
    let full_path = std::path::Path::new(&std::env::current_dir().unwrap().to_str().unwrap_or("/"))
        .join(&file_name);
    let file_cloned = &app
        .cols
        .clone()
        .into_iter()
        .map(|col| {
            col.into_iter()
                .map(|el| el.into_iter().map(|c| c.content).collect::<Vec<_>>())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let file = File::create(full_path).unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &file_cloned).unwrap();
    writer.flush().unwrap();
}

fn exec_command(app: &mut App) {
    let splitted_commands = app.command_buf[1..]
        .split_whitespace()
        .collect::<Vec<&str>>();
    match *splitted_commands.first().unwrap() {
        "q" => {
            app.should_leave = true;
            app.command_buf.clear();
        }
        "wq" => {
            app.command_buf = "Not yet implemented.".to_string(); //TODO:
        }
        "cd" => {
            if std::env::set_current_dir(splitted_commands[1..].join(" ")).is_err() {
                app.command_buf = format!("Can't find dir {}.", splitted_commands[1..].join(" "));
            }
            //TODO: only possible if dir exist
        }
        "pwd" => {
            app.command_buf = std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap_or("/")
                .to_string();
        }
        "cf" => {
            app.file_name = splitted_commands[1..]
                .join(" ")
                .split('.')
                .next()
                .unwrap()
                .to_string();
        }
        "o" | "open" => {
            open_file(app, splitted_commands[1..].join(" "));
        }
        "render" => {
            render_and_save_file(app, splitted_commands[1..].join(" "));
        }
        "s" | "save" => {
            save_file(app, splitted_commands[1..].join(" "));
            app.command_buf.clear();
        }
        "e" | "edit" => {
            app.command_buf = "Not yet implemented.".to_string();
        } //TODO:
        command => app.command_buf = format!("Command '{}' not found.", command),
    }
    app.current_mode = Mode::Normal;
}

fn start_app(working_file: &str) -> Result<()> {
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
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    let mut app = App {
        normal_cursor: NormalCursor::default(),
        visual_cursor: VisualCursor::default(),
        insert_cursor: InsertCursor::default(),
        current_mode: Mode::Normal,
        audio_params: OutputDeviceParameters {
            channels_count: 1,
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
        is_help: false,
        should_leave: false,
    };
    let editor = std::env::var("EDITOR").unwrap_or("nvim".to_string());
    let mut rand_iter = core::iter::repeat_with(|| fastrand::u8(0..9));
    let fn_status = String::new();
    let full_path_lib =
        std::path::Path::new(&std::env::current_dir().unwrap().to_str().unwrap_or("/"))
            .join(app.file_name.clone() + ".rs");
    let full_path_file =
        std::path::Path::new(&std::env::current_dir().unwrap().to_str().unwrap_or("/"))
            .join(app.file_name.clone() + ".tr");
    app.y_bound = app.cols[app.normal_cursor.x as usize].len() as u16;
    app.count_lines();
    loop {
        //let table_cols = app.cols.to_owned();
        //table_rows[app.normal_cursor.y as usize][app.normal_cursor.x as usize] = cur_cell_text;
        //let y_bound = core::iter::repeat_with(|| &app.rows.iter().next().unwrap_or(&Vec::<Span>::new()).get(app.normal_cursor.x as usize)).count();
        let y_bound: u16 = app.cols[app.normal_cursor.x as usize].len() as u16;
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
            let lines_count = help::TEXT.lines().count() as u16 + 2;
            if app.is_help {
                f.render_widget(
                    Paragraph::new(help::TEXT).block(
                        Block::bordered()
                            .title_alignment(Alignment::Center)
                            .title("Help"),
                    ),
                    layout::Rect {
                        x: f.area().width / 2 - 20,
                        y: f.area().height / 2 - lines_count / 2,
                        width: 40,
                        height: lines_count,
                    },
                );
            }
        })?;
        let match_event = event::read()?;
        match match_event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => match app.current_mode {
                Mode::Visual | Mode::Normal | Mode::Insert => {
                    app.should_leave = true;
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
                        if (app.normal_cursor.y as usize) < app.cols[final_cursor as usize].len() { app.normal_cursor.x = final_cursor };
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
                        if new_cursor_normal >= 0 {
                            app.insert_cursor.x = if new_cursor_insert >= 0 {
                                new_cursor_insert as u16
                            } else {
                                (insert_bound + new_cursor_insert % insert_bound) as u16
                            };
                            app.normal_cursor.x = new_cursor_normal as u16;
                        } else {
                            app.normal_cursor.x = 0;
                            app.insert_cursor.x = 0;
                        }
                    }
                    Mode::Command => {
                        app.command_buf.push('h');
                    }
                }
                //app.normal_cursor.x = app.normal_cursor.x.saturating_sub(count);
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
                        app.normal_cursor.y = app.normal_cursor.y.saturating_sub(count);
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
                       // let insert_bound = if app.normal_cursor.y == 0 {
                       //     1
                       // } else if app.normal_cursor.y == 1 {
                       //     3
                       // } else {
                       //     7
                       // };
                        let new_cursor_insert = app.insert_cursor.x + count as u16;
                        let new_cursor_normal = app
                            .normal_cursor
                            .x
                            .saturating_add(new_cursor_insert / insert_bound);
                        if new_cursor_normal < app.cols.len() as u16 {
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
                }
                Mode::Visual => {
                    if y_bound - 2 < app.normal_cursor.y {
                        app.normal_cursor.y = app.normal_cursor.y.saturating_sub(1);
                    }
                    app.cols[app.normal_cursor.x as usize].remove(app.normal_cursor.y as usize);
                    app.count_lines();
                }
                Mode::Command => {
                    app.command_buf.push('d');
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char('y'),
                ..
            }) => match app.current_mode {
                Mode::Insert | Mode::Normal | Mode::Visual => {
                    app.yank_buf = app.cols[app.normal_cursor.x as usize]
                        [app.normal_cursor.y as usize]
                        .clone();
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
                                .insert(app.normal_cursor.y as usize + 1, app.yank_buf.clone());
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
                                .insert(app.normal_cursor.y as usize, app.yank_buf.clone());
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
               Mode::Normal | Mode::Visual => {
                    app.cols.remove(app.normal_cursor.x as usize);
                    app.normal_cursor.x = if app.cols.len() - 1 < app.normal_cursor.x as usize {
                        app.cols.len() as u16 - 1
                    } else {
                        app.normal_cursor.x
                    };
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
                let _ = terminal.clear();
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('s'),
                ..
            }) => {
                use std::fs::File;
                use std::io::{BufWriter, Write};
                let file_cloned = &app
                    .cols
                    .clone()
                    .into_iter()
                    .map(|col| {
                        col.into_iter()
                            .map(|el| el.into_iter().map(|c| c.content).collect::<Vec<_>>())
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                let file = File::create(&full_path_file).unwrap();
                let mut writer = BufWriter::new(file);
                serde_json::to_writer(&mut writer, &file_cloned).unwrap();
                writer.flush().unwrap();
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('o'),
                ..
            }) => {
                use std::fs::File;
                use std::io::Read;
                let mut file = File::open(&full_path_file).unwrap();
                let mut data = String::new();
                file.read_to_string(&mut data).unwrap();
                app.cols = serde_json::from_str::<Vec<Vec<Vec<String>>>>(&data)
                    .unwrap()
                    .into_iter()
                    .map(|col| {
                        col.into_iter()
                            .map(|el| el.into_iter().map(Span::from).collect::<Vec<_>>())
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                app.normal_cursor.x = 0;
                app.normal_cursor.y = 0;
                app.visual_cursor.x = 0;
                app.visual_cursor.y = 0;
                app.insert_cursor.x = 0;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => match app.current_mode {
                Mode::Command => exec_command(&mut app),
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
                let out_vec = render(&mut app);
                let max_len = out_vec.len();
                let mut out_vec_iter = out_vec.into_iter();
                        std::thread::spawn(move || {
                            let _aud = run_output_device(app.audio_params, move |data| {
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
        if app.should_leave {
            break;
        };
    }
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
fn main() {
    let cli = Cli::parse();

    use std::path::Path;
    let mut working_file: &str = "project";
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
