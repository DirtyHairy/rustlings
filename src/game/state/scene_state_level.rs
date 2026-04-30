use std::collections::VecDeque;

use rustlings::game_data::{
    Bitmap, NUM_SKILLS, Skill,
    file::main::{LEMMING_SPRITE_LAYOUT, LemmingSprite},
};

#[derive(Clone, Default)]
pub struct ObjectState {
    pub triggered: bool,
    pub frame: usize,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub struct ActivityStateFalling {
    pub delta_y: usize,
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
    Falling(ActivityStateFalling),
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
    Hoisting,
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
        Self::Hoisting,
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

    pub const fn foot(self) -> (usize, usize) {
        match self {
            Self::Digging => (7, 11),
            Self::Explosion => (15, 24),
            _ => (
                LEMMING_SPRITE_LAYOUT[self as usize].1 / 2 - 1,
                LEMMING_SPRITE_LAYOUT[self as usize].2 - 1,
            ),
        }
    }

    pub const fn frame_count(self) -> usize {
        LEMMING_SPRITE_LAYOUT[self as usize].0
    }

    pub fn mirror(self, direction: Direction) -> bool {
        direction == Direction::Left && (self as usize) < (Self::Exitting as usize)
    }

    pub fn sprite(self) -> LemmingSprite {
        match self {
            Self::Walking => LemmingSprite::WalkingR,
            Self::Jumping => LemmingSprite::JumpingR,
            Self::Climbing => LemmingSprite::ClimbingR,
            Self::Hoisting => LemmingSprite::HoistingR,
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

#[derive(Clone, Copy, Default)]
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

#[derive(Clone, Copy, PartialEq, Default)]
pub enum LevelState {
    #[default]
    Intro,
    Open,
    Spawn,
    Late,
}

#[derive(Clone, Default)]
pub struct SceneStateLevel {
    pub level_state: LevelState,

    pub level_x: usize,
    pub terrain: Bitmap,
    pub object_state: Vec<ObjectState>,

    pub clock_msec: u64,
    pub simulation_clock_offset: i64,
    pub paused: bool,
    pub tick: u64,

    pub selected_skill: Skill,
    pub remaining_skills: [usize; NUM_SKILLS],

    pub lemmings_out: usize,
    pub lemmings_in: usize,
    pub release_rate: usize,

    pub remaining_time_seconds: usize,

    pub cursor_state: Option<CursorState>,

    pub lemmings: VecDeque<LemmingState>,

    pub spawn_countdown: usize,
}
