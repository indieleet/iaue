use ratatui::backend::{self, Backend};
use ratatui::buffer;
use ratatui::style::Color;
use ratatui::prelude::*;
use std::io;
use crate::fns::*;
use crate::app::App;
#[cfg(feature = "sdl")]
pub struct SdlBackend {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    ctx: sdl2::Sdl,
    ttf_context: sdl2::ttf::Sdl2TtfContext,
}

#[cfg(feature = "sdl")]
impl SdlBackend {
    pub fn new() -> SdlBackend {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("iaue", 640, 480)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();
        let canvas = window
            .into_canvas()
            .software()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
        SdlBackend {
            canvas,
            ctx: sdl_context,
            ttf_context,
        }
    }
}

#[cfg(feature = "sdl")]
impl Backend for SdlBackend {
    fn draw<'a, I>(&mut self, content: I) -> std::io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a buffer::Cell)>,
    {
        let ttf_context = &self.ttf_context;
        let mut font = ttf_context
            .load_font("IosevkaTermSlabNerdFontPropo-Regular.ttf", 9)
            .unwrap();
        font.set_style(sdl2::ttf::FontStyle::NORMAL);
        let texture_creator = self.canvas.texture_creator();
        for (x, y, el) in content {
            let target = sdl2::rect::Rect::new(x as i32 * 8, y as i32 * 12, 8, 12);
            let mut color_fg = match el.fg {
                Color::Rgb(red, green, blue) => (red, green, blue),
                _ => (255, 255, 255),
            };
            let mut color_bg = match el.bg {
                Color::Rgb(red, green, blue) => (red, green, blue),
                _ => (0, 0, 0),
            };
            if let ratatui::style::Modifier::REVERSED = el.modifier {
                core::mem::swap(&mut color_bg, &mut color_fg);
            };
            let bg_rect = sdl2::rect::Rect::new(x as i32 * 8, y as i32 * 12, 8, 12);
            self.canvas.set_draw_color(sdl2::pixels::Color::RGBA(
                color_bg.0, color_bg.1, color_bg.2, 255,
            ));
            let _ = self.canvas.fill_rect(bg_rect);
            let surface = font
                .render(el.symbol())
                .blended(sdl2::pixels::Color::RGBA(
                    color_fg.0, color_fg.1, color_fg.2, 255,
                ))
                .map_err(|e| e.to_string())
                .unwrap();
            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())
                .unwrap();

            //let sdl2::render::TextureQuery { width, height, .. } = texture.query();
            self.canvas.copy(&texture, None, Some(target)).unwrap();
        }
        self.canvas.present();
        Ok(())
    }
    fn size(&self) -> io::Result<Size> {
        Ok(Size::new(80, 40))
    }
    fn clear(&mut self) -> io::Result<()> {
        self.canvas.clear();
        Ok(())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
        Ok((0, 0))
    }
    fn set_cursor(&mut self, _x: u16, _y: u16) -> io::Result<()> {
        Ok(())
    }
    fn hide_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn show_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn window_size(&mut self) -> io::Result<backend::WindowSize> {
        Ok(backend::WindowSize {
            columns_rows: (50, 25).into(),
            pixels: (640, 480).into(),
        })
    }
    fn append_lines(&mut self, _n: u16) -> io::Result<()> {
        Ok(())
    }
    fn clear_region(&mut self, _clear_type: backend::ClearType) -> io::Result<()> {
        Ok(())
    }
    fn get_cursor_position(&mut self) -> io::Result<Position> {
        Ok(Position::new(0, 0))
    }
    fn set_cursor_position<P: Into<Position>>(&mut self, _position: P) -> io::Result<()> {
        Ok(())
    }
}
#[cfg(feature = "sdl")]
pub fn sdl_event(app: &mut App, terminal: &mut Terminal<SdlBackend>) {
    let sdl_context = &terminal.backend_mut().ctx;
    app.y_bound = app.cols[app.normal_cursor.x as usize].len() as u16;
    for event in sdl_context.event_pump().unwrap().poll_iter() {
        use sdl2::event::Event;
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::Escape),
                ..
            } => {
                app.should_leave = true;
                break;
            }
            Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Right),
                keymod: sdl2::keyboard::Mod::LSHIFTMOD,
                ..
            } => change_page(app),
            Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::X),
                keymod: sdl2::keyboard::Mod::LSHIFTMOD,
                ..
            } => paste_down(app),
            Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::X),
                keymod: sdl2::keyboard::Mod::NOMOD,
                ..
            } => app.x_active = true,
            Event::KeyUp {
                keycode: Some(sdl2::keyboard::Keycode::X),
                ..
            } => app.x_active = false,
            Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::Up),
                keymod: sdl2::keyboard::Mod::LCTRLMOD,
                ..
            } => up_cell(app),
            Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::Down),
                keymod: sdl2::keyboard::Mod::LCTRLMOD,
                ..
            } => down_cell(app),
            Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::Left),
                ..
            } => match app.x_active {
                true => down_cell(app),
                false => move_left(app),
            }
            Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::Down),
                ..
            } => move_down(app),
            Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::Up),
                ..
            } => move_up(app),
            Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::Right),
                ..
            } => match app.x_active {
                true => up_cell(app),
                false => move_right(app),
            }

            Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::Home),
                ..
            } => goto_start(app),
            Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::End),
                ..
            } => goto_end(app),
            Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::Space),
                ..
            } => child_render(app),
            Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::Z),
                keymod: sdl2::keyboard::Mod::LSHIFTMOD,
                ..
            } => enter_visual_mode(app),
            Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::E),
                ..
            } => enter_insert_mode(app),
            Event::KeyDown {
                keycode: Some(sdl2::keyboard::Keycode::Z),
                ..
            } => match app.x_active { 
                true => remove_line(app),
                false => yank(app),
            }
            //e => { println!("{:?}", e); }
            _ => {}
        }
    }
}

