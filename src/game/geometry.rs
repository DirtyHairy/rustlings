use std::fmt::Display;

#[derive(Default, Clone, Copy)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

impl From<&Rect> for sdl3::render::FRect {
    fn from(value: &Rect) -> Self {
        sdl3::render::FRect::new(
            value.x as f32,
            value.y as f32,
            value.width as f32,
            value.height as f32,
        )
    }
}

impl Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}@{},{}", self.width, self.height, self.x, self.y)
    }
}
