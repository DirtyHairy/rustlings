use crate::state::{FadeState, SceneStateBriefing, SceneStateLevel};

#[derive(Clone)]
pub enum Scene {
    Level(Option<(FadeState, Box<SceneStateLevel>)>),
    Briefing(Option<(FadeState, Option<SceneStateBriefing>)>),
}

#[derive(Clone)]
pub struct GameStateGlobal {
    pub current_level: usize,
}

#[derive(Clone)]
pub struct GameState {
    pub global: GameStateGlobal,
    pub scene: Scene,
}
