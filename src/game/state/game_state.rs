use crate::state::SceneStateLevel;

#[derive(Clone)]
pub enum Scene {
    Level(Option<Box<SceneStateLevel>>),
    Briefing,
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
