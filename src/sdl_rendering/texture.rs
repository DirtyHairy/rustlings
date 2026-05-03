use anyhow::Result;
use sdl3::{
    pixels::{Color, PixelFormat},
    rect::Rect,
    render::{BlendMode, ScaleMode, Texture, TextureAccess, TextureCreator},
};

use crate::{
    game_data::{Bitmap, PALETTE_SIZE, PaletteEntry},
    sdl_rendering::util::copy_bitmap_to_texture_data,
};

pub fn texture_from_bitmap<'a, T>(
    bitmap: &Bitmap,
    palette: &[PaletteEntry; PALETTE_SIZE],
    texture_creator: &'a TextureCreator<T>,
) -> Result<Texture<'a>> {
    texture_from_bitmap_mapped(bitmap, palette, texture_creator, |c| c)
}

pub fn texture_from_bitmap_mapped<'a, T, F: Fn(Color) -> Color>(
    bitmap: &Bitmap,
    palette: &[PaletteEntry; PALETTE_SIZE],
    texture_creator: &'a TextureCreator<T>,
    mapping: F,
) -> Result<Texture<'a>> {
    let mut texture = texture_creator.create_texture(
        PixelFormat::RGBA8888,
        TextureAccess::Static,
        bitmap.width as u32,
        bitmap.height as u32,
    )?;
    texture.set_scale_mode(ScaleMode::Nearest);

    let mut texture_data = vec![0u8; bitmap.width * bitmap.height * 4];
    copy_bitmap_to_texture_data(bitmap, palette, &mut texture_data, mapping)?;

    texture.update(
        Rect::new(0, 0, bitmap.width as u32, bitmap.height as u32),
        &texture_data,
        4 * bitmap.width,
    )?;

    texture.set_blend_mode(BlendMode::Blend);

    Ok(texture)
}
