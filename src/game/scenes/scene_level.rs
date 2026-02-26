use std::{cmp, rc::Rc};

use anyhow::Result;
use rustlings::{
    game_data::{GameData, LEVEL_HEIGHT, LEVEL_WIDTH},
    sdl_rendering::{texture_from_bitmap, with_texture_canvas},
};
use sdl3::{
    pixels::{Color, PixelFormat},
    rect::Rect as SdlRect,
    render::{Canvas, ScaleMode, Texture, TextureCreator},
    video::Window,
};

use crate::{
    geometry::Rect,
    state::{GameState, SceneState, SceneStateLevel},
};
use crate::{scene::Scene, scenes::level::ScrollController};

const SCREEN_WIDTH: usize = 320;
const SCREEN_HEIGHT: usize = 200;

const SKILL_PANEL_HEIGHT: usize = 40;

const MINIMAP_HEIGHT: usize = 18;
const MINIMAP_WIDTH: usize = 100;
const MINIMAP_RIGHT: usize = 8;
const MINIMAP_BOTTOM: usize = 3;

const TEXTURE_ID_MAIN_SCREEN: usize = 0;
const TEXTURE_ID_PREVIEW: usize = 1;

const ENGINE_TICK_MSEC: u64 = 66; // 15.15 FPS

const REDRAW_LEVEL: u32 = 0x01;
const REDRAW_SCREEN: u32 = 0x02;
const REDRAW_ALL: u32 = !0;

pub struct SceneLevel<'texture_creator> {
    game_data: Rc<GameData>,
    game_state: GameState,
    state: SceneStateLevel,

    redraw: u32,

    texture_terrain: Texture<'texture_creator>,
    texture_skill_panel: Texture<'texture_creator>,

    texture_level: Texture<'texture_creator>,
    texture_screen: Texture<'texture_creator>,

    scroll_controller: ScrollController,
}

impl<'texture_creator> SceneLevel<'texture_creator> {
    pub fn new<T>(
        game_data: Rc<GameData>,
        game_state: GameState,
        scene_state: SceneState,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        let level = game_data.resolve_level(21)?;
        let palette = game_data.resolve_palette(&level)?;

        let state = match scene_state {
            SceneState::Level(state_level) => state_level,
            _ => SceneStateLevel {
                level_x: level.start_x as usize,
                terrain: game_data.compose_terrain(&level)?,
                current_clock_msec: 0,
            },
        };

        let texture_terrain = texture_from_bitmap(&state.terrain, &palette, texture_creator)?;

        let texture_skill_panel = texture_from_bitmap(
            &game_data.skill_panel,
            &game_data.resolve_skill_panel_palette(0),
            texture_creator,
        )?;

        let texture_level = texture_creator.create_texture_target(
            PixelFormat::RGBA8888,
            LEVEL_WIDTH as u32,
            LEVEL_HEIGHT as u32,
        )?;

        let texture_screen = texture_creator.create_texture_target(
            PixelFormat::RGBA8888,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        )?;

        Ok(SceneLevel {
            game_data,
            game_state,
            state,
            texture_terrain,
            texture_skill_panel,
            texture_level,
            texture_screen,
            redraw: REDRAW_ALL,
            scroll_controller: ScrollController::new(),
        })
    }
}

impl<'texture_creator> Scene<'texture_creator> for SceneLevel<'texture_creator> {
    fn finish(mut self: Box<Self>) -> (GameState, SceneState) {
        self.state.current_clock_msec %= ENGINE_TICK_MSEC;

        (self.game_state, SceneState::Level(self.state))
    }

    fn width(&self) -> usize {
        SCREEN_WIDTH
    }

    fn height(&self) -> usize {
        SCREEN_HEIGHT
    }

    fn aspect(&self) -> f32 {
        1.2
    }

    fn dispatch_event(&mut self, event: crate::scene::SceneEvent) {
        self.scroll_controller.dispatch_event(event);
    }

    fn tick(&mut self, clock_msec: u64) {
        if clock_msec <= self.state.current_clock_msec {
            return;
        };

        if self.scroll_controller.tick(clock_msec, &mut self.state) {
            self.redraw |= REDRAW_SCREEN;
        }

        self.state.current_clock_msec = clock_msec;
    }

    fn next_tick_at_msec(&self) -> u64 {
        let next_tick_engine =
            ((self.state.current_clock_msec / ENGINE_TICK_MSEC) + 1) * ENGINE_TICK_MSEC;

        match self.scroll_controller.next_tick_at_msec(&self.state) {
            None => next_tick_engine,
            Some(next_tick_scroll) => cmp::min(next_tick_engine, next_tick_scroll),
        }
    }

    fn register_layers(&self, compositor: &mut dyn crate::scene::Compositor) {
        compositor.add_layer(
            TEXTURE_ID_MAIN_SCREEN,
            320,
            200,
            Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT),
        );

        compositor.add_layer(
            TEXTURE_ID_PREVIEW,
            LEVEL_WIDTH,
            LEVEL_HEIGHT,
            Rect::new(
                SCREEN_WIDTH - MINIMAP_RIGHT - MINIMAP_WIDTH,
                SCREEN_HEIGHT - MINIMAP_BOTTOM - MINIMAP_HEIGHT,
                MINIMAP_WIDTH,
                MINIMAP_HEIGHT,
            ),
        );
    }

    fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<bool> {
        let redraw = self.redraw;
        self.redraw = 0;

        if redraw == 0 {
            return Ok(false);
        }

        if redraw & REDRAW_LEVEL != 0 {
            with_texture_canvas(canvas, &mut self.texture_level, |canvas| -> Result<()> {
                canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
                canvas.clear();

                canvas
                    .copy(&self.texture_terrain, None, None)
                    .map_err(anyhow::Error::from)
            })?;
        }

        with_texture_canvas(canvas, &mut self.texture_screen, |canvas| -> Result<()> {
            canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
            canvas.clear();

            canvas.copy(
                &self.texture_skill_panel,
                None,
                SdlRect::new(
                    0,
                    LEVEL_HEIGHT as i32,
                    SCREEN_WIDTH as u32,
                    SKILL_PANEL_HEIGHT as u32,
                ),
            )?;

            self.texture_level.set_scale_mode(ScaleMode::Nearest);
            canvas.copy(
                &self.texture_level,
                SdlRect::new(
                    self.state.level_x as i32,
                    0,
                    SCREEN_WIDTH as u32,
                    LEVEL_HEIGHT as u32,
                ),
                SdlRect::new(0, 0, SCREEN_WIDTH as u32, LEVEL_HEIGHT as u32),
            )?;

            Ok(())
        })?;

        Ok(true)
    }

    fn texture(&mut self, id: usize) -> Result<&mut Texture<'texture_creator>> {
        match id {
            TEXTURE_ID_MAIN_SCREEN => Ok(&mut self.texture_screen),
            TEXTURE_ID_PREVIEW => Ok(&mut self.texture_level),
            _ => Err(anyhow::format_err!("invalid texture id {}", id)),
        }
    }
}
