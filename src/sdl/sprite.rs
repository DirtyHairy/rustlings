use anyhow::Result;
use sdl3::{
    pixels::PixelFormat,
    rect::Rect,
    render::{BlendMode, Canvas, RenderTarget, ScaleMode, Texture, TextureAccess, TextureCreator},
};

use crate::{
    game_data::{Bitmap, PALETTE_SIZE, PaletteEntry, Sprite},
    sdl::{texture_from_bitmap, util::copy_bitmap_to_texture_data},
};

pub struct SDLSprite<'a> {
    pub width: u32,
    pub height: u32,
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
            copy_bitmap_to_texture_data(&sprite.frames[iframe], palette, &mut texture_data, |c| c)?;

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
            width: sprite.width as u32,
            height: sprite.height as u32,
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
            width: bitmap.width as u32,
            height: bitmap.height as u32,
            texture,
            frame_count: 1,
        })
    }

    pub fn blit<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        x: i32,
        y: i32,
        iframe: u32,
        scale: u32,
        flip_x: bool,
        flip_y: bool,
    ) -> Result<()> {
        canvas
            .copy_ex(
                &self.texture,
                Rect::new(
                    ((iframe % self.frame_count as u32) * self.width) as i32,
                    0,
                    self.width,
                    self.height,
                ),
                Rect::new(x, y, scale * self.width, scale * self.height),
                0.,
                None,
                flip_x,
                flip_y,
            )
            .map_err(anyhow::Error::from)
    }

    pub fn texture(&mut self) -> &mut Texture<'a> {
        &mut self.texture
    }
}
