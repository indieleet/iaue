use tinyaudio::prelude::*;
use ratatui::prelude::*;
use std::collections::HashMap;
use crate::synths::Synths;

pub struct App<'a> {
    pub normal_cursor: NormalCursor,
    pub insert_cursor: InsertCursor,
    pub visual_cursor: VisualCursor,
    pub current_times: String,
    pub current_mode: Mode,
    pub audio_params: OutputDeviceParameters,
    pub command_buf: String,
    //file_path: String,
    pub file_name: String,
    pub theme: HashMap<String, style::Color>,
    pub x_bound: u16,
    pub y_bound: u16,
    pub cols: Vec<Vec<Vec<Span<'a>>>>,
    pub instrs: [Option<Instrument<'a>>; 256],
    pub yank_buf: Vec<Vec<Vec<Span<'a>>>>,
    //constrains: Vec<Constraint>,
    pub page: Page,
    pub help_page: usize,
    pub instr_cursor: usize,
    pub is_help: bool,
    pub should_leave: bool,
    pub x_active: bool,
    pub lib: libloading::Library
}

#[derive(Debug, Copy, Clone)]
pub struct Instrument<'a> {
    pub name: &'a str,
    pub synth: Synths<'a>
}

#[derive(Debug, Copy, Clone)]
pub enum Page {
    Sequencer,
    Instrument { id: u8 },
    InsturmentList,
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
}

impl App<'_> {
   pub fn count_lines(&mut self) {
        let max_y = self.cols[1..].iter().map(|it| it.len()).max().unwrap_or(0);
        let mut cols = (0..max_y as isize)
            .map(|it| (it - self.normal_cursor.y as isize).abs())
            .map(|it| vec![Span::from(it.to_string()).style(self.theme["fg_dark"])])
            .collect::<Vec<_>>();
        cols[self.normal_cursor.y as usize][0] =
            Span::from(self.normal_cursor.y.to_string()).style(self.theme["orange"]);
        self.cols[0] = cols;
    }
    pub fn count_bound(&self) -> usize {
        let bound = self.cols[self.normal_cursor.x as usize][self.normal_cursor.y as usize]
            .iter()
            .map(|it| it.content.len() + 1)
            .sum::<usize>();
        if bound < 14 {
            14
        } else {
            bound
        }
    }
}

#[derive(Debug, Default)]
pub struct NormalCursor {
   pub x: u16,
   pub y: u16,
}

#[derive(Debug, Default)]
pub struct InsertCursor {
   pub x: u16,
}

#[derive(Debug, Default)]
pub struct VisualCursor {
    pub x: u16,
    pub y: u16,
}
