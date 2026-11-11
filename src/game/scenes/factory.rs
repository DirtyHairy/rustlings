use std::rc::Rc;

use anyhow::Result;
use rustlings::game_data::GameData;
use sdl3::render::TextureCreator;

use crate::{
    scene::Scene,
    scenes::{SceneBriefing, SceneDebrief, SceneLevel},
    state::{self, GameState},
};

pub fn create_scene<'texture_creator, T>(
    game_data: Rc<GameData>,
    game_state: GameState,
    texture_creator: &'texture_creator TextureCreator<T>,
) -> Result<Box<dyn Scene<'texture_creator> + 'texture_creator>> {
    match game_state.scene {
        state::Scene::Level(state) => Ok(Box::new(SceneLevel::new(
            game_data,
            game_state.global,
            state,
            texture_creator,
        )?)),
        state::Scene::Briefing(state) => Ok(Box::new(SceneBriefing::new(
            game_data,
            game_state.global,
            state,
            texture_creator,
        )?)),
        state::Scene::Debrief(state) => Ok(Box::new(SceneDebrief::new(
            game_data,
            game_state.global,
            state,
            texture_creator,
        )?)),
    }
}
