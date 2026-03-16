use rustlings::game_data::{Bitmap, NUM_SKILLS, Skill};

#[derive(Default, Clone)]
pub enum Screen {
    #[default]
    Level,
}

#[derive(Default, Clone)]
pub struct GameState {
    pub screen: Screen,
    pub current_level: usize,
}

#[derive(Clone, Default)]
pub struct ObjectState {
    pub triggered: bool,
    pub frame: usize,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Profession {
    Climber,
    Floater,
    Bomber,
    Blocker,
    Builder,
    Basher,
    Miner,
    Digger,
    Faller,
    Walker,
}

#[derive(Clone, Copy, PartialEq)]
pub struct CursorState {
    pub lemming_count: usize,
    pub leading_profession: Profession,
}

#[derive(Clone)]
pub struct SceneStateLevel {
    pub level_x: usize,
    pub terrain: Bitmap,
    pub object_state: Vec<ObjectState>,

    pub current_clock_msec: u64,

    pub selected_skill: Option<Skill>,
    pub remaining_skills: [usize; NUM_SKILLS],

    pub lemmings_out: usize,
    pub lemmings_in: usize,
    pub release_rate: usize,

    pub remaining_time_seconds: usize,

    pub cursor_state: Option<CursorState>,
}

#[derive(Default, Clone)]
pub enum SceneState {
    #[default]
    None,
    Level(SceneStateLevel),
}
