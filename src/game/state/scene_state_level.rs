use bitfield_struct::bitfield;
use rustlings::game_data::{Bitmap, Skill};
use strum::{EnumCount, FromRepr};

use crate::state::{LemmingsState, ObjectsState};

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct Selection {
    pub lemming_count: u32,
    pub primary_lemming: Option<u32>,
    pub secondary_lemming: Option<u32>,
    pub secondary_lemming_stale: bool,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum LevelState {
    #[default]
    Intro,
    Open,
    Spawn,
    Late,
}

#[repr(u8)]
#[derive(FromRepr, Debug, PartialEq)]
pub enum BlockField {
    None = 0,
    Left = 1,
    Right = 2,
    Center = 3,
}

impl BlockField {
    const fn into_bits(self) -> u8 {
        self as u8
    }

    const fn from_bits(value: u8) -> Self {
        match Self::from_repr(value) {
            Some(x) => x,
            None => Self::None,
        }
    }
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
    #[bits(2)]
    pub block_field: BlockField,
    #[bits(6)]
    pub object_index: u8,
}

#[derive(Clone, Default)]
pub struct ArmageddonState {
    pub nuking: bool,
}

#[derive(Clone, Default)]
pub struct SceneStateLevel {
    pub level_state: LevelState,

    pub level_x: u32,
    pub terrain: Bitmap,
    pub terrain_map: Vec<TerrainProps>,
    pub object_state: ObjectsState,

    pub clock_msec: u64,
    pub simulation_clock_offset: i64,
    pub paused: bool,
    pub tick: u64,

    pub selected_skill: Skill,
    pub remaining_skills: [u32; Skill::COUNT],

    pub lemmings_out: u32,
    pub lemmings_in: u32,
    pub release_rate: u32,

    pub remaining_time_seconds: u32,

    pub selection: Selection,

    pub lemmings: LemmingsState,

    pub spawn_countdown: u32,

    pub armageddon: Option<ArmageddonState>,
}
