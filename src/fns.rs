use crate::app::{App, Mode, Page, Instrument};
use crate::synths::Synths;
use crate::help;
use ratatui::prelude::*;
use std::io::{stdout, Result};
use std::process::Stdio;
use tinyaudio::prelude::*;

#[cfg(not(feature = "sdl"))]
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use std::process::Command;
fn minmax_x(app: &App) -> (u16, u16) {
    if app.normal_cursor.x < app.visual_cursor.x {
        (app.normal_cursor.x, app.visual_cursor.x)
    } else {
        (app.visual_cursor.x, app.normal_cursor.x)
    }
}

fn minmax_y(app: &App) -> (u16, u16) {
    if app.normal_cursor.y < app.visual_cursor.y {
        (app.normal_cursor.y, app.visual_cursor.y)
    } else {
        (app.visual_cursor.y, app.normal_cursor.y)
    }
}

type Sample = (f32, f32);
fn apply_fn(app: &mut App, id: u8, f: f32, l: f32, v: f32, t: usize, p: &[f32]) -> Vec<Sample> {
    match app.instrs[id as usize] {
        Some(instr) => {
            match instr.synth {
                Synths::Rust { name, num, level, fn_symbol } => {
                    if let Some(inner_fn_symbol) = fn_symbol {
                        unsafe {
                            inner_fn_symbol(f, l, v, t, p)
                        }
                    }
                    else {
                        vec![(0.0, 0.0); (l * t as f32) as usize]
                    }

                },
                Synths::Macro { name, level } => { vec![(0.0, 0.0); (l * t as f32) as usize] },
            }
        },
        None => { 
            vec![(0.0, 0.0); (l * t as f32) as usize]
        }

    }
}

fn build_lib<'a>(app: &'a mut App<'a>) {
    let cur_dir = std::env::current_dir().unwrap();
    let lib_name;
    let mut unique_fn: Vec<String> = Vec::new();
    let mut unique_fx: Vec<String> = Vec::new();
    //let mut fns = std::collections::HashMap::new();
    let mut fxes_fns = std::collections::HashMap::new();
    let comp_status = if cur_dir.join("cargolib/").exists() {
        //cargo run --release --manifest-path=iaue/Cargo.toml
        let full_path_lib = std::env::current_dir().unwrap().join("cargolib/");
        let out = std::process::Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--manifest-path=cargolib/Cargo.toml")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            //.inspect_err(|e| app.command_buf = e.to_string())
            .unwrap();
        lib_name = std::path::Path::new(&full_path_lib)
            .join("target/")
            .join("release/")
            .join("libcargolib.so")
            .canonicalize()
            .unwrap();
        out
    } else {
        let full_path_lib = std::env::current_dir()
            .unwrap()
            .join(app.file_name.clone() + ".rs");
        let out = std::process::Command::new("rustc")
            .arg("-C")
            .arg("target-feature=-crt-static")
            .arg("--crate-type")
            .arg("cdylib")
            .arg(&full_path_lib)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            //.inspect_err(|e| app.command_buf = e.to_string())
            .unwrap();
        lib_name = std::path::Path::new(&("lib".to_string().to_owned() + &app.file_name + ".so"))
            .canonicalize()
            .unwrap();
        out
    };
    let err_out = std::str::from_utf8(&comp_status.stderr).unwrap_or("meh");
    app.command_buf = err_out.to_string();
    if comp_status.status.success() {
        unsafe {
            app.lib = libloading::Library::new(lib_name).unwrap();
            for i in 0..app.instrs.len() {
                match app.instrs[i] {
                    Some(ref mut instr) => match instr.synth {
                        Synths::Rust { name, num, level, ref mut fn_symbol } => {
                            core::mem::replace(fn_symbol, Some(app.lib.get::<libloading::Symbol<
                                unsafe extern "C" fn(f32, f32, f32, usize, &[f32]) -> Vec<(f32, f32)>,
                                >>(("f".to_string() + &num.to_string()).as_bytes()).unwrap()));}
                        _ => {}
                    },
                    _ => {}
                }
            }
            for el in unique_fx {
                let f0 = app.lib.get::<libloading::Symbol<
                    unsafe extern "C" fn(
                        &[(f32, f32)],
                        usize,
                        &[f32],
                        &[Vec<(f32, f32)>],
                    ) -> Vec<(f32, f32)>,
                    >>(("fx".to_string() + &el).as_bytes());
                fxes_fns.insert(el.clone(), f0);
            }
        }
    }
}

pub fn render(app: &mut App) -> Vec<f32> {
    let mut out_vec: Vec<(f32, f32)> = vec![];
    let mut output: Vec<Vec<(f32, f32)>> = vec![Vec::new(); app.cols.len()];
    let mut unique_fn: Vec<String> = Vec::new();
    //for col in &app.cols[1..] {
    //    for el in &col[2..] {
    //        if !unique_fn.contains(&el[6].to_string()) {
    //            unique_fn.push(el[6].to_string().clone());
    //        }
    //    }
    //}
    //
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
    {  
        {
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
                        for fx in fx_and_params.chunks(2) {
                            fxes.push(fx[0]);
                            fx_params.push(
                                fx[1]
                                    .split(',')
                                    .map(|it| it.parse::<f32>().unwrap_or(0.0))
                                    .collect::<Vec<f32>>(),
                            );
                        }
                    } else {
                        let mut pushed_args = Vec::new();
                        let el_iter = &mut el.iter();
                        let elems: Vec<_> = el_iter.take(6).collect();
                        let mut vec_args = Vec::with_capacity(3);
                        for indx in 0..3 {
                            vec_args.push(
                                str::parse::<f32>(&elems[indx * 2].content).unwrap_or(0.0)
                                    / str::parse::<f32>(&elems[indx * 2 + 1].content)
                                        .unwrap_or(0.0),
                            );
                        }
                        let (f, l, v) = (vec_args[0], vec_args[1], vec_args[2]);
                        let (old_f, old_l, old_v) = (fs, ls, vs);
                        (fs, ls, vs) = (fs * f, ls * l, v * vs);
                        let (mut new_f, mut new_l, mut new_v) = (fs, ls, vs);
                        let (mut fc, mut lc, mut vc) = (new_f, new_l, new_v);
                        let pushed_fn = &fns[&el_iter
                            .next()
                            .unwrap_or(&Span::from("0"))
                            .content
                            .to_string()];
                        let mut note_repeat = 1;
                        let mut slice_param = 1.0;
                        let mut fx_params_slice = Vec::new();
                        for note_param in el_iter.as_slice().chunks(2) {
                            let note_fx = &note_param[0].content;
                            let fx_args = &note_param[1]
                                .content
                                .split(',')
                                .map(|it| it.to_string())
                                .collect::<Vec<_>>();
                            match note_fx.to_string().as_str() {
                                "0" => {
                                    (fc, lc, vc) = (
                                        fc * fx_args
                                            .first()
                                            .unwrap_or(&"1".to_string())
                                            .split("/")
                                            .map(|it| it.parse::<f32>().unwrap_or(1.0))
                                            .reduce(|x, y| x / y)
                                            .unwrap_or(1.0),
                                        lc,
                                        vc * fx_args
                                            .get(1)
                                            .unwrap_or(&"1".to_string())
                                            .split("/")
                                            .map(|it| it.parse::<f32>().unwrap_or(1.0))
                                            .reduce(|x, y| x / y)
                                            .unwrap_or(1.0),
                                    );
                                    pushed_args.push((fc, lc, vc));
                                }
                                "1" => pushed_args.push((
                                    fs * fx_args
                                        .first()
                                        .unwrap_or(&"1".to_string())
                                        .split("/")
                                        .map(|it| it.parse::<f32>().unwrap_or(1.0))
                                        .reduce(|x, y| x / y)
                                        .unwrap_or(1.0),
                                    ls,
                                    vs * fx_args
                                        .get(1)
                                        .unwrap_or(&"1".to_string())
                                        .split("/")
                                        .map(|it| it.parse::<f32>().unwrap_or(1.0))
                                        .reduce(|x, y| x / y)
                                        .unwrap_or(1.0),
                                )),
                                "2" => {
                                    note_repeat *= fx_args
                                        .first()
                                        .unwrap_or(&"1".to_string())
                                        .parse::<usize>()
                                        .unwrap_or(1);
                                }

                                "3" => {
                                    fx_params_slice.extend(
                                        fx_args.iter().map(|it| it.parse::<f32>().unwrap_or(0.0)),
                                    );
                                }

                                "4" => {
                                    new_f = fx_args
                                        .first()
                                        .unwrap_or(&fs.to_string())
                                        .parse::<f32>()
                                        .unwrap_or(fs);
                                    fs = new_f;
                                }

                                "5" => {
                                    new_l = fx_args
                                        .first()
                                        .unwrap_or(&ls.to_string())
                                        .parse::<f32>()
                                        .unwrap_or(ls);
                                    ls = new_l;
                                }

                                "6" => {
                                    new_v = fx_args
                                        .first()
                                        .unwrap_or(&vs.to_string())
                                        .parse::<f32>()
                                        .unwrap_or(vs);
                                    vs = new_v;
                                }

                                "7" => {
                                    (new_f, new_l, new_v) = (old_f, old_l, old_v);
                                }
                                "8" => {
                                    note_repeat *= fx_args
                                        .first()
                                        .unwrap_or(&"1".to_string())
                                        .parse::<usize>()
                                        .unwrap_or(1);
                                    slice_param = if note_repeat == 0 {
                                        1.0
                                    } else {
                                        note_repeat as f32
                                    };
                                }
                                "9" => {
                                    let mut bound = fx_args
                                        .first()
                                        .unwrap_or(&"1".to_string())
                                        .parse::<usize>()
                                        .unwrap_or(1);
                                    if bound == 0 {
                                        bound = 1
                                    };
                                    let down_bound = fx_args
                                        .get(1)
                                        .unwrap_or(&"20".to_string())
                                        .parse::<f32>()
                                        .unwrap_or(20.0);
                                    let up_bound = fx_args
                                        .get(2)
                                        .unwrap_or(&"20000".to_string())
                                        .parse::<f32>()
                                        .unwrap_or(20_000.0);
                                    let mut rand_iter =
                                        core::iter::repeat_with(|| fastrand::usize(1..=bound));
                                    fs *= rand_iter.next().unwrap_or(1) as f32
                                        / rand_iter.next().unwrap_or(1) as f32;
                                    let mut it = 0;
                                    while (fs < down_bound) && (it < 8) {
                                        fs *= 2.0;
                                        it += 1;
                                    }
                                    while (fs > up_bound) && (it < 8) {
                                        fs /= 2.0;
                                        it += 1;
                                    }
                                    new_f = fs;
                                }
                                "10" => {
                                    let mut bound = fx_args
                                        .first()
                                        .unwrap_or(&"1".to_string())
                                        .parse::<usize>()
                                        .unwrap_or(1);
                                    if bound == 0 {
                                        bound = 1
                                    };
                                    let down_bound = fx_args
                                        .get(1)
                                        .unwrap_or(&"0.01".to_string())
                                        .parse::<f32>()
                                        .unwrap_or(0.01);
                                    let up_bound = fx_args
                                        .get(2)
                                        .unwrap_or(&"10".to_string())
                                        .parse::<f32>()
                                        .unwrap_or(10.0);
                                    let mut rand_iter =
                                        core::iter::repeat_with(|| fastrand::usize(1..=bound));
                                    ls *= rand_iter.next().unwrap_or(1) as f32
                                        / rand_iter.next().unwrap_or(1) as f32;
                                    let mut it = 0;
                                    while (ls < down_bound) && (it < 8) {
                                        ls *= 2.0;
                                        it += 1;
                                    }
                                    while (ls > up_bound) && (it < 8) {
                                        ls /= 2.0;
                                        it += 1;
                                    }
                                    new_l = ls;
                                }
                                "11" => {
                                    let mut bound = fx_args
                                        .first()
                                        .unwrap_or(&"1".to_string())
                                        .parse::<usize>()
                                        .unwrap_or(1);
                                    if bound == 0 {
                                        bound = 1
                                    };
                                    let down_bound = fx_args
                                        .get(1)
                                        .unwrap_or(&"0.1".to_string())
                                        .parse::<f32>()
                                        .unwrap_or(0.1);
                                    let up_bound = fx_args
                                        .get(2)
                                        .unwrap_or(&"1".to_string())
                                        .parse::<f32>()
                                        .unwrap_or(1.0);
                                    let mut rand_iter =
                                        core::iter::repeat_with(|| fastrand::usize(1..=bound));
                                    vs *= rand_iter.next().unwrap_or(1) as f32
                                        / rand_iter.next().unwrap_or(1) as f32;
                                    let mut it = 0;
                                    while (vs < down_bound) && (it < 8) {
                                        vs *= 2.0;
                                        it += 1;
                                    }
                                    while (vs > up_bound) && (it < 8) {
                                        vs /= 2.0;
                                        it += 1;
                                    }
                                    new_v = vs;
                                }
                                "12" => {
                                    let mut bound = fx_args
                                        .first()
                                        .unwrap_or(&"1".to_string())
                                        .parse::<usize>()
                                        .unwrap_or(1);
                                    if bound == 0 {
                                        bound = 1
                                    };
                                    let mut rand_iter =
                                        core::iter::repeat_with(|| fastrand::usize(1..=bound));
                                    fs *= rand_iter.next().unwrap_or(1) as f32
                                        / rand_iter.next().unwrap_or(1) as f32;
                                    ls *= rand_iter.next().unwrap_or(1) as f32
                                        / rand_iter.next().unwrap_or(1) as f32;
                                    vs *= rand_iter.next().unwrap_or(1) as f32
                                        / rand_iter.next().unwrap_or(1) as f32;
                                    new_f = fs;
                                    new_l = ls;
                                    new_v = vs;
                                }

                                _ => {}
                            }
                        }
                        pushed_args.push((fs, ls, vs));
                        (fs, ls, vs) = (new_f, new_l, new_v);
                        let mut temp_vec: Vec<Vec<(f32, f32)>> = Vec::new();
                        for (fs, ls, vs) in pushed_args {
                            match pushed_fn {
                                Ok(val) => {
                                    let out_tuple = apply_fn(
                                        &mut app,
                                        el_iter
                            .next()
                            .unwrap_or(&Span::from("0"))
                            .content
                            .parse::<u8>()
                            .unwrap_or(0),
                                        fs,
                                        ls / slice_param,
                                        vs,
                                        44100,
                                        fx_params_slice.as_slice(),
                                    );
                                    temp_vec.push(out_tuple);
                                }
                                Err(_) => {
                                    let out_tuple = f1(
                                        fs,
                                        ls / slice_param,
                                        vs,
                                        44100,
                                        fx_params_slice.as_slice(),
                                    );
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
                            let out_tuple = val(
                                output[i].as_slice(),
                                44100,
                                fx_params[idx].as_slice(),
                                output.as_slice(),
                            );
                            output[i] = out_tuple;
                        }
                        Err(_) => {}
                    }
                }
            }
            let max_len = output.iter().map(|it| it.len()).max().unwrap_or(0);
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
    out_vec
        .iter()
        .map(|&(it, y)| {
            if it == f32::INFINITY {
                (f32::MAX, y)
            } else if it == f32::NEG_INFINITY {
                (f32::MIN, y)
            } else if it.is_nan() {
                (0.0, y)
            } else {
                (it, y)
            }
        })
        .map(|(x, it)| {
            if it == f32::INFINITY {
                (x, f32::MAX)
            } else if it == f32::NEG_INFINITY {
                (x, f32::MIN)
            } else if it.is_nan() {
                (x, 0.0)
            } else {
                (x, it)
            }
        })
        .flat_map(|(x, y)| [x, y])
        .collect::<Vec<_>>()
}

pub fn render_and_save_file(app: &mut App, file_name: String) {
    use std::fs::File;
    use std::path::absolute;
    use std::path::Path;
    let out_file = render(app);
    let new_file_name = if file_name.is_empty() {
        app.file_name.clone() + ".wav"
    } else {
        file_name
    };
    let full_path = absolute(Path::new(&new_file_name)).unwrap().to_path_buf();
    let mut file = File::create(full_path).unwrap();
    let header = wav_io::new_stereo_header();
    let _ = wav_io::write_to_file(&mut file, &header, &out_file);
    app.command_buf = format!("Saved to {}", new_file_name); //TODO:
}

pub fn open_file(app: &mut App, mut file_name: String) {
    use std::fs::File;
    use std::io::Read;
    //use std::env::current_dir;
    use std::path::Path;
    if file_name.is_empty() {
        file_name = app.file_name.clone();
    };
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
    app.file_name = full_path.file_name().unwrap().to_str().unwrap().to_string();
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
            app.normal_cursor.x = 1;
            app.normal_cursor.y = 1;
            app.visual_cursor.x = 1;
            app.visual_cursor.y = 1;
            app.insert_cursor.x = 0;
            app.command_buf.clear();
        }
        Err(_) => {
            app.command_buf = format!("Can't find file {}.", file_name);
        }
    }
}

pub fn save_file(app: &mut App, mut file_name: String) {
    use std::fs::File;
    use std::io::{BufWriter, Write};
    use std::path::absolute;
    use std::path::Path;
    if file_name.is_empty() {
        file_name = app.file_name.clone();
    };
    let new_file = absolute(Path::new(&file_name)).unwrap().to_path_buf();
    let full_path = match new_file.is_file() {
        true => new_file.to_path_buf(),
        false => new_file.join(&app.file_name),
    };
    let _ = std::env::set_current_dir(full_path.parent().unwrap());
    app.file_name = full_path.file_name().unwrap().to_str().unwrap().to_string();
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
    let mut buf_writer = BufWriter::new(file);
    serde_json::to_writer(&mut buf_writer, &file_cloned).unwrap();
    buf_writer.flush().unwrap();
}

#[inline]
pub fn rename_track(app: &mut App, name: String) {
    app.cols[app.normal_cursor.x as usize][0][0] = Span::from(name);
}

pub fn exec_command(app: &mut App) {
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
        "rename" => {
            rename_track(app, splitted_commands[1..].join(" "));
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
pub fn false_quit(app: &mut App) {
    match app.current_mode {
        Mode::Visual | Mode::Normal | Mode::Insert => {
            app.is_help = false;
            app.command_buf = "to quit type :q and hit enter".to_string();
        }
        Mode::Command => {
            app.command_buf.push('q');
        }
    }
}

pub fn num_to_buf(app: &mut App, matched_code: char) {
    match app.current_mode {
        Mode::Normal | Mode::Visual => {
            app.current_times.push(matched_code);
        }
        Mode::Insert => {
            let temp_span = app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                [app.insert_cursor.x as usize]
                .clone();
            app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                [app.insert_cursor.x as usize]
                .content = (temp_span.content.to_string() + &matched_code.to_string()).into();
        }
        Mode::Command => {
            app.command_buf.push(matched_code);
        }
    }
}

pub fn dot(app: &mut App) {
    match app.current_mode {
        Mode::Normal | Mode::Visual => {}
        Mode::Insert => {
            let temp_span = app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                [app.insert_cursor.x as usize]
                .clone();
            app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                [app.insert_cursor.x as usize]
                .content = (temp_span.content.to_string() + ".").into();
        }
        Mode::Command => {
            app.command_buf.push('.');
        }
    }
}

pub fn comma(app: &mut App) {
    match app.current_mode {
        Mode::Normal | Mode::Visual => {}
        Mode::Insert => {
            let temp_span = app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                [app.insert_cursor.x as usize]
                .clone();
            app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                [app.insert_cursor.x as usize]
                .content = (temp_span.content.to_string() + ",").into();
        }
        Mode::Command => {
            app.command_buf.push(',');
        }
    }
}

pub fn slash(app: &mut App) {
    match app.current_mode {
        Mode::Normal | Mode::Visual => {}
        Mode::Insert => {
            let temp_span = app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                [app.insert_cursor.x as usize]
                .clone();
            app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                [app.insert_cursor.x as usize]
                .content = (temp_span.content.to_string() + "/").into();
        }
        Mode::Command => {
            app.command_buf.push('/');
        }
    }
}

pub fn escape(app: &mut App) {
    app.is_help = false;
    app.current_mode = Mode::Normal;
    let _ = &app.command_buf.clear();
    let _ = &app.current_times.clear();
}

pub fn enter_insert_mode(app: &mut App) {
    match app.current_mode {
        Mode::Normal | Mode::Visual => {
            app.current_mode = Mode::Insert;
            let _ = &app.current_times.clear();
        }
        Mode::Command => {
            app.command_buf.push('i');
        }
        Mode::Insert => {}
    }
}

pub fn rand_interal(app: &mut App, rand_iter: &mut impl Iterator<Item = u8>) {
    match app.current_mode {
        Mode::Insert => {
            let temp_cell = app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                [app.insert_cursor.x as usize]
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
    }
}
pub fn move_left(app: &mut App) {
    let count: u16 = app.current_times.parse().unwrap_or(1);
    let _ = &app.current_times.clear();
    match app.current_mode {
        Mode::Normal | Mode::Visual => {
            //let x_bound = app.rows[app.normal_cursor.y as usize].len() as u16;
            let final_cursor = app.normal_cursor.x.saturating_sub(count);
            if ((app.normal_cursor.y as usize) < app.cols[final_cursor as usize].len())
                && (final_cursor > 0)
            {
                app.normal_cursor.x = final_cursor
            };
        }
        Mode::Insert => {
            let new_cursor_insert = app.insert_cursor.x as isize - count as isize;
            let insert_bound =
                app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize].len() as isize;
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
            } else if new_cursor_normal > 0 {
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

pub fn move_down(app: &mut App) {
    let count: u16 = app.current_times.parse().unwrap_or(1);
    let _ = &app.current_times.clear();
    //cursor.y = cursor.y.saturating_add(count);
    //let new_y = app.normal_cursor.y.saturating_add(count);
    match (app.current_mode, app.page) {
        (Mode::Normal | Mode::Visual | Mode::Insert, Page::Sequencer) => {
            //let y_bound = app.rows[app.normal_cursor.y as usize].len() as u16;
            let new_y = app.normal_cursor.y.saturating_add(count);
            app.normal_cursor.y = if new_y > app.y_bound - 1 {
                app.y_bound - 1
            } else {
                new_y
            };
            app.count_lines();
        }
        (Mode::Command, Page::Sequencer) => {
            app.command_buf.push('j');
        }
        (Mode::Normal | Mode::Insert, Page::Instrument { .. }) => { app.instr_cursor = app.instr_cursor.saturating_add(count as usize); }
        _ => {}
    }
    //app.normal_cursor.y = if new_y > app.y_bound - 1 { app.y_bound - 1 }
    //    else { new_y };
}
pub fn move_up(app: &mut App) {
    let count: u16 = app.current_times.parse().unwrap_or(1);
    let _ = &app.current_times.clear();
    match (app.current_mode, app.page) {
        (Mode::Normal | Mode::Visual | Mode::Insert, Page::Sequencer) => {
            let new_cursor = app.normal_cursor.y.saturating_sub(count);
            if new_cursor > 0 {
                app.normal_cursor.y = new_cursor
            };
            app.count_lines();
        }
        (Mode::Command, Page::Sequencer) => {
            app.command_buf.push('k');
        }
        (Mode::Normal | Mode::Insert, Page::Instrument { .. }) => { app.instr_cursor = app.instr_cursor.saturating_sub(count as usize); }
        _ => {}
    }
    //app.normal_cursor.y = app.normal_cursor.y.saturating_sub(count);
}
pub fn move_right(app: &mut App) {
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
            } else {
                app.normal_cursor.x
            };
        }
        Mode::Insert => {
            let insert_bound =
                app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize].len() as u16;
            let new_cursor_insert = app.insert_cursor.x + count;
            let new_cursor_normal = app
                .normal_cursor
                .x
                .saturating_add(new_cursor_insert / insert_bound);
            if new_cursor_normal < app.cols.len() as u16
                && app.normal_cursor.y >= app.cols[new_cursor_normal as usize].len() as u16
            {
            } else if new_cursor_normal < app.cols.len() as u16 {
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
            app.help_page = if new_page >= help::TEXT.len() {
                help::TEXT.len() - 1
            } else {
                new_page
            };
        }
    }
    //    else { new_x };
}
pub fn goto_end(app: &mut App) {
    let count: u16 = app.current_times.parse().unwrap_or(app.y_bound - 1);
    let _ = &app.current_times.clear();
    match app.current_mode {
        Mode::Normal | Mode::Visual => {
            app.normal_cursor.y = if count < app.y_bound {
                count
            } else {
                app.y_bound - 1
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
pub fn goto_start(app: &mut App) {
    let count: u16 = app.current_times.parse().unwrap_or(0);
    let _ = &app.current_times.clear();
    match app.current_mode {
        Mode::Normal | Mode::Visual => {
            app.normal_cursor.y = if count < app.y_bound {
                count
            } else {
                app.y_bound - 1
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
pub fn add_line(app: &mut App) {
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
pub fn add_fx(app: &mut App) {
    match app.current_mode {
        Mode::Insert | Mode::Normal | Mode::Visual => {
            app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                .extend(vec![Span::from("0"), Span::from("0")]);
            app.count_lines();
        }
        Mode::Command => {
            app.command_buf.push('t');
        }
    }
}
pub fn remove_fx(app: &mut App) {
    match app.current_mode {
        Mode::Normal | Mode::Visual => {}
        Mode::Insert => {
            if app.insert_cursor.x % 2 == 0 {
                app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                    .remove(app.insert_cursor.x as usize);
                app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                    .remove(app.insert_cursor.x as usize - 1);
            } else {
                app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                    .remove(app.insert_cursor.x as usize + 1);
                app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                    .remove(app.insert_cursor.x as usize);
            };
        }
        Mode::Command => {
            app.command_buf.push('T');
        }
    }
}
pub fn add_column(app: &mut App) {
    match app.current_mode {
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
    }
}
pub fn remove_line(app: &mut App) {
    match app.current_mode {
        Mode::Insert | Mode::Normal => {
            app.cols[app.normal_cursor.x as usize].remove(app.normal_cursor.y as usize);
            if app.y_bound - 2 < app.normal_cursor.y {
                app.normal_cursor.y = app.normal_cursor.y.saturating_sub(1);
            }
            app.count_lines();
            app.current_mode = Mode::Normal;
        }
        Mode::Visual => {
            let (min_x, max_x) = minmax_x(app);
            let (min_y, max_y) = minmax_y(app);
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
    }
}

pub fn yank(app: &mut App) {
    match app.current_mode {
        Mode::Insert | Mode::Normal => {
            app.yank_buf = vec![vec![app.cols[app.normal_cursor.x as usize]
                [app.normal_cursor.y as usize]
                .clone()]];
            app.current_mode = Mode::Normal;
        }
        Mode::Visual => {
            let (min_x, max_x) = minmax_x(app);
            let (min_y, max_y) = minmax_y(app);
            app.yank_buf = app.cols[(min_x as usize)..=(max_x as usize)]
                .to_vec()
                .iter()
                .map(|it| it[(min_y as usize)..=(max_y as usize)].to_vec())
                .collect::<Vec<_>>();
            app.current_mode = Mode::Normal;
        }
        Mode::Command => {
            app.command_buf.push('y');
        }
    }
}
pub fn paste_down(app: &mut App) {
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
pub fn paste_up(app: &mut App) {
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
pub fn remove_column(app: &mut App) {
    match app.current_mode {
        Mode::Normal => {
            app.cols.remove(app.normal_cursor.x as usize);
            app.normal_cursor.x = if app.cols.len() - 1 < app.normal_cursor.x as usize {
                app.cols.len() as u16 - 1
            } else {
                app.normal_cursor.x
            };
            app.count_lines();
        }
        Mode::Visual => {
            let (min_x, max_x) = minmax_x(app);
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
            let temp_span = app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                [app.insert_cursor.x as usize]
                .clone();
            app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                [app.insert_cursor.x as usize]
                .content = (temp_span.content.to_string() + "-").into();
        }
        Mode::Command => {
            app.command_buf.push('-');
        }
    }
}
pub fn enter_visual_mode(app: &mut App) {
    match app.current_mode {
        Mode::Insert | Mode::Normal | Mode::Visual => {
            app.current_mode = Mode::Visual;
            (app.visual_cursor.x, app.visual_cursor.y) = (app.normal_cursor.x, app.normal_cursor.y);
        }
        Mode::Command => {
            app.command_buf.push('v');
        }
    }
}
pub fn backspace(app: &mut App) {
    match app.current_mode {
        Mode::Insert => {
            let temp_span = app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                [app.insert_cursor.x as usize]
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
    }
}
pub fn enter_command_mode(app: &mut App) {
    match app.current_mode {
        Mode::Insert | Mode::Normal | Mode::Visual => {
            app.current_mode = Mode::Command;
            app.command_buf.clear();
            app.command_buf.push(':');
        }
        Mode::Command => {
            app.command_buf.push(':');
        }
    }
}
pub fn toggle_help(app: &mut App) {
    match app.current_mode {
        Mode::Insert | Mode::Normal | Mode::Visual => {
            app.is_help = !app.is_help;
        }
        Mode::Command => {}
    }
}
pub fn child_render(app: &mut App) {
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
        std::thread::sleep(std::time::Duration::from_secs(max_len as u64 / 44100));
    });
}

#[cfg(not(feature = "sdl"))]
pub fn open_editor(
    editor: &String,
    full_path_lib: &std::path::PathBuf,
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Command::new(editor).arg(full_path_lib).status()?;
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let _ = terminal.clear();
    Ok(())
}
pub fn child_execute_command(app: &mut App) {
    match app.current_mode {
        Mode::Command => exec_command(app),
        Mode::Normal | Mode::Insert | Mode::Visual => {}
    }
}
pub fn insert_symbol_to_cmd(app: &mut App, matched_code: char) {
    match app.current_mode {
        Mode::Command => {
            app.command_buf.push(matched_code);
        }
        Mode::Normal | Mode::Visual | Mode::Insert => {}
    }
}

pub fn up_cell<const AMOUNT: i8>(app: &mut App) {
    match (app.current_mode, app.page) {
        (Mode::Insert, Page::Sequencer) => {
            if let Ok(num) = app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                [app.insert_cursor.x as usize]
                .to_string()
                .parse::<u8>()
            {
                app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                    [app.insert_cursor.x as usize]
                    .content = num.saturating_add_signed(AMOUNT).to_string().into();
            }
        }
        (Mode::Insert | Mode::Normal, Page::Instrument { id }) => {
            app.instrs[id as usize] = Some(Instrument { name: "synth", synth: Synths::Rust { name: "rust", num: 0, level: 1, fn_symbol: None }});}
        (Mode::Normal | Mode::Visual | Mode::Command, ..) => {},
        _ => {}
    }
}
pub fn down_cell(app: &mut App) {
    match app.current_mode {
        Mode::Insert => {
            if let Ok(num) = app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                [app.insert_cursor.x as usize]
                .to_string()
                .parse::<u16>()
            {
                app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                    [app.insert_cursor.x as usize]
                    .content = (num - 1).to_string().into();
            }
        }
        Mode::Normal | Mode::Visual | Mode::Command => {}
    }
}

pub fn change_page<const SIDE_IS_RIGHT: bool>(app: &mut App) {
    match (app.page, SIDE_IS_RIGHT) {
        (Page::Instrument { .. }, false) => app.page = Page::Sequencer,
        (Page::Sequencer, true) => {
            app.page = Page::Instrument {
                id: app.cols[app.normal_cursor.x as usize][app.normal_cursor.y as usize]
                    .get(6)
                    .unwrap_or(&Span::from("0"))
                    .to_string()
                    .parse::<u8>()
                    .unwrap_or(0),
            }
        }
        (Page::InsturmentList, true) => {}
        _ => {}
    }
}

pub fn change_instr(app: &mut App, id: u8) {

}
