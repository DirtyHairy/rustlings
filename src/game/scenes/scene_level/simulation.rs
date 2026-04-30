use std::rc::Rc;

use anyhow::{Result, format_err};
use rustlings::game_data::{GameData, LEVEL_HEIGHT, Level, file::ground::InteractionType};

use crate::state::{
    Activity, ActivityStateFalling, LemmingAnimation, LemmingState, LevelState, SceneStateLevel,
};

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
    x: usize,
    y: usize,
}

pub struct Simulation {
    objects: Vec<Object>,
    entrances: Vec<usize>,
    released_total: usize,
}

const TICK_OPEN_ENTRANCES: u64 = 34;
const TICK_START_SPAWN: u64 = 44;

const SPAWN_COUNTDOWN_DEFAULT: usize = 20;
const SPAWN_X: usize = 23;
const SPAWN_Y: usize = 13;

impl Simulation {
    pub fn new(game_data: Rc<GameData>, level: &Level) -> Result<Self> {
        let objects = level
            .objects
            .iter()
            .map(|o| -> Result<Object> {
                let tileset = game_data
                    .tilesets
                    .get(level.graphics_set as usize)
                    .ok_or(format_err!("invalid tileset {}", level.graphics_set))?;
                let info = tileset
                    .object_info
                    .get(o.id as usize)
                    .ok_or(format_err!("invalid object {}", o.id))?;

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
                    x: o.x as usize,
                    y: o.y as usize,
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
            released_total: level.parameters.released as usize,
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
        let mut index = 0;
        while index < state.lemmings.len() {
            let kill = state.lemmings[index].tick();

            if kill {
                state.lemmings.remove(index);
            } else {
                index += 1;
            }
        }
    }

    fn tick_spawn(&self, state: &mut SceneStateLevel) {
        state.spawn_countdown = state.spawn_countdown.saturating_sub(1);
        if state.spawn_countdown > 0 {
            return;
        }

        let entrance = &self.objects[self.entrances[state.lemmings_out % self.entrances.len()]];

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
            state.spawn_countdown = (99_usize).saturating_sub(state.release_rate) / 2 + 4;
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
        let mut kill = match self.activity {
            Activity::Falling(mut activity_state) => self.tick_faller(&mut activity_state),
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

    fn tick_faller(&mut self, activity_state: &mut ActivityStateFalling) -> bool {
        self.frame = (self.frame + 1) % self.animation.frame_count();
        self.y += 3;

        activity_state.delta_y += 3;

        false
    }
}
