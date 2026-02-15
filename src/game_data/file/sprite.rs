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

            for y in 0..height {
                for x in 0..width {
                    let ipixel = (y * width) + x;

                    let byte = *data.get(base + ipixel / 8).ok_or(anyhow!(
                        "read_planar: out of bounds {} {} {} {}",
                        x,
                        y,
                        ipixel,
                        data.len()
                    ))?;

                    bitmap.data[ipixel] |= ((byte >> (7 - (ipixel % 8))) & 0x01) << iplane;
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
                for y in 0..height {
                    for x in 0..width {
                        let ipixel = (y * width) + x;
                        bitmap.transparency[ipixel] = bitmap.data[ipixel] == 0;
                    }
                }
            }
            TransparencyEncoding::PlanarAt(transparency_data) => {
                for y in 0..height {
                    for x in 0..width {
                        let ipixel = (y * width) + x;

                        let byte = *transparency_data
                            .get(ipixel / 8)
                            .ok_or(anyhow!("read_planar: transparency: out of bounds"))?;

                        bitmap.transparency[ipixel] = ((byte >> (7 - (ipixel % 8))) & 0x01) == 0x00;
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
