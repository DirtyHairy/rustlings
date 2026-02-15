use std::fmt::Display;

#[derive(Clone)]
pub struct Bitmap {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
    pub transparency: Vec<bool>,
}

#[derive(Clone, Copy)]
pub enum TransparencyEncoding<'a> {
    Black,
    PlanarAt(&'a [u8]),
    PlanarOffset(usize),
}

#[derive(Clone)]
pub struct Sprite {
    pub width: usize,
    pub height: usize,
    pub frames: Vec<Bitmap>,
}

impl Display for Sprite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for frame in &self.frames {
            for y in 0..self.height {
                for x in 0..self.width {
                    let pixel = frame.data[(y * self.width) + x];
                    let char = if pixel == 0 {
                        String::from(" ")
                    } else {
                        pixel.to_string()
                    };

                    write!(f, "{}{}", char, char)?;
                }

                writeln!(f)?;
            }

            writeln!(f)?;
        }

        std::fmt::Result::Ok(())
    }
}
