use anyhow::*;
use sdl3::{
    pixels::{Color, PixelFormat},
    rect::Rect,
    render::*,
};

use super::game_data::{Bitmap, PaletteEntry, Sprite};

pub struct SDLSprite<'a> {
    pub width: usize,
    pub height: usize,
    pub frame_count: usize,
    texture: Texture<'a>,
}

impl<'a> SDLSprite<'a> {
    pub fn from_sprite<T>(
        sprite: &Sprite,
        palette: &[PaletteEntry; 16],
        texture_creator: &'a TextureCreator<T>,
    ) -> Result<SDLSprite<'a>> {
        let mut texture = texture_creator.create_texture(
            PixelFormat::RGBA8888,
            TextureAccess::Static,
            (sprite.width * sprite.frames.len()) as u32,
            sprite.height as u32,
        )?;
        texture.set_scale_mode(ScaleMode::Nearest);

        let mut bitmap_data = vec![0u32; sprite.width * sprite.height];

        for iframe in 0..sprite.frames.len() {
            let mut i: usize = 0;

            for _ in 0..sprite.width {
                for _ in 0..sprite.height {
                    bitmap_data[i] = if sprite.frames[iframe].transparency[i] {
                        0
                    } else {
                        let (r, g, b) = palette[sprite.frames[iframe].data[i] as usize];
                        Color::RGBA(r, g, b, 0xff).to_u32(&PixelFormat::RGBA8888)
                    };

                    i += 1;
                }
            }

            let data8: &[u8];
            unsafe {
                let (_, x, _) = bitmap_data.align_to();
                assert_eq!(x.len(), 4 * bitmap_data.len());

                data8 = x;
            }

            texture.update(
                Rect::new(
                    (iframe * sprite.width) as i32,
                    0,
                    (sprite.width) as u32,
                    (sprite.height) as u32,
                ),
                data8,
                4 * sprite.width,
            )?;

            texture.set_blend_mode(BlendMode::Blend)
        }

        Ok(SDLSprite {
            width: sprite.width,
            height: sprite.height,
            texture,
            frame_count: sprite.frames.len(),
        })
    }

    pub fn from_bitmap<T>(
        bitmap: &Bitmap,
        palette: &[PaletteEntry; 16],
        texture_creator: &'a TextureCreator<T>,
    ) -> Result<SDLSprite<'a>> {
        let mut texture = texture_creator.create_texture(
            PixelFormat::RGBA8888,
            TextureAccess::Static,
            bitmap.width as u32,
            bitmap.height as u32,
        )?;
        texture.set_scale_mode(ScaleMode::Nearest);

        let mut bitmap_data = vec![0u32; bitmap.width * bitmap.height];

        let mut i = 0;
        for _ in 0..bitmap.width {
            for _ in 0..bitmap.height {
                bitmap_data[i] = if bitmap.transparency[i] {
                    0
                } else {
                    let (r, g, b) = palette[bitmap.data[i] as usize];
                    Color::RGBA(r, g, b, 0xff).to_u32(&PixelFormat::RGBA8888)
                };

                i += 1;
            }
        }

        let data8: &[u8];
        unsafe {
            let (_, x, _) = bitmap_data.align_to();
            assert_eq!(x.len(), 4 * bitmap_data.len());

            data8 = x;
        }

        texture.update(
            Rect::new(0, 0, bitmap.width as u32, bitmap.height as u32),
            data8,
            4 * bitmap.width,
        )?;

        Ok(SDLSprite {
            width: bitmap.width,
            height: bitmap.height,
            texture,
            frame_count: 1,
        })
    }

    pub fn blit<T: RenderTarget>(
        &mut self,
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
