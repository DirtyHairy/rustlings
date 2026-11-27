use rustlings::game_data::DifficultyRating;
use strum::{EnumCount, FromRepr};

use crate::state::{
    FadeState, SceneStateBriefing, SceneStateDebrief, SceneStateLevel, SceneStateMainScreen,
};

#[derive(Clone)]
pub enum Scene {
    Briefing(Option<(FadeState, Option<SceneStateBriefing>)>),
    Level(Option<(FadeState, Box<SceneStateLevel>)>),
    Debrief(Option<(FadeState, SceneStateDebrief)>),
    MainScreen(Option<(FadeState, Box<SceneStateMainScreen>)>),
}

#[derive(Clone, Default, PartialEq, EnumCount, FromRepr, Copy)]
pub enum AudioSetting {
    #[default]
    Music,
    None,
    Fy,
}

impl AudioSetting {
    pub fn next(self) -> Self {
        Self::from_repr((self as usize + 1) % Self::COUNT).unwrap()
    }
}

#[derive(Clone, Default)]
pub struct GameStateGlobal {
    pub current_level: usize,
    pub last_saved_percent: u8,
    pub last_time_remaining_seconds: u32,
    pub code_salt: u8,

    pub difficulty: DifficultyRating,
    pub audio_setting: AudioSetting,
}

#[derive(Clone)]
pub struct GameState {
    pub global: GameStateGlobal,
    pub scene: Scene,
}
