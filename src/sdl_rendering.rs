use anyhow::{Result, anyhow};
use sdl3::{
    pixels::{Color, PixelFormat},
    rect::Rect,
    render::*,
};

use crate::game_data::PALETTE_SIZE;
use crate::game_data::{Bitmap, PaletteEntry, Sprite};

fn copy_bitmap_to_texture_data(
    bitmap: &Bitmap,
    palette: &[PaletteEntry; PALETTE_SIZE],
    texture_data: &mut [u8],
) -> Result<()> {
    let data32: &mut [u32];
    unsafe {
        let (prefix, x, _) = texture_data.align_to_mut::<u32>();
        if prefix.len() != 0 {
            return Err(anyhow!("misaligned texture data"));
        }

        data32 = x;
    }

    let mut i = 0;
    for _ in 0..bitmap.width {
        for _ in 0..bitmap.height {
            data32[i] = if bitmap.transparency[i] {
                0
            } else {
                let (r, g, b) = palette[bitmap.data[i] as usize];
                Color::RGBA(r, g, b, 0xff).to_u32(&PixelFormat::RGBA8888)
            };

            i += 1;
        }
    }

    Ok(())
}

pub fn texture_from_bitmap<'a, T>(
    bitmap: &Bitmap,
    palette: &[PaletteEntry; PALETTE_SIZE],
    texture_creator: &'a TextureCreator<T>,
) -> Result<Texture<'a>> {
    let mut texture = texture_creator.create_texture(
        PixelFormat::RGBA8888,
        TextureAccess::Static,
        bitmap.width as u32,
        bitmap.height as u32,
    )?;
    texture.set_scale_mode(ScaleMode::Nearest);

    let mut texture_data = vec![0u8; bitmap.width * bitmap.height * 4];
    copy_bitmap_to_texture_data(bitmap, palette, &mut texture_data)?;

    texture.update(
        Rect::new(0, 0, bitmap.width as u32, bitmap.height as u32),
        &texture_data,
        4 * bitmap.width,
    )?;

    texture.set_blend_mode(BlendMode::Blend);

    Ok(texture)
}

pub struct SDLSprite<'a> {
    pub width: usize,
    pub height: usize,
    pub frame_count: usize,
    texture: Texture<'a>,
}

impl<'a> SDLSprite<'a> {
    pub fn from_sprite<T>(
        sprite: &Sprite,
        palette: &[PaletteEntry; PALETTE_SIZE],
        texture_creator: &'a TextureCreator<T>,
    ) -> Result<Self> {
        let mut texture = texture_creator.create_texture(
            PixelFormat::RGBA8888,
            TextureAccess::Static,
            (sprite.width * sprite.frames.len()) as u32,
            sprite.height as u32,
        )?;
        texture.set_scale_mode(ScaleMode::Nearest);

        let mut texture_data = vec![0u8; sprite.width * sprite.height * 4];

        for iframe in 0..sprite.frames.len() {
            copy_bitmap_to_texture_data(&sprite.frames[iframe], palette, &mut texture_data)?;

            texture.update(
                Rect::new(
                    (iframe * sprite.width) as i32,
                    0,
                    (sprite.width) as u32,
                    (sprite.height) as u32,
                ),
                &texture_data,
                4 * sprite.width,
            )?;
        }

        texture.set_blend_mode(BlendMode::Blend);

        Ok(SDLSprite {
            width: sprite.width,
            height: sprite.height,
            texture,
            frame_count: sprite.frames.len(),
        })
    }

    pub fn from_bitmap<T>(
        bitmap: &Bitmap,
        palette: &[PaletteEntry; PALETTE_SIZE],
        texture_creator: &'a TextureCreator<T>,
    ) -> Result<Self> {
        let texture = texture_from_bitmap(bitmap, palette, texture_creator)?;

        Ok(SDLSprite {
            width: bitmap.width,
            height: bitmap.height,
            texture,
            frame_count: 1,
        })
    }

    pub fn blit<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        x: i32,
        y: i32,
        iframe: usize,
        scale: usize,
        flip_y: bool,
    ) -> Result<()> {
        canvas
            .copy_ex(
                &self.texture,
                Rect::new(
                    ((iframe % self.frame_count) * self.width) as i32,
                    0,
                    (self.width) as u32,
                    self.height as u32,
                ),
                Rect::new(
                    x,
                    y,
                    (scale * self.width) as u32,
                    (scale * self.height) as u32,
                ),
                0.,
                None,
                false,
                flip_y,
            )
            .map_err(anyhow::Error::from)
    }
}
