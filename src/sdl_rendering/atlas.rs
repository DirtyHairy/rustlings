use anyhow::{Result, format_err};
use sdl3::sys::blendmode::SDL_BlendMode;
use sdl3::{pixels::PixelFormat, rect::Rect, render::*};

use crate::game_data::PALETTE_SIZE;
use crate::sdl3_aux::apply_blend_mode;
use crate::{
    game_data::{PaletteEntry, Sprite},
    sdl_rendering::util::copy_bitmap_to_texture_data_at,
};

struct AtlasFrame {
    x: u32,
    y: u32,
}

pub struct AtlasSprite {
    pub height: u32,
    pub width: u32,

    frames: Vec<AtlasFrame>,
}

pub struct SdlAtlas<'texture_creator> {
    texture: Texture<'texture_creator>,
    sprites: Vec<AtlasSprite>,
}

impl<'texture_creator> SdlAtlas<'texture_creator> {
    pub fn get_sprite(&self, index: usize) -> Result<&AtlasSprite> {
        self.sprites
            .get(index)
            .ok_or(format_err!("invalid sprite index {}", index))
    }

    pub fn apply_blend_mode(&mut self, blend: SDL_BlendMode) -> bool {
        apply_blend_mode(&mut self.texture, blend)
    }

    pub fn width(&self) -> u32 {
        self.texture.width()
    }

    pub fn height(&self) -> u32 {
        self.texture.height()
    }

    pub fn blit<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        sprite_index: usize,
        x: i32,
        y: i32,
        iframe: u32,
        flip_x: bool,
        flip_y: bool,
    ) -> Result<()> {
        let sprite = self
            .sprites
            .get(sprite_index)
            .ok_or(format_err!("invalid sprite index {}", sprite_index))?;

        let frame = sprite
            .frames
            .get(iframe as usize)
            .ok_or(format_err!("invalid frame index {}", iframe))?;

        canvas
            .copy_ex(
                &self.texture,
                Rect::new(frame.x as i32, frame.y as i32, sprite.width, sprite.height),
                Rect::new(x, y, sprite.width, sprite.height),
                0.,
                None,
                flip_x,
                flip_y,
            )
            .map_err(anyhow::Error::from)
    }
}

pub struct SdlAtlasBuilder<'a> {
    sprites: Vec<&'a Sprite>,
}

impl<'a> SdlAtlasBuilder<'a> {
    pub fn new() -> Self {
        SdlAtlasBuilder {
            sprites: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        SdlAtlasBuilder {
            sprites: Vec::with_capacity(capacity),
        }
    }

    pub fn add_sprite(&mut self, sprite: &'a Sprite) -> usize {
        self.sprites.push(sprite);

        self.sprites.len() - 1
    }

    pub fn build<'texture_creator, T>(
        self,
        palette: &[PaletteEntry; PALETTE_SIZE],
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<SdlAtlas<'texture_creator>> {
        let width = self
            .sprites
            .iter()
            .map(|s| s.frames.len() * s.width)
            .max()
            .unwrap_or(0) as u32;

        let mut atlas_sprites: Vec<_> = self
            .sprites
            .iter()
            .map(|s| AtlasSprite {
                height: s.height as u32,
                width: s.width as u32,
                frames: Vec::with_capacity(s.frames.len()),
            })
            .collect();

        let mut sprites_sorted: Vec<_> = self.sprites.iter().copied().enumerate().collect();
        sprites_sorted.sort_by(|s1, s2| s2.1.height.cmp(&s1.1.height));

        let mut x: u32 = 0;
        let mut y: u32 = 0;
        let mut row_height: u32 = 0;

        for (i, sprite) in sprites_sorted.iter().copied() {
            let atlas_sprite = &mut atlas_sprites[i];

            for _ in 0..sprite.frames.len() {
                if x + sprite.width as u32 > width {
                    y += row_height;
                    x = 0;
                    row_height = 0;
                }

                atlas_sprite.frames.push(AtlasFrame { x, y });

                x += sprite.width as u32;
                row_height = row_height.max(sprite.height as u32);
            }
        }

        let height = y + row_height;
        let pitch = 4 * width as usize;

        let mut texture_data = vec![0u8; height as usize * pitch];

        for (sprite, atlas_sprite) in self.sprites.iter().copied().zip(atlas_sprites.iter()) {
            for (bitmap, frame) in sprite.frames.iter().zip(atlas_sprite.frames.iter()) {
                copy_bitmap_to_texture_data_at(
                    bitmap,
                    palette,
                    frame.x,
                    frame.y,
                    &mut texture_data,
                    pitch,
                )?;
            }
        }

        let mut texture = texture_creator.create_texture(
            PixelFormat::RGBA8888,
            TextureAccess::Static,
            width,
            height,
        )?;
        texture.set_scale_mode(ScaleMode::Nearest);

        texture.update(Rect::new(0, 0, width, height), &texture_data, pitch)?;

        Ok(SdlAtlas {
            sprites: atlas_sprites,
            texture,
        })
    }
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
