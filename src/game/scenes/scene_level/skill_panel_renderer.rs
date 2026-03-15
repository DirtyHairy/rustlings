use std::rc::Rc;

use anyhow::{Result, bail};
use rustlings::{
    game_data::{
        GameData, Level, NUM_SKILLS, SCREEN_WIDTH, SKILL_PANEL_HEIGHT, SKILL_TILE_LABEL_X,
        SKILL_TILE_LABEL_Y, SKILL_TILE_WIDTH, SKILL_TILE_Y, resolve_skill_panel_skill_font_index,
    },
    sdl_rendering::{SDLSprite, texture_from_bitmap, with_texture_canvas},
};
use sdl3::{
    pixels::{Color, PixelFormat},
    render::{BlendMode, Canvas, RenderTarget, ScaleMode, Texture, TextureCreator},
    video::Window,
};

use crate::state::{CursorState, SceneStateLevel};

pub struct SkillPanelRenderer<'texture_creator> {
    texture_skill_panel: Texture<'texture_creator>,
    pub texture: Texture<'texture_creator>,

    font: SDLSprite<'texture_creator>,
    font_skills: SDLSprite<'texture_creator>,

    force_redraw: bool,

    remaining_skills: [usize; NUM_SKILLS],

    lemmings_out: usize,
    lemmings_in: usize,
    release_rate: usize,
    release_rate_min: usize,

    remaining_time_seconds: usize,

    cursor_state: Option<CursorState>,
}

impl<'texture_creator> SkillPanelRenderer<'texture_creator> {
    pub fn new<T>(
        level: &Level,
        game_data: Rc<GameData>,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        let palette_skill_panel = game_data.resolve_skill_panel_palette(0);

        let texture_skill_panel = texture_from_bitmap(
            &game_data.skill_panel.panel,
            &palette_skill_panel,
            texture_creator,
        )?;

        let texture = texture_creator.create_texture_target(
            PixelFormat::RGBA8888,
            SCREEN_WIDTH as u32,
            SKILL_PANEL_HEIGHT as u32,
        )?;

        let mut font = SDLSprite::from_sprite(
            &game_data.skill_panel.font,
            &palette_skill_panel,
            texture_creator,
        )?;

        font.texture().set_blend_mode(BlendMode::None);
        font.texture().set_scale_mode(ScaleMode::Nearest);

        let mut font_skills = SDLSprite::from_sprite(
            &game_data.skill_panel.font_skills,
            &palette_skill_panel,
            texture_creator,
        )?;

        font_skills.texture().set_blend_mode(BlendMode::None);
        font_skills.texture().set_scale_mode(ScaleMode::Nearest);

        Ok(Self {
            texture_skill_panel,
            texture,
            font,
            font_skills,
            force_redraw: true,
            remaining_skills: [0; NUM_SKILLS],
            lemmings_out: 0,
            lemmings_in: 0,
            release_rate: 0,
            release_rate_min: level.parameters.release_rate as usize,
            remaining_time_seconds: 0,
            cursor_state: None,
        })
    }

    pub fn draw(&mut self, state: &SceneStateLevel, canvas: &mut Canvas<Window>) -> Result<bool> {
        let mut updated = false;

        with_texture_canvas(canvas, &mut self.texture, |canvas| {
            canvas.set_draw_color(Color::RGBA(0, 0, 0, 0xff));
            canvas.clear();

            if self.force_redraw {
                self.texture_skill_panel
                    .set_blend_mode(sdl3::render::BlendMode::None);
                self.texture_skill_panel.set_scale_mode(ScaleMode::Nearest);
                canvas.copy(&mut self.texture_skill_panel, None, None)?;

                updated = true;
            }

            if self.force_redraw {
                draw_tile_label(canvas, &self.font_skills, 0, self.release_rate_min)?;
                updated = true;
            }

            if state.release_rate != self.release_rate || self.force_redraw {
                draw_tile_label(canvas, &self.font_skills, 1, state.release_rate)?;
                self.release_rate = state.release_rate;
                updated = true;
            }

            for i in 0..NUM_SKILLS {
                if state.remaining_skills[i] == self.remaining_skills[i] && !self.force_redraw {
                    continue;
                }

                draw_tile_label(canvas, &self.font_skills, i + 2, state.remaining_skills[i])?;
                self.remaining_skills[i] = state.remaining_skills[i];

                updated = true;
            }

            Ok(())
        })?;

        self.force_redraw = false;
        Ok(updated)
    }
}

fn draw_tile_label<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    font: &SDLSprite,
    tile_index: usize,
    value: usize,
) -> Result<()> {
    if value > 99 {
        bail!("value {} exceeds bounds", value);
    }

    let (char_10, char_1) = match (value / 10, value % 10) {
        (0, 0) => (' ', ' '),
        (value_10, value_1) => (
            char::from_digit(value_10 as u32, 10).unwrap(),
            char::from_digit(value_1 as u32, 10).unwrap(),
        ),
    };

    let x = tile_index * SKILL_TILE_WIDTH + SKILL_TILE_LABEL_X;
    let y = SKILL_TILE_Y + SKILL_TILE_LABEL_Y;

    font.blit(
        canvas,
        x as i32,
        y as i32,
        resolve_skill_panel_skill_font_index(char_10),
        1,
        false,
    )?;

    font.blit(
        canvas,
        x as i32 + 4,
        y as i32,
        resolve_skill_panel_skill_font_index(char_1),
        1,
        false,
    )?;

    Ok(())
}
