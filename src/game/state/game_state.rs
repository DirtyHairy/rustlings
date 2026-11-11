use crate::state::{FadeState, SceneStateBriefing, SceneStateDebrief, SceneStateLevel};

#[derive(Clone)]
pub enum Scene {
    Briefing(Option<(FadeState, Option<SceneStateBriefing>)>),
    Level(Option<(FadeState, Box<SceneStateLevel>)>),
    Debrief(Option<(FadeState, SceneStateDebrief)>),
}

#[derive(Clone, Default)]
pub struct GameStateGlobal {
    pub current_level: usize,
    pub last_saved_percent: u8,
    pub last_time_remaining_seconds: u32,
    pub code_salt: u8,
}

#[derive(Clone)]
pub struct GameState {
    pub global: GameStateGlobal,
    pub scene: Scene,
}
