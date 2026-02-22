use crate::{
    scene::Scene,
    scenes::SceneLevel,
    state::{GameState, SceneState, Screen},
};
use anyhow::Result;
use rustlings::game_data::GameData;
use sdl3::render::TextureCreator;
use std::rc::Rc;

pub fn create_scene<'game_state, 'texture_creator: 'game_state, T>(
    game_data: Rc<GameData>,
    game_state: &'game_state mut GameState,
    scene_state: &SceneState,
    texture_creator: &'texture_creator TextureCreator<T>,
) -> Result<Box<dyn Scene<'texture_creator> + 'game_state>> {
    match game_state.screen {
        Screen::Level => Ok(Box::from(SceneLevel::new(
            game_data,
            game_state,
            scene_state,
            texture_creator,
        )?)),
    }
}
