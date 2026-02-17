use anyhow::{Result, anyhow, bail};
use std::fmt::Display;

#[derive(Clone)]
pub struct Bitmap {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
    pub transparency: Vec<bool>,
}

#[derive(Clone)]
pub struct Sprite {
    pub width: usize,
    pub height: usize,
    pub frames: Vec<Bitmap>,
}

#[derive(Clone, Copy)]
pub enum TransparencyEncoding<'a> {
    Black,
    PlanarAt(&'a [u8]),
    PlanarOffset(usize),
}

impl Bitmap {
    pub fn read_planar(
        width: usize,
        height: usize,
        bpp: usize,
        data: &[u8],
        transparency_encoding: TransparencyEncoding,
    ) -> Result<Bitmap> {
        if bpp > 8 {
            bail!("bad bpp {}", bpp);
        }

        if width * height % 8 != 0 {
            bail!("bad dimensions {}x{}", width, height);
        }

        let plane_size = width * height / 8;

        let mut bitmap = Bitmap {
            width,
            height,
            data: vec![0; width * height],
            transparency: vec![false; width * height],
        };

        for iplane in 0..bpp {
            let base = plane_size * iplane;

            let mut i: usize = 0;
            for y in 0..height {
                for x in 0..width {
                    let byte = *data.get(base + i / 8).ok_or(anyhow!(
                        "read_planar: out of bounds {} {} {} {}",
                        x,
                        y,
                        i,
                        data.len()
                    ))?;

                    bitmap.data[i] |= ((byte >> (7 - (i % 8))) & 0x01) << iplane;

                    i += 1;
                }
            }
        }

        let effective_transparency_encoding = match transparency_encoding {
            TransparencyEncoding::PlanarOffset(offset) => TransparencyEncoding::PlanarAt(
                data.get(offset..).ok_or(anyhow!("unable to obtain mask"))?,
            ),
            _ => transparency_encoding,
        };

        match effective_transparency_encoding {
            TransparencyEncoding::Black => {
                let mut i: usize = 0;
                for _ in 0..height {
                    for _ in 0..width {
                        bitmap.transparency[i] = bitmap.data[i] == 0;
                        i += 1;
                    }
                }
            }
            TransparencyEncoding::PlanarAt(transparency_data) => {
                let mut i: usize = 0;
                for _ in 0..height {
                    for _ in 0..width {
                        let byte = *transparency_data
                            .get(i / 8)
                            .ok_or(anyhow!("read_planar: transparency: out of bounds"))?;

                        bitmap.transparency[i] = ((byte >> (7 - (i % 8))) & 0x01) == 0x00;
                        i += 1;
                    }
                }
            }
            _ => (),
        }

        Ok(bitmap)
    }
}

impl Display for Sprite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for frame in &self.frames {
            let mut i: usize = 0;

            for _ in 0..self.height {
                for _ in 0..self.width {
                    let pixel = frame.data[i];
                    let char = if pixel == 0 {
                        String::from(" ")
                    } else {
                        pixel.to_string()
                    };

                    i += 1;
                    write!(f, "{}{}", char, char)?;
                }

                writeln!(f)?;
            }

            writeln!(f)?;
        }

        std::fmt::Result::Ok(())
    }
}

impl Sprite {
    pub fn read_planar(
        frame_count: usize,
        width: usize,
        height: usize,
        bpp: usize,
        data: &[u8],
        offset: &mut usize,
        frame_size: usize,
        transparency_encoding: TransparencyEncoding,
    ) -> Result<Sprite> {
        let mut sprite = Sprite {
            width,
            height,
            frames: Vec::with_capacity(frame_count),
        };

        for iframe in 0..frame_count {
            let base = *offset + iframe * frame_size;

            sprite.frames.push(Bitmap::read_planar(
                width,
                height,
                bpp,
                data.get(base..base + frame_size)
                    .ok_or(anyhow!("Sprite::read_planar: out of bounds"))?,
                transparency_encoding,
            )?);
        }

        *offset += frame_size * frame_count;

        Ok(sprite)
    }
}
