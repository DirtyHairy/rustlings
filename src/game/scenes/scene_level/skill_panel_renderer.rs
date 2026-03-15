use std::fmt::Write;
use std::rc::Rc;

use anyhow::Result;
use rustlings::{
    game_data::{
        GameData, Level, NUM_ASSIGNABLE_SKILLS, SCREEN_WIDTH, SKILL_PANEL_HEIGHT,
        SKILL_TILE_LABEL_X, SKILL_TILE_LABEL_Y, SKILL_TILE_WIDTH, SKILL_TILE_Y, Skill,
        resolve_skill_panel_font_index, resolve_skill_panel_skill_font_index,
    },
    sdl_rendering::{SDLSprite, texture_from_bitmap, with_texture_canvas},
};
use sdl3::{
    pixels::PixelFormat,
    render::{BlendMode, Canvas, RenderTarget, ScaleMode, Texture, TextureCreator},
    video::Window,
};

use crate::state::{CursorState, SceneStateLevel};

pub struct SkillPanelRenderer<'texture_creator> {
    texture_skill_panel: Texture<'texture_creator>,
    texture: Texture<'texture_creator>,

    font: SDLSprite<'texture_creator>,
    font_skills: SDLSprite<'texture_creator>,

    force_redraw: bool,

    remaining_skills: [usize; NUM_ASSIGNABLE_SKILLS],

    lemmings_out: usize,
    lemmings_in: usize,
    lemmings_released: usize,
    release_rate: usize,
    release_rate_min: usize,

    remaining_time_seconds: usize,

    cursor_state: Option<CursorState>,

    stats_current: String,
    stats_new: String,
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
            remaining_skills: [0; NUM_ASSIGNABLE_SKILLS],
            lemmings_out: 0,
            lemmings_in: 0,
            lemmings_released: level.parameters.released as usize,
            release_rate: 0,
            release_rate_min: level.parameters.release_rate as usize,
            remaining_time_seconds: 0,
            cursor_state: None,
            stats_current: String::with_capacity(40),
            stats_new: String::with_capacity(40),
        })
    }

    pub fn texture(&mut self) -> &mut Texture<'texture_creator> {
        &mut self.texture
    }

    pub fn draw(&mut self, state: &SceneStateLevel, canvas: &mut Canvas<Window>) -> Result<bool> {
        let mut updated = false;

        with_texture_canvas(canvas, &mut self.texture, |canvas| {
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

            for i in 0..NUM_ASSIGNABLE_SKILLS {
                if state.remaining_skills[i] == self.remaining_skills[i] && !self.force_redraw {
                    continue;
                }

                draw_tile_label(canvas, &self.font_skills, i + 2, state.remaining_skills[i])?;
                self.remaining_skills[i] = state.remaining_skills[i];

                updated = true;
            }

            if state.lemmings_in != self.lemmings_in
                || state.lemmings_out != self.lemmings_out
                || state.cursor_state != self.cursor_state
                || state.remaining_time_seconds != self.remaining_time_seconds
                || self.force_redraw
            {
                format_stats(&mut self.stats_new, state, self.lemmings_released);

                draw_stats(
                    canvas,
                    &self.font,
                    &self.stats_current,
                    &self.stats_new,
                    self.force_redraw,
                )?;

                self.stats_current.clear();
                self.stats_current.push_str(&self.stats_new);

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
    mut value: usize,
) -> Result<()> {
    value = value.min(99);

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

fn draw_stats<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    font: &SDLSprite,
    stats_current: &str,
    stats_new: &str,
    force_redraw: bool,
) -> Result<()> {
    for i in 0..40 {
        let char_current = stats_current.as_bytes().get(i).copied().unwrap_or(b' ') as char;
        let char_new = stats_new.as_bytes().get(i).copied().unwrap_or(b' ') as char;

        if char_current == char_new && !force_redraw {
            continue;
        }

        font.blit(
            canvas,
            i as i32 * 8,
            0,
            resolve_skill_panel_font_index(char_new),
            1,
            false,
        )?;
    }

    Ok(())
}

fn skill_name(skill: Skill) -> &'static str {
    match skill {
        Skill::Climber => "CLIMBER",
        Skill::Floater => "FLOATER",
        Skill::Bomber => "BOMBER",
        Skill::Blocker => "BLOCKER",
        Skill::Builder => "BUILDER",
        Skill::Basher => "BASHER",
        Skill::Miner => "MINER",
        Skill::Digger => "DIGGER",
        Skill::Faller => "FALLER",
    }
}

fn format_stats(str: &mut String, state: &SceneStateLevel, lemmings_released: usize) {
    str.clear();

    // 12 characters for the cursor
    if let Some(cursor_state) = &state.cursor_state {
        write!(
            str,
            "{:7} {:<2}  ",
            skill_name(cursor_state.leading_skill),
            cursor_state.lemming_count
        )
        .unwrap();
    } else {
        write!(str, "{:12}", "").unwrap();
    }

    // 10 characters for lemmings out
    write!(str, "OUT {:2}    ", state.lemmings_out).unwrap();

    // 9 characteres for lemmings in
    if lemmings_released == state.lemmings_in {
        write!(str, "IN 100%  ").unwrap();
    } else {
        write!(
            str,
            "IN {:2}%   ",
            (state.lemmings_in * 100)
                .checked_div(lemmings_released)
                .unwrap_or(0)
        )
        .unwrap();
    }

    // 9 characters for time
    write!(
        str,
        "TIME {:1}-{:0>2}",
        state.remaining_time_seconds / 60,
        state.remaining_time_seconds % 60
    )
    .unwrap();
}
