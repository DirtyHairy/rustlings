use anyhow::{Result, format_err};
use sdl3::pixels::{Color, PixelFormat};

use crate::game_data::{Bitmap, PALETTE_SIZE, PaletteEntry};

pub fn recast_buffer<T>(data: &mut [u8]) -> Result<&mut [T]> {
    unsafe {
        let (prefix, res, _) = data.align_to_mut::<T>();

        if prefix.len() != 0 {
            Err(anyhow::format_err!("misaligned texture data"))
        } else {
            Ok(res)
        }
    }
}

pub fn copy_bitmap_to_texture_data<T: Fn(Color) -> Color>(
    bitmap: &Bitmap,
    palette: &[PaletteEntry; PALETTE_SIZE],
    texture_data: &mut [u8],
    mapping: T,
) -> Result<()> {
    let data32 = recast_buffer::<u32>(texture_data)?;

    for i in 0..bitmap.width * bitmap.height {
        data32[i] = if bitmap.transparency[i] {
            0
        } else {
            let (r, g, b) = palette[bitmap.data[i] as usize];
            mapping(Color::RGBA(r, g, b, 0xff)).to_u32(&PixelFormat::RGBA8888)
        };
    }

    Ok(())
}

pub fn copy_bitmap_to_texture_data_at(
    bitmap: &Bitmap,
    palette: &[PaletteEntry; PALETTE_SIZE],
    x: u32,
    y: u32,
    texture_data: &mut [u8],
    pitch: usize,
) -> Result<()> {
    let data32 = recast_buffer::<u32>(texture_data)?;

    let delta = (pitch >> 2)
        .checked_sub(bitmap.width)
        .ok_or(format_err!("invalid pitch for bitmap width"))?;

    let mut i_data = y as usize * (pitch >> 2) + x as usize;
    let mut i_bitmap = 0;
    for _ in 0..bitmap.height {
        for _ in 0..bitmap.width {
            data32[i_data] = if bitmap.transparency[i_bitmap] {
                0
            } else {
                let (r, g, b) = palette[bitmap.data[i_bitmap] as usize];
                Color::RGBA(r, g, b, 0xff).to_u32(&PixelFormat::RGBA8888)
            };

            i_data += 1;
            i_bitmap += 1;
        }

        i_data += delta;
    }

    Ok(())
}
