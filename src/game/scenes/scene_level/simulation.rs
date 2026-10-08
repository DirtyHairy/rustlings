use std::rc::Rc;

use anyhow::Result;
use rustlings::game_data::{
    Bitmap, GameData, LEVEL_HEIGHT, Level, Skill, Sprite,
    file::{ground::InteractionType, main::MaskSprite},
};

use crate::{
    scenes::scene_level::terrain_diff::{DIG_LINE_WIDTH, TerrainDiff, TerrainDiffKind},
    state::{
        Activity, ActivityStateDigging, ActivityStateFalling, BlockField, Direction,
        LemmingAnimation, LemmingHealth, LemmingState, LevelState, ObjectState, SceneStateLevel,
        TerrainProps,
    },
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
    terrain_diff: Vec<TerrainDiff>,
    game_data: Rc<GameData>,
}

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub enum SelectionResult {
    Abort,
    Fallback,
    Success,
}

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(Debug))]
enum LemmingVerdict {
    Continue,
    Death,
    Exit,
}

const TERRAIN_DIFF_CAPACITY: usize = 99;

const TICK_OPEN_ENTRANCES: u64 = 36;
const TICK_START_SPAWN: u64 = 46;

const SPAWN_COUNTDOWN_DEFAULT: u32 = 10;
const SPAWN_X: u32 = 24;
const SPAWN_Y: u32 = 14;

const MAX_SAFE_FALL: u32 = 60;
const FALL_DISTANCE_PER_FRAME: u32 = 3;
const FALL_DISTANCE_START_OFFSET: u32 = 3;
const FALL_DISTANCE_FLOAT: u32 = 16;

const MAX_STEP_UP: u32 = 2;
const MAX_JUMP: u32 = 6;
const MAX_STEP_DOWN: u32 = 3;
const JUMP_DISTANCE: u32 = 2;

const DIG_X_OFFSET: i32 = -4;

const MIN_FOOT_Y: i32 = 5;
const CEILING_HIT_Y_RESET: i32 = MIN_FOOT_Y - 2;

const DROWNER_MIN_WALL_DISTANCE: u32 = 8;

const BOMBER_COUNTDOWN_TICKS: u32 = 79;

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
            terrain_diff: Vec::with_capacity(TERRAIN_DIFF_CAPACITY),
            game_data: Rc::clone(&game_data),
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

    pub fn assign_skill(
        &mut self,
        state: &mut SceneStateLevel,
        index: usize,
        skill: Skill,
    ) -> SelectionResult {
        let mut terrain = Terrain::new(
            &mut state.terrain,
            &mut state.terrain_map,
            &mut self.terrain_diff,
            &self.game_data.mask_sprites,
        );

        state.lemmings[index].assign_skill(&mut terrain, skill)
    }

    pub fn get_diff(&self) -> &[TerrainDiff] {
        &self.terrain_diff
    }

    pub fn clear_diff(&mut self) {
        self.terrain_diff.clear();
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

    fn tick_lemmings(&mut self, state: &mut SceneStateLevel) {
        let mut terrain = Terrain::new(
            &mut state.terrain,
            &mut state.terrain_map,
            &mut self.terrain_diff,
            &self.game_data.mask_sprites,
        );
        let mut lemmings_rescued: u32 = 0;

        state.lemmings.retain_mut(|lemming| {
            let verdict = lemming.tick(&mut terrain, &mut state.object_state);

            if verdict == LemmingVerdict::Exit {
                lemmings_rescued += 1;
            }

            verdict == LemmingVerdict::Continue
        });

        state.lemmings_in += lemmings_rescued;
    }

    fn tick_spawn(&self, state: &mut SceneStateLevel) {
        const PATTERNS: [[usize; 4]; 4] = [[0, 0, 0, 0], [0, 1, 1, 0], [0, 1, 2, 1], [0, 1, 2, 3]];

        state.spawn_countdown = state.spawn_countdown.saturating_sub(1);
        if state.spawn_countdown > 0 {
            return;
        }

        let entrance_index =
            PATTERNS[(self.entrances.len() - 1) % 4][state.lemmings_out as usize % 4];
        let entrance = &self.objects[self.entrances[entrance_index]];

        let mut lemming = LemmingState {
            id: state.lemmings_out,
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
            state.spawn_countdown = 99_u32.saturating_sub(state.release_rate) / 2 + 4;
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
    fn tick(&mut self, terrain: &mut Terrain, objects: &mut [ObjectState]) -> LemmingVerdict {
        self.tick_countdown();

        let mut verdict = match self.health {
            LemmingHealth::Healthy => match &self.activity {
                Activity::Falling(_) => self.tick_faller(terrain),
                Activity::Walking => self.tick_walker(terrain),
                Activity::Digging(_) => self.tick_digger(terrain),
                Activity::Blocking => self.tick_blocker(terrain),
                Activity::Splatting | Activity::Frying => self.tick_death(),
                Activity::Jumping => self.tick_jumper(terrain),
                Activity::Drowning => self.tick_drowner(terrain),
                Activity::Floating(_) => self.tick_floater(terrain),
                Activity::Exiting => self.tick_exiting(),
                _ => LemmingVerdict::Continue,
            },
            LemmingHealth::OhNo => self.tick_ohno(terrain),
            LemmingHealth::Exploding => self.tick_exploding(terrain),
        };

        self.turn_if_blocked(terrain);

        if verdict != LemmingVerdict::Death
            && (self.y >= (LEVEL_HEIGHT + self.animation.map_or_default(|a| a.foot().1)) as i32
                || !self.process_environment(terrain, objects))
        {
            verdict = LemmingVerdict::Death;
        }

        if verdict == LemmingVerdict::Death && matches!(self.activity, Activity::Blocking) {
            terrain.clear_block_field(self.x, self.y);
        }

        verdict
    }

    fn tick_countdown(&mut self) {
        let Some(countdown) = &mut self.countdown else {
            return;
        };

        if *countdown > 0 {
            *countdown -= 1;
        } else {
            self.countdown = None;

            self.set_health(match self.activity {
                Activity::Falling(_)
                | Activity::Floating(_)
                | Activity::Drowning
                | Activity::Frying => LemmingHealth::Exploding,
                _ => LemmingHealth::OhNo,
            });
        }
    }

    fn set_health(&mut self, health: LemmingHealth) {
        match health {
            LemmingHealth::Exploding => {
                self.animation = Some(LemmingAnimation::Explosion);
            }
            LemmingHealth::OhNo => {
                self.animation = Some(LemmingAnimation::OhNo);
            }
            _ => (),
        }

        self.frame = 0;
        self.health = health;
    }

    fn turn_if_blocked(&mut self, terrain: &Terrain) {
        let block_field = terrain
            .terrain_at(self.x, self.y)
            .unwrap_or_default()
            .block_field();

        match (block_field, self.direction) {
            (BlockField::Left, Direction::Right) => self.direction = Direction::Left,
            (BlockField::Right, Direction::Left) => self.direction = Direction::Right,
            _ => (),
        }
    }

    fn assign_skill(&mut self, terrain: &mut Terrain, skill: Skill) -> SelectionResult {
        if !self.supports_skill_tier1(terrain, skill) {
            return SelectionResult::Abort;
        }

        if !self.supports_skill_tier2(skill) {
            return skill_fallback_mode(skill);
        }

        if !self.supports_skill_tier3(skill) {
            return SelectionResult::Abort;
        }

        self.assign_skill_unchecked(terrain, skill);

        SelectionResult::Success
    }

    fn process_environment(&mut self, terrain: &Terrain, objects: &mut [ObjectState]) -> bool {
        let Some(terrain) = terrain.terrain_at(self.x, self.y) else {
            return true;
        };

        let mut keep = true;

        if terrain.trap() {
            let object_state = &mut objects[terrain.object_index() as usize];

            if !object_state.triggered {
                object_state.triggered = true;
                keep = false;
            }
        }

        if terrain.disintegrate() {
            self.transition_to(Activity::Frying);
        }

        if terrain.drown() {
            self.transition_to(Activity::Drowning);
        }

        if terrain.exit() && !matches!(self.activity, Activity::Falling(_)) {
            self.transition_to(Activity::Exiting);
        }

        keep
    }

    fn transition_to(&mut self, activity: Activity) {
        if self.activity == activity {
            return;
        }

        self.frame = match activity {
            Activity::Digging(_) => activity.default_animation().frame_count() - 1,
            _ => 0,
        };

        self.animation = Some(activity.default_animation());
        self.activity = activity;
    }

    fn tick_faller(&mut self, terrain: &Terrain) -> LemmingVerdict {
        let mut transition_to: Option<Activity> = None;

        let Activity::Falling(state) = &mut self.activity else {
            unreachable!();
        };

        let dy = terrain.delta_y_descend(self.x, self.y, FALL_DISTANCE_PER_FRAME);

        if dy < FALL_DISTANCE_PER_FRAME {
            self.y += dy as i32;

            transition_to = Some(if state.delta_y <= MAX_SAFE_FALL {
                Activity::Walking
            } else {
                Activity::Splatting
            });
        } else if self.floater && state.delta_y >= FALL_DISTANCE_FLOAT {
            self.transition_to(Activity::Floating(Default::default()));
        } else {
            self.frame = (self.frame + 1) % self.animation.unwrap().frame_count();

            self.y += FALL_DISTANCE_PER_FRAME as i32;
            state.delta_y += FALL_DISTANCE_PER_FRAME;
        }

        if let Some(activity) = transition_to {
            self.transition_to(activity);
        }

        LemmingVerdict::Continue
    }

    fn tick_floater(&mut self, terrain: &Terrain) -> LemmingVerdict {
        const FRAME_PATTERN: &[usize] = &[1, 2, 3, 3, 2, 1, 0, 0];
        let mut transition_to: Option<Activity> = None;

        let Activity::Floating(state) = &mut self.activity else {
            unreachable!();
        };

        let descent: i32 = match state.tick {
            0..=2 => {
                self.frame += 1;
                3
            }
            3 => {
                self.animation = Some(LemmingAnimation::Umbrella);
                self.frame = 1;
                3
            }
            4 => -1,
            5 => 0,
            6 | 7 => 1,
            _ => {
                self.frame = FRAME_PATTERN[(state.tick - 8) as usize % FRAME_PATTERN.len()];
                2
            }
        };

        state.tick += 1;

        let dy = terrain.delta_y_descend(self.x, self.y, 4) as i32;

        if dy <= descent {
            self.y += dy;

            transition_to = Some(Activity::Walking);
        } else {
            self.y += descent;
        }

        if let Some(activity) = transition_to {
            self.transition_to(activity);
        }

        LemmingVerdict::Continue
    }

    fn tick_jumper(&mut self, terrain: &Terrain) -> LemmingVerdict {
        let old_y = self.y;
        let dy = terrain.delta_y_ascend(self.x, self.y - 1, JUMP_DISTANCE + 1);

        if dy < JUMP_DISTANCE {
            self.y -= dy as i32;
            self.transition_to(Activity::Walking);
        } else {
            self.y -= JUMP_DISTANCE as i32;
        }

        if old_y > self.y {
            self.turn_if_ceiling();
        }

        LemmingVerdict::Continue
    }

    fn tick_walker(&mut self, terrain: &Terrain) -> LemmingVerdict {
        let old_y = self.y;
        self.frame = (self.frame + 1) % self.animation.unwrap().frame_count();

        self.x += self.direction.delta(1);

        if self.x == 0 || self.x == terrain.width() as i32 - 1 {
            self.direction = !self.direction;
        } else if terrain.is_solid(self.x, self.y) {
            let dy = terrain.delta_y_ascend(self.x, self.y - 1, MAX_JUMP + 1);

            if dy <= MAX_STEP_UP {
                self.y -= dy as i32;
            } else if dy <= MAX_JUMP {
                self.transition_to(Activity::Jumping);
                self.y -= JUMP_DISTANCE as i32;
            } else {
                self.direction = !self.direction;
            }
        } else {
            let dy = terrain.delta_y_descend(self.x, self.y, MAX_STEP_DOWN + 1);
            self.y += dy as i32;

            if dy > MAX_STEP_DOWN {
                self.transition_to(Activity::Falling(Default::default()));
            }
        }

        if old_y > self.y {
            self.turn_if_ceiling();
        }

        LemmingVerdict::Continue
    }

    fn tick_digger(&mut self, terrain: &mut Terrain) -> LemmingVerdict {
        let Activity::Digging(state) = &mut self.activity else {
            unreachable!();
        };

        let newborn = state.newborn;
        state.newborn = false;

        if newborn {
            terrain.dig(self.x + DIG_X_OFFSET, self.y - 2);
            terrain.dig(self.x + DIG_X_OFFSET, self.y - 1);
        }

        if self.frame == 15 || self.frame == 7 {
            let y = self.y;
            self.y += 1;

            if !terrain.dig(self.x + DIG_X_OFFSET, y) {
                self.transition_to(Activity::Falling(Default::default()));
            } else if terrain.is_steel(self.x, self.y) {
                self.transition_to(Activity::Walking);
            }
        }

        if matches!(self.activity, Activity::Digging(_)) {
            self.frame = (self.frame + 1) % self.animation.unwrap().frame_count();
        }

        LemmingVerdict::Continue
    }

    fn tick_blocker(&mut self, terrain: &mut Terrain) -> LemmingVerdict {
        if terrain.is_solid(self.x, self.y) {
            self.frame = (self.frame + 1) % self.animation.unwrap().frame_count();
        } else {
            terrain.clear_block_field(self.x, self.y);
            self.transition_to(Activity::Walking);
        }

        LemmingVerdict::Continue
    }

    fn turn_if_ceiling(&mut self) {
        if self.y < MIN_FOOT_Y {
            self.direction = !self.direction;
            self.y = CEILING_HIT_Y_RESET;

            if let Activity::Jumping = self.activity {
                self.transition_to(Activity::Walking);
            }
        }
    }

    fn tick_death(&mut self) -> LemmingVerdict {
        self.frame = (self.frame + 1) % self.animation.unwrap().frame_count();

        if self.frame > 0 {
            LemmingVerdict::Continue
        } else {
            LemmingVerdict::Death
        }
    }

    fn tick_drowner(&mut self, terrain: &Terrain) -> LemmingVerdict {
        self.frame = (self.frame + 1) % self.animation.unwrap().frame_count();

        if !terrain.is_solid(
            self.x + self.direction.delta(DROWNER_MIN_WALL_DISTANCE),
            self.y,
        ) {
            self.x += self.direction.delta(1);
        }

        if self.frame > 0 {
            LemmingVerdict::Continue
        } else {
            LemmingVerdict::Death
        }
    }

    fn tick_exiting(&mut self) -> LemmingVerdict {
        self.frame = (self.frame + 1) % self.animation.unwrap().frame_count();

        if self.frame > 0 {
            LemmingVerdict::Continue
        } else {
            LemmingVerdict::Exit
        }
    }

    fn tick_ohno(&mut self, terrain: &Terrain) -> LemmingVerdict {
        if self.frame == self.animation.unwrap().frame_count() - 1 {
            self.set_health(LemmingHealth::Exploding);
        } else {
            self.frame += 1;

            let dy = terrain.delta_y_descend(self.x, self.y, FALL_DISTANCE_PER_FRAME);
            self.y += dy as i32;
        }

        LemmingVerdict::Continue
    }

    fn tick_exploding(&mut self, terrain: &mut Terrain) -> LemmingVerdict {
        if self.frame == self.animation.unwrap().frame_count() - 1 {
            terrain.explode(self.x, self.y);
            LemmingVerdict::Death
        } else {
            self.frame += 1;
            LemmingVerdict::Continue
        }
    }

    fn assign_skill_unchecked(&mut self, terrain: &mut Terrain, skill: Skill) {
        match skill {
            Skill::Floater => self.floater = true,
            Skill::Climber => self.climber = true,
            Skill::Basher => self.transition_to(Activity::Bashing),
            Skill::Blocker => {
                self.transition_to(Activity::Blocking);
                terrain.create_block_field(self.x, self.y);
            }
            Skill::Bomber => self.countdown = Some(BOMBER_COUNTDOWN_TICKS),
            Skill::Builder => self.transition_to(Activity::Building),
            Skill::Digger => self.transition_to(Activity::Digging(Default::default())),
            Skill::Miner => self.transition_to(Activity::Mining),
        }
    }

    fn supports_skill_tier1(&self, terrain: &Terrain, skill: Skill) -> bool {
        match skill {
            Skill::Digger => !terrain.is_steel(self.x, self.y),
            Skill::Blocker => !terrain.block_field_overlaps(self.x, self.y),
            _ => true,
        }
    }

    fn supports_skill_tier2(&self, skill: Skill) -> bool {
        match skill {
            Skill::Floater if self.floater => return false,
            Skill::Climber if self.climber => return false,
            Skill::Bomber if self.countdown.is_some() => return false,
            _ => (),
        };

        match self.health {
            LemmingHealth::Exploding => false,
            LemmingHealth::OhNo => matches!(skill, Skill::Climber | Skill::Floater),
            LemmingHealth::Healthy => self.activity.supports_skill(skill),
        }
    }

    fn supports_skill_tier3(&self, _skill: Skill) -> bool {
        // handle steel / terrain rejection for bashers and miners
        true
    }
}

struct Terrain<'a> {
    bitmap: &'a mut Bitmap,
    map: &'a mut [TerrainProps],
    diff: &'a mut Vec<TerrainDiff>,
    masks_sprites: &'a [Sprite],
}

impl<'a> Terrain<'a> {
    fn new(
        bitmap: &'a mut Bitmap,
        map: &'a mut [TerrainProps],
        diff: &'a mut Vec<TerrainDiff>,
        masks_sprites: &'a [Sprite],
    ) -> Self {
        assert!((bitmap.width * bitmap.height) as usize == map.len());

        Self {
            bitmap,
            map,
            diff,
            masks_sprites,
        }
    }

    fn width(&self) -> u32 {
        self.bitmap.width
    }

    fn height(&self) -> u32 {
        self.bitmap.height
    }

    fn terrain_at(&self, x: i32, y: i32) -> Option<TerrainProps> {
        if y >= self.bitmap.height as i32 || y < 0 || x < 0 || x >= self.bitmap.width as i32 {
            None
        } else {
            Some(self.map[(x + y * self.bitmap.width as i32) as usize])
        }
    }

    fn is_solid(&self, x: i32, y: i32) -> bool {
        self.terrain_at(x, y)
            .map(|terrain_info| terrain_info.solid())
            .unwrap_or(false)
    }

    fn is_steel(&self, x: i32, y: i32) -> bool {
        self.terrain_at(x, y)
            .map(|terrain_info| terrain_info.steel())
            .unwrap_or(false)
    }

    fn delta_y_ascend(&self, x: i32, y: i32, limit: u32) -> u32 {
        let mut dy: u32 = 0;

        for i in 0..=limit {
            dy = i;
            let ypos = y - dy as i32;

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

    fn dig(&mut self, x: i32, y: i32) -> bool {
        let terrain_width = self.width();
        let terrain_height = self.height();

        if y >= terrain_height as i32 || y < 0 {
            return false;
        }

        let x_start = x.max(0);
        let x_end = x
            .saturating_add_unsigned(DIG_LINE_WIDTH)
            .min(terrain_width as i32);

        if x_end < 0 {
            return false;
        }

        let extend_start = (y * terrain_width as i32 + x_start) as usize;
        let extend_end = (y * terrain_width as i32 + x_end) as usize;

        if !self.map[extend_start..extend_end].iter().any(|t| t.solid()) {
            return false;
        }

        self.map[extend_start..extend_end]
            .iter_mut()
            .for_each(|terrain_prop| terrain_prop.set_solid(false));

        self.map[extend_start..extend_end].fill(TerrainProps::new());
        self.bitmap.data[extend_start..extend_end].fill(0);
        self.bitmap.transparency[extend_start..extend_end].fill(false);

        self.diff.push(TerrainDiff {
            x,
            y,
            kind: TerrainDiffKind::Dig,
        });

        true
    }

    fn explode(&mut self, x: i32, y: i32) {
        self.apply_mask(x - 8, y - 14, MaskSprite::Explosion, 0);
        self.diff.push(TerrainDiff {
            x: x - 8,
            y: y - 14,
            kind: TerrainDiffKind::Mask(MaskSprite::Explosion, 0),
        });
    }

    fn block_field_overlaps(&self, x: i32, y: i32) -> bool {
        let terrain_width = self.width();

        let Some((x_left, x_right, y_top, y_bottom)) = self.block_field_bounds(x, y) else {
            return false;
        };

        for iy in y_top..y_bottom {
            let extend_start = iy as usize * terrain_width as usize + x_left as usize;
            let extend_end = iy as usize * terrain_width as usize + x_right as usize;

            if self.map[extend_start..extend_end]
                .iter()
                .any(|t| t.block_field() != BlockField::None)
            {
                return true;
            }
        }

        false
    }

    fn clear_block_field(&mut self, x: i32, y: i32) {
        let terrain_width = self.width();

        let Some((x_left, x_right, y_top, y_bottom)) = self.block_field_bounds(x, y) else {
            return;
        };

        for iy in y_top..y_bottom {
            let extend_start = iy as usize * terrain_width as usize + x_left as usize;
            let extend_end = iy as usize * terrain_width as usize + x_right as usize;

            self.map[extend_start..extend_end].iter_mut().for_each(|t| {
                t.set_block_field(BlockField::None);
            });
        }
    }

    fn create_block_field(&mut self, x: i32, y: i32) {
        let terrain_height = self.height();

        let x_left = (x - 4) & !0x03;

        let y_top = ((y - 6) & !0x03).max(0);
        let y_bottom = ((y + 6) & !0x03).min(terrain_height as i32);

        if y_bottom < 0 || y_top >= terrain_height as i32 {
            return;
        }

        for iy in y_top..y_bottom {
            (x_left..x_left + 4).for_each(|ix| self.set_block_field_at(ix, iy, BlockField::Left));
            (x_left + 4..x_left + 8)
                .for_each(|ix| self.set_block_field_at(ix, iy, BlockField::Center));
            (x_left + 8..x_left + 12)
                .for_each(|ix| self.set_block_field_at(ix, iy, BlockField::Right));
        }
    }

    fn set_block_field_at(&mut self, x: i32, y: i32, block_field: BlockField) {
        let terrain_width = self.width();
        let terrain_height = self.height();

        if x < 0 || x >= terrain_width as i32 || y < 0 || y >= terrain_height as i32 {
            return;
        }

        self.map[y as usize * terrain_width as usize + x as usize].set_block_field(block_field);
    }

    fn block_field_bounds(&self, x: i32, y: i32) -> Option<(u32, u32, u32, u32)> {
        let terrain_width = self.width();
        let terrain_height = self.height();

        let x_left = ((x - 4) & !0x03).max(0);
        let x_right = ((x + 8) & !0x03).min(terrain_width as i32);

        if x_right < 0 || x_left >= terrain_width as i32 {
            return None;
        }

        let y_top = ((y - 6) & !0x03).max(0);
        let y_bottom = ((y + 6) & !0x03).min(terrain_height as i32);

        if y_bottom < 0 || y_top >= terrain_height as i32 {
            return None;
        }

        Some((x_left as u32, x_right as u32, y_top as u32, y_bottom as u32))
    }

    fn apply_mask(&mut self, x: i32, y: i32, sprite: MaskSprite, frame: usize) {
        let sprite = &self.masks_sprites[sprite as usize];

        let terrain_width = self.width();
        let terrain_height = self.height();

        if x + (sprite.width as i32) < 0
            || x >= terrain_width as i32
            || y + (sprite.height as i32) < 0
            || y >= terrain_height as i32
        {
            return;
        }

        let x_eff = x.max(0) as u32;
        let y_eff = y.max(0) as u32;
        let offset_x = (x_eff as i32 - x) as u32;
        let offset_y = (y_eff as i32 - y) as u32;
        let width_eff = (x + sprite.width as i32).min(terrain_width as i32) as u32 - x_eff;
        let height_eff = (y + sprite.height as i32).min(terrain_height as i32) as u32 - y_eff;
        let frame = &sprite.frames[frame];

        for iy in 0..height_eff {
            for ix in 0..width_eff {
                if frame.transparency[((offset_y + iy) * sprite.width + offset_x + ix) as usize] {
                    let index = ((y_eff + iy) * terrain_width + x_eff + ix) as usize;

                    self.map[index].set_solid(false);
                    self.bitmap.data[index] = 0;
                }
            }
        }
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
            Activity::Digging(_) => LemmingAnimation::Digging,
            Activity::Drowning => LemmingAnimation::Drowning,
            Activity::Exiting => LemmingAnimation::Exiting,
            Activity::Floating(_) => LemmingAnimation::PreUmbrella,
            Activity::Frying => LemmingAnimation::Frying,
            Activity::Mining => LemmingAnimation::Mining,
            Activity::Splatting => LemmingAnimation::Splatting,
            Activity::Walking => LemmingAnimation::Walking,
            Activity::Jumping => LemmingAnimation::Jumping,
        }
    }

    pub fn supports_skill(&self, skill: Skill) -> bool {
        match self {
            Activity::Bashing => skill != Skill::Basher,
            Activity::Blocking => skill == Skill::Bomber,
            // caveat: a shrugger can be assigned everything -> needs modelling
            Activity::Building => skill != Skill::Builder,
            // caveat: a hoister can be assigned everything -> needs modelling
            Activity::Climbing => matches!(skill, Skill::Bomber | Skill::Floater),
            Activity::Falling(_) => {
                matches!(skill, Skill::Bomber | Skill::Floater | Skill::Climber)
            }
            Activity::Digging(_) => skill != Skill::Digger,
            Activity::Drowning => matches!(skill, Skill::Climber | Skill::Floater | Skill::Bomber),
            Activity::Exiting => matches!(skill, Skill::Climber | Skill::Floater | Skill::Bomber),
            Activity::Floating(_) => matches!(skill, Skill::Climber | Skill::Bomber),
            Activity::Frying => matches!(skill, Skill::Climber | Skill::Floater),
            Activity::Mining => skill != Skill::Miner,
            Activity::Splatting => false,
            Activity::Walking => true,
            Activity::Jumping => matches!(skill, Skill::Climber | Skill::Floater | Skill::Bomber),
        }
    }
}

fn skill_fallback_mode(skill: Skill) -> SelectionResult {
    if matches!(
        skill,
        Skill::Basher | Skill::Miner | Skill::Digger | Skill::Builder
    ) {
        SelectionResult::Fallback
    } else {
        SelectionResult::Abort
    }
}

impl Default for ActivityStateFalling {
    fn default() -> Self {
        Self {
            delta_y: FALL_DISTANCE_START_OFFSET,
        }
    }
}

impl Default for ActivityStateDigging {
    fn default() -> Self {
        Self { newborn: true }
    }
}

#[path = "./simulation_test/mod.rs"]
#[cfg(test)]
mod test;
