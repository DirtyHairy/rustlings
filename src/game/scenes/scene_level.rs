use crate::scene::Scene;
use crate::{
    geometry::Rect,
    state::{GameState, SceneState, SceneStateLevel},
};
use anyhow::Result;
use rustlings::{game_data::GameData, sdl_rendering::texture_from_bitmap};
use sdl3::{
    pixels::PixelFormat,
    render::{Canvas, Texture, TextureCreator},
    video::Window,
};
use std::rc::Rc;

pub struct SceneLevel<'texture_creator> {
    game_data: Rc<GameData>,
    game_state: GameState,
    state: SceneStateLevel,

    texture_screen: Texture<'texture_creator>,
    texture_skill_panel: Texture<'texture_creator>,
}

const TEXTURE_ID_MAIN_SCREEN: usize = 0;

impl<'texture_creator> SceneLevel<'texture_creator> {
    pub fn new<T>(
        game_data: Rc<GameData>,
        game_state: GameState,
        scene_state: SceneState,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        let texture_screen =
            texture_creator.create_texture_target(PixelFormat::RGBA8888, 320, 200)?;

        let texture_skill_panel = texture_from_bitmap(
            &game_data.skill_panel,
            &game_data.resolve_skill_panel_palette(0),
            texture_creator,
        )?;

        let state = match scene_state {
            SceneState::Level(state_level) => state_level,
            _ => Default::default(),
        };

        Ok(SceneLevel {
            game_data,
            game_state,
            state,
            texture_screen,
            texture_skill_panel,
        })
    }
}

impl<'texture_creator> Scene<'texture_creator> for SceneLevel<'texture_creator> {
    fn finish(self: Box<Self>) -> (GameState, SceneState) {
        (self.game_state, SceneState::Level(self.state))
    }

    fn width(&self) -> usize {
        320
    }

    fn height(&self) -> usize {
        200
    }

    fn aspect(&self) -> f32 {
        1.2
    }

    fn register_layers(&self, compositor: &mut dyn crate::scene::Compositor) {
        compositor.add_layer(TEXTURE_ID_MAIN_SCREEN as usize, Rect::new(0, 0, 320, 200));
    }

    fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<()> {
        let mut blit_result: Result<(), sdl3::Error> = Ok(());
        canvas.with_texture_canvas(&mut self.texture_screen, |canvas| {
            blit_result = canvas.copy(
                &self.texture_skill_panel,
                None,
                sdl3::rect::Rect::new(0, 160, 320, 40),
            );
        })?;

        blit_result.map_err(anyhow::Error::from)
    }

    fn texture(&mut self, id: usize) -> Result<&mut Texture<'texture_creator>> {
        match id {
            TEXTURE_ID_MAIN_SCREEN => Ok(&mut self.texture_screen),
            _ => Err(anyhow::format_err!("invalid texture id {}", id)),
        }
    }
}
