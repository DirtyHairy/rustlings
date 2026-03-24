use rustlings::game_data::{Bitmap, NUM_SKILLS, Skill, file::main::LemmingSprite};

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
    Climbing,
    Floating,
    Blocking,
    Building,
    Bashing,
    Mining,
    Digging,
    Falling,
    Walking,
    Splatting,
    Drowning,
    Frying,
    Exitting,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Direction {
    #[default]
    Right,
    Left,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum LemmingAnimation {
    #[default]
    Walking,
    Jumping,
    Climbing,
    PostClimb,
    Building,
    Bashing,
    Mining,
    Falling,
    PreUmbrella,
    Umbrella,
    Shrugging, // last assymetric animation
    Exitting,  // first symmetric animation
    Frying,
    Blocking,
    OhNo,
    Explosion,
    Digging,
    Drowning,
    Splatting,
}

impl LemmingAnimation {
    pub const ALL: &[LemmingAnimation] = &[
        Self::Walking,
        Self::Jumping,
        Self::Climbing,
        Self::PostClimb,
        Self::Building,
        Self::Bashing,
        Self::Mining,
        Self::Falling,
        Self::PreUmbrella,
        Self::Umbrella,
        Self::Shrugging,
        Self::Exitting,
        Self::Frying,
        Self::Blocking,
        Self::OhNo,
        Self::Explosion,
        Self::Digging,
        Self::Drowning,
        Self::Splatting,
    ];

    pub fn mirror(self, direction: Direction) -> bool {
        direction == Direction::Left && (self as usize) < (Self::Exitting as usize)
    }

    pub fn sprite(self) -> LemmingSprite {
        match self {
            Self::Walking => LemmingSprite::WalkingR,
            Self::Jumping => LemmingSprite::JumpingR,
            Self::Climbing => LemmingSprite::ClimbingR,
            Self::PostClimb => LemmingSprite::PostClimbR,
            Self::Building => LemmingSprite::BuildingR,
            Self::Bashing => LemmingSprite::BashingR,
            Self::Mining => LemmingSprite::MiningR,
            Self::Falling => LemmingSprite::FallingR,
            Self::PreUmbrella => LemmingSprite::PreUmbrellaR,
            Self::Umbrella => LemmingSprite::UmbrellaR,
            Self::Exitting => LemmingSprite::Exitting,
            Self::Shrugging => LemmingSprite::ShruggingR,
            Self::Frying => LemmingSprite::Frying,
            Self::Blocking => LemmingSprite::Blocking,
            Self::OhNo => LemmingSprite::OhNo,
            Self::Explosion => LemmingSprite::Explosion,
            Self::Digging => LemmingSprite::Digging,
            Self::Drowning => LemmingSprite::Drowning,
            Self::Splatting => LemmingSprite::Splatting,
        }
    }
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
    pub direction: Direction,
    pub animation: LemmingAnimation,
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
