use rustlings::game_data::{Bitmap, NUM_SKILLS, Skill};

pub const MAX_LEMMING_COUNT: usize = 100;

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

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Activity {
    #[default]
    Climber,
    Floater,
    Blocker,
    Builder,
    Basher,
    Miner,
    Digger,
    Faller,
    Walker,
    Splatter,
    Drowner,
}

#[derive(Clone, Copy, PartialEq)]
pub struct CursorState {
    pub lemming_count: usize,
    pub leading_lemming: usize,
}

#[derive(Clone, Default)]
pub struct LemmingState {
    pub x: usize,
    pub y: usize,

    pub activity: Activity,
    pub frame: usize,

    pub countdown: Option<usize>,
    pub floater: bool,
    pub climber: bool,
}

#[derive(Clone, Default)]
pub struct SceneStateLevel {
    pub level_x: usize,
    pub terrain: Bitmap,
    pub object_state: Vec<ObjectState>,

    pub clock_msec: u64,
    pub simulation_clock_offset: i64,
    pub paused: bool,

    pub selected_skill: Skill,
    pub remaining_skills: [usize; NUM_SKILLS],

    pub lemmings_out_total: usize,
    pub lemmings_in: usize,
    pub release_rate: usize,

    pub remaining_time_seconds: usize,

    pub cursor_state: Option<CursorState>,

    pub lemmings: Vec<LemmingState>,
    pub lemming_count: usize,
    pub lemming_offset: usize,
}

#[derive(Default, Clone)]
pub enum SceneState {
    #[default]
    None,
    Level(SceneStateLevel),
}
