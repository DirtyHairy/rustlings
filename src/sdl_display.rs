use anyhow::*;
use sdl2::{pixels::PixelFormatEnum, rect::Rect, render::*};

use crate::file::sprite::{Bitmap, Sprite};

pub struct SDLSprite<'a> {
    pub width: usize,
    pub height: usize,
    pub frame_count: usize,
    texture: Texture<'a>,
}

impl<'a> SDLSprite<'a> {
    pub fn from_sprite<T>(
        sprite: &Sprite,
        palette: &[u32; 16],
        texture_creator: &'a TextureCreator<T>,
    ) -> Result<SDLSprite<'a>> {
        let mut texture = texture_creator.create_texture(
            PixelFormatEnum::RGBA8888,
            TextureAccess::Static,
            (sprite.width * sprite.frames.len()) as u32,
            sprite.height as u32,
        )?;

        let mut bitmap_data = vec![0u32; sprite.width * sprite.height];

        for iframe in 0..sprite.frames.len() {
            for x in 0..sprite.width {
                for y in 0..sprite.height {
                    bitmap_data[(y * sprite.width) + x] =
                        palette[sprite.frames[iframe].data[(y * sprite.width) + x] as usize];
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
        palette: &[u32; 16],
        texture_creator: &'a TextureCreator<T>,
    ) -> Result<SDLSprite<'a>> {
        let mut texture = texture_creator.create_texture(
            PixelFormatEnum::RGBA8888,
            TextureAccess::Static,
            bitmap.width as u32,
            bitmap.height as u32,
        )?;

        let mut bitmap_data = vec![0u32; bitmap.width * bitmap.height];

        for x in 0..bitmap.width {
            for y in 0..bitmap.height {
                bitmap_data[(y * bitmap.width) + x] =
                    palette[bitmap.data[(y * bitmap.width) + x] as usize];
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
        &self,
        canvas: &mut Canvas<T>,
        x: i32,
        y: i32,
        iframe: usize,
        scale: usize,
    ) -> Result<()> {
        return canvas
            .copy(
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
            )
            .map_err(|s| anyhow!(s));
    }
}
