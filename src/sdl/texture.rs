use anyhow::Result;
use sdl3::{
    pixels::{Color, PixelFormat},
    rect::Rect,
    render::{BlendMode, Canvas, RenderTarget, ScaleMode, Texture, TextureAccess, TextureCreator},
};

use crate::{
    game_data::{Bitmap, PALETTE_SIZE, PaletteEntry},
    sdl::util::copy_bitmap_to_texture_data,
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
        bitmap.width,
        bitmap.height,
    )?;
    texture.set_scale_mode(ScaleMode::Nearest);

    let mut texture_data = vec![0u8; bitmap.width as usize * bitmap.height as usize * 4];
    copy_bitmap_to_texture_data(bitmap, palette, &mut texture_data, mapping)?;

    texture.update(
        Rect::new(0, 0, bitmap.width, bitmap.height),
        &texture_data,
        4 * bitmap.width as usize,
    )?;

    texture.set_blend_mode(BlendMode::Blend);

    Ok(texture)
}

pub fn with_texture_canvas<T: RenderTarget, F>(
    canvas: &mut Canvas<T>,
    texture: &mut Texture,
    f: F,
) -> Result<()>
where
    F: FnOnce(&mut Canvas<T>) -> Result<()>,
{
    let mut render_result: Result<()> = Ok(());

    canvas.with_texture_canvas(texture, |c| {
        render_result = f(c);
    })?;

    render_result.map_err(anyhow::Error::from)
}
