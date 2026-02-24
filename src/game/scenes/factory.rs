use std::rc::Rc;

use anyhow::Result;
use rustlings::game_data::GameData;
use sdl3::render::TextureCreator;

use crate::{
    scene::Scene,
    scenes::SceneLevel,
    state::{GameState, SceneState, Screen},
};

pub fn create_scene<'texture_creator, T>(
    game_data: Rc<GameData>,
    game_state: GameState,
    scene_state: SceneState,
    texture_creator: &'texture_creator TextureCreator<T>,
) -> Result<Box<dyn Scene<'texture_creator> + 'texture_creator>> {
    match game_state.screen {
        Screen::Level => Ok(Box::from(SceneLevel::new(
            game_data,
            game_state,
            scene_state,
            texture_creator,
        )?)),
    }
}
