#[derive(Default, Clone, Copy)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Rect {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

impl From<sdl3::rect::Rect> for Rect {
    fn from(value: sdl3::rect::Rect) -> Self {
        Rect {
            x: value.x as usize,
            y: value.y as usize,
            width: value.w as usize,
            height: value.h as usize,
        }
    }
}

impl From<Rect> for sdl3::rect::Rect {
    fn from(value: Rect) -> Self {
        sdl3::rect::Rect::new(
            value.x as i32,
            value.y as i32,
            value.width as u32,
            value.height as u32,
        )
    }
}
