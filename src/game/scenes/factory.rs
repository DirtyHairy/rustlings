use std::rc::Rc;

use anyhow::Result;
use rustlings::game_data::GameData;
use sdl3::{
    render::{Canvas, TextureCreator},
    video::Window,
};

use crate::{
    scene::Scene,
    scenes::{SceneBriefing, SceneDebrief, SceneLevel, SceneMainScreen},
    state::{self, GameState},
    timing::RenderConfiguration,
};

pub fn create_scene<'texture_creator, T>(
    game_data: Rc<GameData>,
    game_state: GameState,
    render_configuration: &RenderConfiguration,
    canvas: &mut Canvas<Window>,
    texture_creator: &'texture_creator TextureCreator<T>,
) -> Result<Box<dyn Scene<'texture_creator> + 'texture_creator>> {
    match game_state.scene {
        state::Scene::Level(state) => Ok(Box::new(SceneLevel::new(
            game_data,
            game_state.global,
            state,
            render_configuration,
            texture_creator,
        )?)),
        state::Scene::Briefing(state) => Ok(Box::new(SceneBriefing::new(
            game_data,
            game_state.global,
            state,
            render_configuration,
            canvas,
            texture_creator,
        )?)),
        state::Scene::Debrief(state) => Ok(Box::new(SceneDebrief::new(
            game_data,
            game_state.global,
            state,
            canvas,
            texture_creator,
        )?)),
        state::Scene::MainScreen(state) => Ok(Box::new(SceneMainScreen::new(
            game_data,
            game_state.global,
            state,
            render_configuration,
            canvas,
            texture_creator,
        )?)),
    }
}
