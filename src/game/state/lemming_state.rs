use std::{collections::VecDeque, ops::Not};

use crate::state::LemmingAnimation;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum LemmingHealth {
    #[default]
    Healthy,
    OhNo,
    Exploding,
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum Direction {
    #[default]
    Right,
    Left,
}

impl Not for Direction {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

impl Direction {
    pub fn delta(self, abs: u32) -> i32 {
        match self {
            Direction::Left => -(abs as i32),
            Direction::Right => abs as i32,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ActivityStateFalling {
    pub delta_y: u32,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ActivityStateFloating {
    pub tick: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ActivityStateDigging {
    pub newborn: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ActivityStateBuilding {
    pub bricks: u32,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub enum Activity {
    #[default]
    Climbing,
    Hoisting,
    Floating(ActivityStateFloating),
    Blocking,
    Building(ActivityStateBuilding),
    Shrugging,
    Bashing,
    Mining,
    Digging(ActivityStateDigging),
    Falling(ActivityStateFalling),
    Walking,
    Jumping,
    Splatting,
    Drowning,
    Frying,
    Exiting,
}

#[derive(Clone, Default)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct LemmingState {
    pub id: u32,

    pub x: i32,
    pub y: i32,

    pub activity: Activity,
    pub direction: Direction,
    pub animation: Option<LemmingAnimation>,
    pub particle_frame: Option<usize>,
    pub frame: usize,

    pub countdown: Option<u32>,
    pub health: LemmingHealth,
    pub floater: bool,
    pub climber: bool,

    pub armageddon: bool,
    pub ignore_blocker: bool,
}

pub type LemmingsState = VecDeque<LemmingState>;
