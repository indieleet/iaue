use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, KeyCode, KeyEventKind, KeyEvent, Event},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    style::Stylize,
    widgets::*,
    Terminal,
};
use std::io::{stdout, Result};

#[derive(Debug, Default)]
pub struct App<'a> {
    counter: u16,
    cursor: u8,
    items: Vec<Constraint>,
    current_times: String,
    columns: u8,
    x_bound: u16,
    y_bound: u16,
    rows: Vec<Vec<Cell<'a>>>,
    constrains: Vec<Constraint>,
}


#[derive(Debug)]
enum Mode {
    Normal,
    Insert,
    Visual,
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
            vec![Cell::from("7/9 1/2 1"), Cell::from("1/2 3/5 1/2")],
            vec![Cell::from("3/2 1 1")],
        ],
        constrains: vec![Constraint::Max(10), Constraint::Max(10)],
    };
    //let mut state = TableState::new();
    loop {
        let mut table_rows = app.rows.to_owned();
        let mut cur_cell_text = Cell::default();
        for (i_row, c_row) in table_rows.iter_mut().enumerate() {
            for (i_el, el) in c_row.iter_mut().enumerate() {
                match current_mode {
                    Mode::Normal if (normal_cursor.x as usize, normal_cursor.y as usize) == (i_el, i_row) => {
                        cur_cell_text = el.clone().add_modifier(Modifier::REVERSED);
                    },
                    Mode::Visual if (normal_cursor.x as usize, normal_cursor.y as usize) == (i_el, i_row) => {
                        cur_cell_text = el.clone().add_modifier(Modifier::REVERSED);
                    },
                    Mode::Insert if (normal_cursor.x as usize, normal_cursor.y as usize) == (i_el, i_row) => {
                        cur_cell_text = el.clone().add_modifier(Modifier::REVERSED);
                    },
                    _ => ()
                }
            }
        }
        //let y_bound = core::iter::repeat_with(|| &app.rows.iter().next().unwrap_or(&Vec::<Cell>::new()).get(normal_cursor.x as usize)).count();
        let mut y_bound: u16 = 0;
        for el in &app.rows {
            if el.get(normal_cursor.x as usize).is_some() {
                y_bound += 1;
            }
            else {
                break;
            }
        }
        table_rows[normal_cursor.y as usize][normal_cursor.x as usize] = cur_cell_text;
        let mut mode_str = match current_mode { 
            Mode::Normal => "Normal",
            Mode::Visual => "Visual",
            Mode::Insert => "Insert",
            _ => todo!()
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
                    table_rows.into_iter().map(Row::new),
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
                code: KeyCode::Char('h'),
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
                }
                //normal_cursor.x = normal_cursor.x.saturating_sub(count);
            },
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('j'),
                ..
            }) => 
            {
                let count: u16 = app.current_times.parse().unwrap_or(1);
                let _ = &app.current_times.clear();
                //cursor.y = cursor.y.saturating_add(count);
                let new_y = normal_cursor.y.saturating_add(count);
                match current_mode {
                    Mode::Normal | Mode::Visual => { 
                        //let y_bound = app.rows[normal_cursor.y as usize].len() as u16;
                        let new_y = normal_cursor.y.saturating_add(count);
                        normal_cursor.y = if new_y > y_bound - 1 { y_bound - 1 }
                            else { new_y };
                    },
                    Mode::Insert => todo!(),
                }
                //normal_cursor.y = if new_y > app.y_bound - 1 { app.y_bound - 1 }
                //    else { new_y };
            },
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('k'),
                ..
            }) => 
            {
                let count: u16 = app.current_times.parse().unwrap_or(1);
                let _ = &app.current_times.clear();
                match current_mode {
                    Mode::Normal | Mode::Visual => { 
                        normal_cursor.y = normal_cursor.y.saturating_sub(count);
                    },
                    Mode::Insert => todo!(),
                }
                normal_cursor.y = normal_cursor.y.saturating_sub(count);
            },
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('l'),
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
                    Mode::Insert => todo!(),
                }
                //cursor.x = if new_x > app.x_bound - 1 { app.x_bound - 1 }
                //    else { new_x };
            },
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('+'),
                ..
            }) => 
            {},
            Event::Key(KeyEvent {
                //modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('v'),
                ..
            }) => 
            {
                current_mode = Mode::Visual;
                (visual_cursor.x, visual_cursor.y) = (normal_cursor.x, normal_cursor.y);
            },
                   // KeyCode::Char('h') => {
                   //     cursor.x = cursor.x.saturating_sub(1);
                   // }
                   // KeyCode::Char('j') => {
                   //     cursor.y = cursor.y.saturating_add(1);
                   // }
                   // KeyCode::Char('k') => {
                   //     cursor.y = cursor.y.saturating_sub(1);
                   // }
                   // KeyCode::Char('l') => {
                   //     cursor.x = cursor.x.saturating_add(1);
                   // }
                   // KeyCode::Char('+') => {
                   //     app.counter = app.counter.saturating_add(1);
                   //     app.rows[0].push("7/9 1/2 1".into());
                   //     app.constrains.push(Constraint::Max(10));
                   // }
                   // KeyCode::Char('-') => {
                   //     app.counter = app.counter.saturating_sub(1);
                   //     app.rows[0].pop();
                   //     app.constrains.pop();
                   // }
            _ => (),
                }
    }
        
    
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
