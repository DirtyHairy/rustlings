use std::collections::VecDeque;

use bitfield_struct::bitfield;
use rustlings::game_data::{Bitmap, NUM_SKILLS, Skill};

use crate::state::LemmingAnimation;

#[derive(Clone, Default)]
pub struct ObjectState {
    pub triggered: bool,
    pub frame: usize,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub struct ActivityStateFalling {
    pub delta_y: u32,
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

#[derive(Clone, Copy, PartialEq)]
pub struct CursorState {
    pub lemming_count: u32,
    pub leading_lemming: u32,
}

#[derive(Clone, Copy, Default)]
pub struct LemmingState {
    pub x: u32,
    pub y: u32,

    pub activity: Activity,
    pub direction: Direction,
    pub animation: LemmingAnimation,
    pub frame: usize,

    pub countdown: Option<u32>,
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

#[bitfield(u16)]
pub struct TerrainProps {
    pub solid: bool,
    pub steel: bool,
    pub one_way_left: bool,
    pub one_way_right: bool,
    pub exit: bool,
    pub drown: bool,
    pub disintegrate: bool,
    pub trap: bool,
    pub object_index: u8,
}

#[derive(Clone, Default)]
pub struct SceneStateLevel {
    pub level_state: LevelState,

    pub level_x: u32,
    pub terrain: Bitmap,
    pub terrain_map: Vec<TerrainProps>,
    pub object_state: Vec<ObjectState>,

    pub clock_msec: u64,
    pub simulation_clock_offset: i64,
    pub paused: bool,
    pub tick: u64,

    pub selected_skill: Skill,
    pub remaining_skills: [u32; NUM_SKILLS],

    pub lemmings_out: u32,
    pub lemmings_in: u32,
    pub release_rate: u32,

    pub remaining_time_seconds: u32,

    pub cursor_state: Option<CursorState>,

    pub lemmings: VecDeque<LemmingState>,

    pub spawn_countdown: u32,
}
