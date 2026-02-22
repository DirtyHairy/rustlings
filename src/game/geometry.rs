use std::fmt::Display;

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
