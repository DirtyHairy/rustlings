use std::fmt::Display;

pub struct Bitmap {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl Bitmap {
    pub fn read_planar(width: usize, height: usize, bpp: usize, data: &[u8]) -> Result<Bitmap, ()> {
        assert!(bpp < 8 && bpp > 0);

        if width * height % 8 != 0 {
            return Err(());
        }

        let plane_size = width * height / 8;

        if data.len() != plane_size * bpp {
            return Err(());
        }

        let mut bitmap = Bitmap {
            width,
            height,
            data: vec![0; width * height],
        };

        for iplane in 0..bpp {
            let base = plane_size * iplane;

            for x in 0..width {
                for y in 0..height {
                    let ipixel = (y * width) + x;

                    bitmap.data[ipixel] |=
                        ((data[base + ipixel / 8] >> (7 - (ipixel % 8))) & 0x01) << iplane;
                }
            }
        }

        Ok(bitmap)
    }
}

pub struct Sprite {
    pub width: usize,
    pub height: usize,
    pub frames: Vec<Bitmap>,
}

impl Sprite {
    pub fn read_planar(
        frame_count: usize,
        width: usize,
        height: usize,
        bpp: usize,
        data: &[u8],
    ) -> Result<Sprite, ()> {
        assert!(bpp < 8);

        if width * height % 8 != 0 {
            return Err(());
        }

        let frame_size = width * height / 8 * bpp;
        if data.len() < frame_size * frame_count {
            return Err(());
        }

        let mut sprite = Sprite {
            width,
            height,
            frames: Vec::with_capacity(frame_count),
        };

        for iframe in 0..frame_count {
            let base = iframe * frame_size;

            sprite.frames.push(Bitmap::read_planar(
                width,
                height,
                bpp,
                &data[base..base + frame_size],
            )?);
        }

        Ok(sprite)
    }
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

        Ok(())
    }
}
