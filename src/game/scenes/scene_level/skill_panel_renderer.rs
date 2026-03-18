use std::fmt::Write;
use std::rc::Rc;

use anyhow::Result;
use rustlings::{
    game_data::{
        GameData, Level, NUM_SKILLS, SCREEN_WIDTH, SKILL_PANEL_HEIGHT, SKILL_TILE_HEIGHT,
        SKILL_TILE_LABEL_X, SKILL_TILE_LABEL_Y, SKILL_TILE_WIDTH, SKILL_TILE_Y, SKILLS, Skill,
        resolve_skill_panel_font_index, resolve_skill_panel_skill_font_index,
    },
    sdl_rendering::{SDLSprite, texture_from_bitmap, with_texture_canvas},
};
use sdl3::{
    pixels::{Color, PixelFormat},
    rect::Rect,
    render::{BlendMode, Canvas, RenderTarget, ScaleMode, Texture, TextureCreator},
    video::Window,
};

use crate::state::{CursorState, Profession, SceneStateLevel};

pub struct SkillPanelRenderer<'texture_creator> {
    texture_skill_panel: Texture<'texture_creator>,
    texture_font_overlay: Texture<'texture_creator>,
    texture_selected_skill_frame: Texture<'texture_creator>,
    texture: Texture<'texture_creator>,

    font: SDLSprite<'texture_creator>,
    font_skills: SDLSprite<'texture_creator>,

    full_redraw: bool,

    lemmings_released_total: usize,
    release_rate_min: usize,
    selected_skill: Skill,

    text_model: SkillPanelTextModel,

    stats_current: String,
    stats_new: String,
}

#[derive(PartialEq, Default)]
struct SkillPanelTextModel {
    remaining_skills: [usize; NUM_SKILLS],

    lemmings_out: usize,
    lemmings_in: usize,
    release_rate: usize,

    remaining_time_seconds: usize,
    cursor_state: Option<CursorState>,
}

impl SkillPanelTextModel {
    fn from_state(state: &SceneStateLevel) -> Self {
        Self {
            remaining_skills: state.remaining_skills.clone(),
            lemmings_out: state.lemmings_out,
            lemmings_in: state.lemmings_in,
            release_rate: state.release_rate,
            remaining_time_seconds: state.remaining_time_seconds,
            cursor_state: state.cursor_state,
        }
    }
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

        let texture_selected_skill_frame = texture_from_bitmap(
            &game_data.skill_panel.skill_tile_frame,
            &palette_skill_panel,
            texture_creator,
        )?;

        let texture_font_overlay = texture_creator.create_texture_target(
            PixelFormat::RGBA8888,
            SCREEN_WIDTH as u32,
            SKILL_PANEL_HEIGHT as u32,
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
            texture_selected_skill_frame,
            texture_font_overlay,
            texture,
            font,
            font_skills,
            full_redraw: true,
            lemmings_released_total: level.parameters.released as usize,
            release_rate_min: level.parameters.release_rate as usize,
            selected_skill: SKILLS[0],
            text_model: Default::default(),
            stats_current: String::with_capacity(40),
            stats_new: String::with_capacity(40),
        })
    }

    pub fn texture(&mut self) -> &mut Texture<'texture_creator> {
        &mut self.texture
    }

    pub fn draw(&mut self, state: &SceneStateLevel, canvas: &mut Canvas<Window>) -> Result<bool> {
        let mut updated = false;

        let text_model = SkillPanelTextModel::from_state(state);

        if text_model != self.text_model || self.full_redraw {
            self.draw_text_overlay(&text_model, canvas)?;

            self.text_model = text_model;
            updated = true;
        }

        if updated || state.selected_skill != self.selected_skill {
            with_texture_canvas(canvas, &mut self.texture, |canvas| {
                self.texture_skill_panel
                    .set_blend_mode(sdl3::render::BlendMode::None);
                self.texture_skill_panel.set_scale_mode(ScaleMode::Nearest);
                canvas.copy(&self.texture_skill_panel, None, None)?;

                self.texture_font_overlay.set_blend_mode(BlendMode::Blend);
                self.texture_font_overlay.set_scale_mode(ScaleMode::Nearest);
                canvas.copy(&self.texture_font_overlay, None, None)?;

                self.texture_selected_skill_frame
                    .set_blend_mode(BlendMode::Blend);
                self.texture_selected_skill_frame
                    .set_scale_mode(ScaleMode::Nearest);
                canvas.copy(
                    &self.texture_selected_skill_frame,
                    None,
                    Rect::new(
                        (state.selected_skill as i32 + 2) * SKILL_TILE_WIDTH as i32,
                        16,
                        SKILL_TILE_WIDTH as u32,
                        SKILL_TILE_HEIGHT as u32,
                    ),
                )?;

                updated = true;
                self.selected_skill = state.selected_skill;

                Ok(())
            })?;
        }

        self.full_redraw = false;
        Ok(updated)
    }

    fn draw_text_overlay(
        &mut self,
        text_model: &SkillPanelTextModel,
        canvas: &mut Canvas<Window>,
    ) -> Result<()> {
        with_texture_canvas(canvas, &mut self.texture_font_overlay, |canvas| {
            if self.full_redraw {
                canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
                canvas.clear();
            }

            if self.full_redraw {
                draw_tile_label(canvas, &self.font_skills, 0, self.release_rate_min)?;
            }

            if text_model.release_rate != self.text_model.release_rate || self.full_redraw {
                draw_tile_label(canvas, &self.font_skills, 1, text_model.release_rate)?;
            }

            for i in 0..NUM_SKILLS {
                if text_model.remaining_skills[i] == self.text_model.remaining_skills[i]
                    && !self.full_redraw
                {
                    continue;
                }

                draw_tile_label(
                    canvas,
                    &self.font_skills,
                    i + 2,
                    text_model.remaining_skills[i],
                )?;
            }

            if text_model.lemmings_in != self.text_model.lemmings_in
                || text_model.lemmings_out != self.text_model.lemmings_out
                || text_model.cursor_state != self.text_model.cursor_state
                || text_model.remaining_time_seconds != self.text_model.remaining_time_seconds
                || self.full_redraw
            {
                format_stats(
                    &mut self.stats_new,
                    text_model,
                    self.lemmings_released_total,
                );

                draw_stats(
                    canvas,
                    &self.font,
                    &self.stats_current,
                    &self.stats_new,
                    self.full_redraw,
                )?;

                self.stats_current.clear();
                self.stats_current.push_str(&self.stats_new);
            }

            Ok(())
        })?;

        Ok(())
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

fn skill_name(skill: Profession) -> &'static str {
    match skill {
        Profession::Climber => "CLIMBER",
        Profession::Floater => "FLOATER",
        Profession::Bomber => "BOMBER",
        Profession::Blocker => "BLOCKER",
        Profession::Builder => "BUILDER",
        Profession::Basher => "BASHER",
        Profession::Miner => "MINER",
        Profession::Digger => "DIGGER",
        Profession::Faller => "FALLER",
        Profession::Walker => "WALKER",
    }
}

fn format_stats(str: &mut String, model: &SkillPanelTextModel, lemmings_released: usize) {
    str.clear();

    // 12 characters for the cursor
    if let Some(cursor_state) = &model.cursor_state {
        write!(
            str,
            "{:7} {:<2}  ",
            skill_name(cursor_state.leading_profession),
            cursor_state.lemming_count
        )
        .unwrap();
    } else {
        write!(str, "{:12}", "").unwrap();
    }

    // 10 characters for lemmings out
    write!(str, "OUT {:2}    ", model.lemmings_out).unwrap();

    // 9 characteres for lemmings in
    if lemmings_released == model.lemmings_in {
        write!(str, "IN 100%  ").unwrap();
    } else {
        write!(
            str,
            "IN {:2}%   ",
            (model.lemmings_in * 100)
                .checked_div(lemmings_released)
                .unwrap_or(0)
        )
        .unwrap();
    }

    // 9 characters for time
    write!(
        str,
        "TIME {:1}-{:0>2}",
        model.remaining_time_seconds / 60,
        model.remaining_time_seconds % 60
    )
    .unwrap();
}
