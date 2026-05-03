use std::rc::Rc;

use anyhow::Result;
use rustlings::game_data::{GameData, LEVEL_HEIGHT, Level, file::ground::InteractionType};

use crate::state::{Activity, LemmingAnimation, LemmingState, LevelState, SceneStateLevel};

#[derive(PartialEq, Clone, Copy)]
enum AnimationType {
    Triggered,
    Loop,
    Static,
}

struct Object {
    animation_type: AnimationType,
    interaction_type: InteractionType,
    animation_start: usize,
    last_frame: usize,
    x: u32,
    y: u32,
}

pub struct Simulation {
    objects: Vec<Object>,
    entrances: Vec<usize>,
    released_total: u32,
}

const TICK_OPEN_ENTRANCES: u64 = 34;
const TICK_START_SPAWN: u64 = 44;

const SPAWN_COUNTDOWN_DEFAULT: u32 = 20;
const SPAWN_X: u32 = 24;
const SPAWN_Y: u32 = 14;

impl Simulation {
    pub fn new(game_data: Rc<GameData>, level: &Level) -> Result<Self> {
        let objects = level
            .objects
            .iter()
            .map(|o| -> Result<Object> {
                let info = game_data.resolve_object(o.id as usize, level.graphics_set as usize)?;

                let animation_type = if info.animation_end == 1 {
                    AnimationType::Static
                } else if info.animation_loops {
                    AnimationType::Loop
                } else {
                    AnimationType::Triggered
                };

                Ok(Object {
                    animation_type,
                    interaction_type: info.interaction_type,
                    animation_start: info.animation_start,
                    last_frame: info.animation_end.saturating_sub(1),
                    x: o.x as u32,
                    y: o.y as u32,
                })
            })
            .collect::<Result<Vec<Object>>>()?;

        let entrances: Vec<usize> = objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.interaction_type == InteractionType::Entrance)
            .map(|(i, _)| i)
            .collect();

        Ok(Self {
            objects,
            entrances,
            released_total: level.parameters.released,
        })
    }

    pub fn initialize(&self, state: &mut SceneStateLevel) {
        for (i, object) in self.objects.iter().enumerate() {
            let object_state = &mut state.object_state[i];

            object_state.triggered = false;
            object_state.frame = if object.interaction_type == InteractionType::Entrance {
                object.animation_start
            } else {
                0
            };
        }
    }

    pub fn tick(&mut self, state: &mut SceneStateLevel) {
        let current_tick = state.tick;
        state.tick += 1;

        match current_tick {
            TICK_OPEN_ENTRANCES => {
                state.level_state = LevelState::Open;
                self.open_entrances(state);
            }
            TICK_START_SPAWN => {
                state.level_state = LevelState::Spawn;
                state.spawn_countdown = SPAWN_COUNTDOWN_DEFAULT
            }
            _ => (),
        }

        if state.level_state == LevelState::Spawn {
            self.tick_spawn(state);
        }

        self.tick_lemmings(state);

        self.tick_objects(state);
    }

    fn open_entrances(&self, state: &mut SceneStateLevel) {
        for (i, object) in self.objects.iter().enumerate() {
            if object.interaction_type != InteractionType::Entrance {
                continue;
            }

            let object_state = &mut state.object_state[i];

            object_state.triggered = true;
            object_state.frame = object.animation_start;
        }
    }

    fn tick_lemmings(&self, state: &mut SceneStateLevel) {
        state.lemmings.retain_mut(|lemming| !lemming.tick());
    }

    fn tick_spawn(&self, state: &mut SceneStateLevel) {
        state.spawn_countdown = state.spawn_countdown.saturating_sub(1);
        if state.spawn_countdown > 0 {
            return;
        }

        let entrance =
            &self.objects[self.entrances[state.lemmings_out as usize % self.entrances.len()]];

        let mut lemming = LemmingState {
            x: entrance.x + SPAWN_X,
            y: entrance.y + SPAWN_Y,
            ..Default::default()
        };

        lemming.start_falling();

        state.lemmings.push_back(lemming);
        state.lemmings_out += 1;

        if state.lemmings_out == self.released_total {
            state.level_state = LevelState::Late;
        } else {
            state.spawn_countdown = 99u32.saturating_sub(state.release_rate) / 2 + 4;
        }
    }

    fn tick_objects(&self, state: &mut SceneStateLevel) {
        for (i, object) in self.objects.iter().enumerate() {
            let object_state = &mut state.object_state[i];

            if object.animation_type == AnimationType::Loop
                || (object.animation_type == AnimationType::Triggered && object_state.triggered)
            {
                object_state.frame += 1;
                if object_state.frame > object.last_frame {
                    object_state.frame = 0;
                    object_state.triggered = false;
                }
            }
        }
    }
}

impl LemmingState {
    pub fn tick(&mut self) -> bool {
        let mut kill = match &self.activity {
            Activity::Falling(_) => self.tick_faller(),
            _ => false,
        };

        kill = kill || self.y >= LEVEL_HEIGHT + self.animation.foot().1;

        kill
    }

    pub fn start_falling(&mut self) {
        self.activity = Activity::Falling(Default::default());
        self.animation = LemmingAnimation::Falling;
        self.frame = 0;
    }

    fn tick_faller(&mut self) -> bool {
        if let Activity::Falling(state) = &mut self.activity {
            self.frame = (self.frame + 1) % self.animation.frame_count();
            self.y += 3;

            state.delta_y += 3;

            false
        } else {
            unreachable!()
        }
    }
}
