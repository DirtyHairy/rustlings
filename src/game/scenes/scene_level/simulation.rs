use std::rc::Rc;

use anyhow::Result;
use rustlings::game_data::{
    GameData, LEVEL_HEIGHT, LEVEL_WIDTH, Level, file::ground::InteractionType,
};

use crate::state::{
    Activity, Direction, LemmingAnimation, LemmingState, LevelState, SceneStateLevel, TerrainProps,
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

const MAX_SAFE_FALL: u32 = 57;
const FALL_DISTANCE_PER_FRAME: u32 = 3;

const MAX_STEP_UP: u32 = 2;
const MAX_JUMP: u32 = 6;
const MAX_STEP_DOWN: u32 = 3;
const JUMP_DISTANCE: u32 = 2;

const MIN_FOOT_Y: i32 = 5;

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
        let terrain_map = TerrainMap(&state.terrain_map);

        state
            .lemmings
            .retain_mut(|lemming| lemming.tick(&terrain_map));
    }

    fn tick_spawn(&self, state: &mut SceneStateLevel) {
        state.spawn_countdown = state.spawn_countdown.saturating_sub(1);
        if state.spawn_countdown > 0 {
            return;
        }

        let entrance =
            &self.objects[self.entrances[state.lemmings_out as usize % self.entrances.len()]];

        let mut lemming = LemmingState {
            x: (entrance.x + SPAWN_X) as i32,
            y: (entrance.y + SPAWN_Y) as i32,
            ..Default::default()
        };

        lemming.transition_to(Activity::Falling(Default::default()));

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
    fn tick(&mut self, terrain_map: &TerrainMap) -> bool {
        let keep = match &self.activity {
            Activity::Falling(_) => self.tick_faller(terrain_map),
            Activity::Walking(_) => self.tick_walker(terrain_map),
            Activity::Splatting => self.tick_splatter(),
            _ => true,
        };

        let keep = keep && self.y < (LEVEL_HEIGHT + self.animation.foot().1) as i32;

        keep
    }

    fn transition_to(&mut self, activity: Activity) {
        self.frame = 0;
        self.animation = activity.default_animation();
        self.activity = activity;
    }

    fn tick_faller(&mut self, terrain_map: &TerrainMap) -> bool {
        let mut transition_to: Option<Activity> = None;

        let keep = if let Activity::Falling(state) = &mut self.activity {
            let dy = terrain_map.delta_y_descend(self.x, self.y, 4);

            if dy <= FALL_DISTANCE_PER_FRAME {
                self.y += dy as i32;
                state.delta_y += dy;

                transition_to = Some(if state.delta_y <= MAX_SAFE_FALL {
                    Activity::Walking(Default::default())
                } else {
                    Activity::Splatting
                });
            } else {
                self.frame = (self.frame + 1) % self.animation.frame_count();

                self.y += FALL_DISTANCE_PER_FRAME as i32;
                state.delta_y += FALL_DISTANCE_PER_FRAME;
            }

            true
        } else {
            unreachable!()
        };

        if let Some(activity) = transition_to {
            self.transition_to(activity);
        }

        keep
    }

    fn tick_walker(&mut self, terrain_map: &TerrainMap) -> bool {
        let mut transition_to: Option<Activity> = None;

        if let Activity::Walking(state) = &mut self.activity {
            let old_y = self.y;

            if state.is_jumper {
                let dy = terrain_map.delta_y_ascend(self.x, self.y - 1, JUMP_DISTANCE + 1);

                if dy <= JUMP_DISTANCE {
                    self.y -= dy as i32;
                    state.is_jumper = false;
                } else {
                    self.y -= JUMP_DISTANCE as i32;
                }
            } else {
                match self.direction {
                    Direction::Left => self.x = (self.x - 1).max(0),
                    Direction::Right => self.x = (self.x + 1).min(LEVEL_WIDTH as i32 - 1),
                }

                if self.x == 0 || self.x == LEVEL_WIDTH as i32 - 1 {
                    self.direction = self.direction.invert();
                } else if terrain_map.is_solid(self.x, self.y) {
                    let dy = terrain_map.delta_y_ascend(self.x, self.y - 1, MAX_JUMP + 1);

                    if dy <= MAX_STEP_UP {
                        self.y -= dy as i32;
                    } else if dy <= MAX_JUMP {
                        state.is_jumper = true;
                        self.y -= JUMP_DISTANCE as i32;
                    } else {
                        self.direction = self.direction.invert();
                    }
                } else {
                    let dy = terrain_map.delta_y_descend(self.x, self.y, MAX_STEP_DOWN + 1);

                    if dy <= MAX_STEP_DOWN {
                        self.y += dy as i32;
                    } else {
                        transition_to = Some(Activity::Falling(Default::default()));
                    }
                }
            }

            if old_y > self.y && self.y < MIN_FOOT_Y as i32 {
                self.direction = self.direction.invert();
                state.is_jumper = false;
            }

            if state.is_jumper {
                self.animation = LemmingAnimation::Jumping;
                self.frame = 0;
            } else {
                self.animation = LemmingAnimation::Walking;
                self.frame = (self.frame + 1) % self.animation.frame_count();
            }
        } else {
            unreachable!()
        }

        if let Some(activity) = transition_to {
            self.transition_to(activity);
        }
        true
    }

    fn tick_splatter(&mut self) -> bool {
        self.frame = (self.frame + 1) % self.animation.frame_count();

        self.frame > 0
    }
}

struct TerrainMap<'a>(&'a Vec<TerrainProps>);

impl<'a> TerrainMap<'a> {
    pub fn is_solid(&self, x: i32, y: i32) -> bool {
        if y >= LEVEL_HEIGHT as i32 || y < 0 || x < 0 || x >= LEVEL_WIDTH as i32 {
            false
        } else {
            self.0[(x + y * LEVEL_WIDTH as i32) as usize].solid()
        }
    }

    fn delta_y_ascend(&self, x: i32, y: i32, limit: u32) -> u32 {
        let mut dy: u32 = 0;

        for i in 0..=limit {
            dy = i;
            let ypos = y as i32 - dy as i32;

            if !self.is_solid(x, ypos) {
                break;
            }
        }

        dy
    }

    fn delta_y_descend(&self, x: i32, y: i32, limit: u32) -> u32 {
        let mut dy: u32 = 0;

        for i in 0..=limit {
            dy = i;
            let ypos = y + dy as i32;

            if self.is_solid(x, ypos) {
                break;
            }
        }

        dy
    }
}

impl Activity {
    pub fn default_animation(&self) -> LemmingAnimation {
        match self {
            Activity::Bashing => LemmingAnimation::Bashing,
            Activity::Blocking => LemmingAnimation::Blocking,
            Activity::Building => LemmingAnimation::Building,
            Activity::Climbing => LemmingAnimation::Climbing,
            Activity::Falling(_) => LemmingAnimation::Falling,
            Activity::Digging => LemmingAnimation::Digging,
            Activity::Drowning => LemmingAnimation::Drowning,
            Activity::Exitting => LemmingAnimation::Exitting,
            Activity::Floating => LemmingAnimation::PreUmbrella,
            Activity::Frying => LemmingAnimation::Frying,
            Activity::Mining => LemmingAnimation::Mining,
            Activity::Splatting => LemmingAnimation::Splatting,
            Activity::Walking(_) => LemmingAnimation::Walking,
        }
    }
}
