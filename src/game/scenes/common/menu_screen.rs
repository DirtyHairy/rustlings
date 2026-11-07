use anyhow::Result;
use rustlings::{
    game_data::{
        GameData,
        file::{main::resolve_main_font_index, palette::MENU_SCREEN_PALETTE},
    },
    sdl::{SDLSprite, texture_from_bitmap},
};
use sdl3::{
    pixels::Color,
    rect::Rect,
    render::{Canvas, RenderTarget, TextureCreator},
};

pub struct TextGrid {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl TextGrid {
    pub fn new(width_px: u32, height_px: u32) -> Self {
        let width = width_px / 16;
        let height = height_px / 16;

        let data: Vec<u8> = vec![255; (width * height) as usize];

        Self {
            width,
            height,
            data,
        }
    }

    pub fn write(&mut self, x: u32, y: u32, text: &str) {
        if y >= self.height {
            return;
        }

        let offset = (y * self.width) as usize;

        for (i, c) in text.chars().enumerate() {
            if x + i as u32 >= self.width {
                return;
            }

            self.data[offset + x as usize + i] = if c == ' ' {
                255
            } else {
                resolve_main_font_index(c) as u8
            };
        }
    }

    pub fn render<T, U: RenderTarget>(
        &self,
        game_data: &GameData,
        texture_creator: &TextureCreator<T>,
        canvas: &mut Canvas<U>,
    ) -> Result<()> {
        let font_sprite =
            SDLSprite::from_sprite(&game_data.font_main, &MENU_SCREEN_PALETTE, texture_creator)?;

        for y in 0..self.height {
            for x in 0..self.width {
                let index = self.data[(y * self.width + x) as usize];

                if index < 255 {
                    font_sprite.blit(
                        canvas,
                        x as i32 * 16,
                        y as i32 * 16,
                        index as usize,
                        1,
                        false,
                        false,
                    )?;
                }
            }
        }

        Ok(())
    }
}

pub fn draw_background<T, U: RenderTarget>(
    y: u32,
    game_data: &GameData,
    texture_creator: &TextureCreator<T>,
    canvas: &mut Canvas<U>,
) -> Result<()> {
    let bg_width = game_data.screen_background.width;
    let bh_height = game_data.screen_background.height;

    let texture_background = texture_from_bitmap(
        &game_data.screen_background,
        &MENU_SCREEN_PALETTE,
        texture_creator,
        sdl3::render::TextureAccess::Static,
    )?;

    canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
    canvas.set_blend_mode(sdl3::render::BlendMode::None);
    canvas.clear();

    let (width, height) = canvas.output_size()?;

    let mut y = y as i32;
    while y < height as i32 {
        let mut x = 0;
        while x < width as i32 {
            canvas.copy(
                &texture_background,
                None,
                Rect::new(x, y, bg_width, bh_height),
            )?;

            x += bg_width as i32;
        }

        y += bh_height as i32;
    }

    Ok(())
}
