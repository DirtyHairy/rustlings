use std::rc::Rc;

use anyhow::Result;
use rustlings::{
    game_data::{Bitmap, GameData, LEVEL_HEIGHT, LEVEL_WIDTH, Level},
    sdl_rendering::{texture_from_bitmap, with_texture_canvas},
};
use sdl3::{
    pixels::PixelFormat,
    rect::Rect as SdlRect,
    render::{Canvas, ScaleMode, Texture, TextureCreator},
    video::Window,
};

use crate::scene::Scene;
use crate::{
    geometry::Rect,
    state::{GameState, SceneState, SceneStateLevel},
};

const SCREEN_WIDTH: usize = 320;
const SCREEN_HEIGHT: usize = 200;

const SKILL_PANEL_HEIGHT: usize = 40;

const MINIMAP_HEIGHT: usize = 18;
const MINIMAP_WIDTH: usize = 100;
const MINIMAP_RIGHT: usize = 8;
const MINIMAP_BOTTOM: usize = 3;

const TEXTURE_ID_MAIN_SCREEN: usize = 0;
const TEXTURE_ID_PREVIEW: usize = 1;

pub struct SceneLevel<'texture_creator> {
    game_data: Rc<GameData>,
    game_state: GameState,
    state: SceneStateLevel,

    level: Level,
    terrain: Bitmap,

    texture_terrain: Texture<'texture_creator>,
    texture_skill_panel: Texture<'texture_creator>,

    texture_level: Texture<'texture_creator>,
    texture_screen: Texture<'texture_creator>,
}

impl<'texture_creator> SceneLevel<'texture_creator> {
    pub fn new<T>(
        game_data: Rc<GameData>,
        game_state: GameState,
        scene_state: SceneState,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        let level = game_data.resolve_level(21)?;
        let terrain = game_data.compose_terrain(&level)?;
        let palette = game_data.resolve_palette(&level)?;

        let texture_terrain = texture_from_bitmap(&terrain, &palette, texture_creator)?;

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

        let state = match scene_state {
            SceneState::Level(state_level) => state_level,
            _ => SceneStateLevel {
                level_x: level.start_x as usize,
                ..Default::default()
            },
        };

        Ok(SceneLevel {
            game_data,
            game_state,
            state,
            level,
            terrain,
            texture_terrain,
            texture_skill_panel,
            texture_level,
            texture_screen,
        })
    }
}

impl<'texture_creator> Scene<'texture_creator> for SceneLevel<'texture_creator> {
    fn finish(self: Box<Self>) -> (GameState, SceneState) {
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

    fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<()> {
        with_texture_canvas(canvas, &mut self.texture_level, |canvas| -> Result<()> {
            canvas
                .copy(&self.texture_terrain, None, None)
                .map_err(anyhow::Error::from)
        })?;

        with_texture_canvas(canvas, &mut self.texture_screen, |canvas| -> Result<()> {
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

        Ok(())
    }

    fn texture(&mut self, id: usize) -> Result<&mut Texture<'texture_creator>> {
        match id {
            TEXTURE_ID_MAIN_SCREEN => Ok(&mut self.texture_screen),
            TEXTURE_ID_PREVIEW => Ok(&mut self.texture_level),
            _ => Err(anyhow::format_err!("invalid texture id {}", id)),
        }
    }
}
