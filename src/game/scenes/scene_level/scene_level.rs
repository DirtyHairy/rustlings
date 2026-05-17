use std::{cmp, collections::VecDeque, rc::Rc};

use anyhow::Result;
use rustlings::game_data::{
    Bitmap, GameData, LEVEL_HEIGHT, LEVEL_WIDTH, Level, LevelParameters, NUM_LEVELS, SCREEN_HEIGHT,
    SCREEN_WIDTH, Skill, decode_level_index, file::ground::InteractionType,
};
use sdl3::{
    keyboard::{Keycode, Mod},
    render::{Canvas, Texture, TextureCreator},
    video::Window,
};

use crate::{
    code::code_for_level,
    scene::{CursorType, MouseButton, Scene, SceneEvent},
    scenes::scene_level::{
        cache::Cache,
        renderer::{Redraw, Renderer},
        selection_controller::{SelectionController, SelectionMode},
        simulation::{SelectionResult, Simulation},
        skill_panel_controller::SkillPanelController,
    },
    state::TerrainProps,
};
use crate::{
    scenes::scene_level::scroll_controller::ScrollController,
    state::{GameState, SceneState, SceneStateLevel},
};

const ENGINE_TICK_MSEC: u64 = 1000 / 17; // 17 FPS
const FADE_IN_MSEC: u64 = 1000;

#[derive(PartialEq)]
enum Status {
    Running,
    DoneNextLevel,
    DonePreviousLevel,
}

pub struct SceneLevel<'texture_creator> {
    game_state: GameState,
    state: Box<SceneStateLevel>,
    status: Status,

    renderer: Renderer<'texture_creator>,
    scroll_controller: ScrollController,
    skill_panel_controller: SkillPanelController,
    selection_controller: SelectionController,
    simulation: Simulation,

    level_parameters: LevelParameters,
    clock_offset_msec: u64,

    last_draw_at_clock_msec: u64,
    fast: bool,

    shift_down: bool,
    right_mouse_down: bool,

    cache: Cache,
}

impl<'texture_creator> SceneLevel<'texture_creator> {
    pub fn new<T>(
        game_data: Rc<GameData>,
        game_state: GameState,
        scene_state: SceneState,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        let level = game_data.resolve_level(game_state.current_level)?;
        print_level(game_state.current_level, &level);

        let simulation = Simulation::new(Rc::clone(&game_data), &level)?;

        let state = match scene_state {
            SceneState::Level(state_level) => state_level,
            _ => {
                let mut state = Box::from(SceneStateLevel {
                    level_x: level.start_x,
                    terrain: game_data.compose_terrain(&level)?,
                    terrain_map: vec![Default::default(); (LEVEL_WIDTH * LEVEL_HEIGHT) as usize],
                    object_state: vec![Default::default(); level.objects.len()],
                    lemmings: VecDeque::with_capacity(level.parameters.released as usize),
                    remaining_skills: level.parameters.skills,
                    release_rate: level.parameters.release_rate,
                    remaining_time_seconds: level.parameters.time_limit * 60,
                    ..Default::default()
                });

                init_terrain_map(&state.terrain, &level, &game_data, &mut state.terrain_map)?;
                simulation.initialize(&mut state);

                state
            }
        };

        let renderer = Renderer::new(&level, &state, Rc::clone(&game_data), texture_creator)?;
        let skill_panel_controller = SkillPanelController::new(&level);
        let scroll_controller = ScrollController::new();
        let selection_controller = SelectionController::new();

        Ok(SceneLevel {
            clock_offset_msec: state.clock_msec,
            game_state,
            state,
            status: Status::Running,
            renderer,
            scroll_controller,
            selection_controller,
            skill_panel_controller,
            simulation,
            level_parameters: level.parameters,
            last_draw_at_clock_msec: 0,
            fast: false,
            shift_down: false,
            right_mouse_down: false,
            cache: Default::default(),
        })
    }

    fn engine_tick_msec(&self) -> u64 {
        if self.fast {
            ENGINE_TICK_MSEC >> 2
        } else {
            ENGINE_TICK_MSEC
        }
    }

    fn fade_in_msec(&self) -> u64 {
        if self.fast {
            FADE_IN_MSEC >> 2
        } else {
            FADE_IN_MSEC
        }
    }

    fn selection_mode(&self) -> SelectionMode {
        if self.shift_down || self.right_mouse_down {
            SelectionMode::Secondary
        } else {
            SelectionMode::Primary
        }
    }

    fn simulation_tick(&mut self) {
        self.simulation.tick(&mut self.state);
        self.cache.clear_selection();

        self.selection_controller
            .update(&mut self.state, &mut self.cache);

        self.renderer
            .mark_for_redraw(Redraw::LEVEL | Redraw::SKILL_PANEL);
    }

    fn assign_skill(&mut self) {
        if self.state.paused {
            return;
        }

        let skill = self.state.selected_skill;

        if self.state.remaining_skills[skill as usize] == 0 {
            return;
        }

        let selection_result = match self.selection_mode() {
            SelectionMode::Primary => self.try_assign_skill_with_fallback(skill),
            SelectionMode::Secondary => self.try_assign_skill(skill, SelectionMode::Secondary),
        };

        if selection_result == SelectionResult::Success {
            self.state.remaining_skills[self.state.selected_skill as usize] -= 1
        }
    }

    fn try_assign_skill_with_fallback(&mut self, skill: Skill) -> SelectionResult {
        let mut selection_result = self.try_assign_skill(skill, SelectionMode::Primary);
        if selection_result == SelectionResult::Fallback {
            selection_result = self.try_assign_skill(skill, SelectionMode::Secondary);
        }

        selection_result
    }

    fn try_assign_skill(&mut self, skill: Skill, selection_mode: SelectionMode) -> SelectionResult {
        let Some(lemming_index) = self.state.selected_lemming(selection_mode, &mut self.cache)
        else {
            return SelectionResult::Fallback;
        };

        self.simulation
            .assign_skill(&mut self.state, lemming_index, skill)
    }
}

impl<'texture_creator> Scene<'texture_creator> for SceneLevel<'texture_creator> {
    fn finish(mut self: Box<Self>) -> (GameState, SceneState) {
        match self.status {
            Status::Running => (self.game_state, SceneState::Level(self.state)),
            Status::DoneNextLevel => {
                self.game_state.current_level = (self.game_state.current_level + 1) % NUM_LEVELS;
                (self.game_state, SceneState::None)
            }
            Status::DonePreviousLevel => {
                self.game_state.current_level =
                    (self.game_state.current_level + NUM_LEVELS - 1) % NUM_LEVELS;
                (self.game_state, SceneState::None)
            }
        }
    }

    fn width(&self) -> u32 {
        SCREEN_WIDTH
    }

    fn height(&self) -> u32 {
        SCREEN_HEIGHT
    }

    fn aspect(&self) -> f32 {
        1.2
    }

    fn opacity(&self) -> u8 {
        ((self.state.clock_msec * 255) / self.fade_in_msec()).min(255) as u8
    }

    fn set_is_fullscreen(&mut self, is_fullscreen: bool) {
        self.scroll_controller.set_is_fullscreen(is_fullscreen);
    }

    fn set_mouse_enabled(&mut self, mouse_enabled: bool) {
        self.scroll_controller.set_mouse_enabled(mouse_enabled);
    }

    fn cursor_type(&mut self) -> CursorType {
        if self
            .state
            .selected_lemming(self.selection_mode(), &mut self.cache)
            .is_some()
        {
            CursorType::Box
        } else {
            CursorType::Crosshair
        }
    }

    fn dispatch_event(&mut self, event: SceneEvent) {
        if self
            .selection_controller
            .dispatch_event(event, &mut self.state, &mut self.cache)
        {
            self.renderer.mark_for_redraw(Redraw::SKILL_PANEL);
        }

        match event {
            SceneEvent::KeyDown {
                keycode: Keycode::PageDown,
                keymod: Mod::NOMOD,
                ..
            } => {
                if self.status == Status::Running {
                    self.status = Status::DonePreviousLevel;
                }
            }
            SceneEvent::KeyDown {
                keycode: Keycode::PageUp,
                keymod: Mod::NOMOD,
                ..
            } => {
                if self.status == Status::Running {
                    self.status = Status::DoneNextLevel;
                }
            }

            SceneEvent::KeyDown {
                keycode: Keycode::Space,
                keymod: Mod::LSHIFTMOD | Mod::RSHIFTMOD,
                ..
            } => {
                if self.state.paused {
                    self.simulation_tick();
                }
            }
            SceneEvent::KeyDown {
                keycode: Keycode::Space,
                keymod: Mod::LCTRLMOD,
                ..
            } => self.fast = true,
            SceneEvent::KeyUp {
                keycode: Keycode::Space,
                ..
            } => self.fast = false,

            SceneEvent::KeyDown {
                keycode: Keycode::LShift | Keycode::RShift,
                keymod: Mod::LSHIFTMOD | Mod::RSHIFTMOD | Mod::NOMOD,
                ..
            } => self.shift_down = true,
            SceneEvent::KeyUp {
                keycode: Keycode::LShift | Keycode::RShift,
                ..
            } => self.shift_down = false,

            SceneEvent::MouseDown(MouseButton::Right, _) => self.right_mouse_down = true,
            SceneEvent::MouseUp(MouseButton::Right, _) => self.right_mouse_down = false,

            SceneEvent::MouseDown(MouseButton::Left, _) => self.assign_skill(),
            SceneEvent::KeyDown {
                keycode: Keycode::Return,
                keymod: Mod::NOMOD,
                ..
            } => self.assign_skill(),

            _ => (),
        };

        if self
            .scroll_controller
            .dispatch_event(event, &mut self.state)
        {
            self.renderer.mark_for_redraw(Redraw::SCREEN);
        }

        if self
            .skill_panel_controller
            .dispatch_event(event, &mut self.state)
        {
            self.renderer.mark_for_redraw(Redraw::SKILL_PANEL);
        }
    }

    fn tick(&mut self, mut clock_msec: u64) {
        clock_msec += self.clock_offset_msec;

        let clock_msec_old = self.state.clock_msec;
        self.state.clock_msec = clock_msec;

        if self.state.paused {
            self.state.simulation_clock_offset += clock_msec_old as i64 - clock_msec as i64;
        }

        if clock_msec <= clock_msec_old {
            return;
        }

        if self
            .scroll_controller
            .tick(clock_msec, clock_msec_old, &mut self.state)
        {
            self.renderer.mark_for_redraw(Redraw::SCREEN);
        }

        let engine_ticks_old = clock_msec_old / self.engine_tick_msec();
        let engine_ticks = clock_msec / self.engine_tick_msec();

        for _ in engine_ticks_old..engine_ticks {
            if !self.state.paused {
                self.simulation_tick();
            }

            if self.skill_panel_controller.tick(&mut self.state) {
                self.renderer.mark_for_redraw(Redraw::SKILL_PANEL);
            }
        }

        let remaining_time_seconds = (self.level_parameters.time_limit * 60).saturating_sub(
            ((clock_msec as i64 + self.state.simulation_clock_offset).max(0) / 1000) as u32,
        );

        if remaining_time_seconds != self.state.remaining_time_seconds {
            self.state.remaining_time_seconds = remaining_time_seconds;
            self.renderer.mark_for_redraw(Redraw::SKILL_PANEL);
        }
    }

    fn next_tick_at_msec(&self) -> u64 {
        let next_tick_engine =
            ((self.state.clock_msec / self.engine_tick_msec()) + 1) * self.engine_tick_msec();

        match self.scroll_controller.next_tick_at_msec(&self.state) {
            None => next_tick_engine,
            Some(next_tick_scroll) => cmp::min(next_tick_engine, next_tick_scroll),
        }
        .saturating_sub(self.clock_offset_msec)
    }

    fn register_layers(&self, compositor: &mut dyn crate::scene::Compositor) {
        self.renderer.register_layers(compositor);
    }

    fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<bool> {
        let last_draw_at_clock_msec = self.last_draw_at_clock_msec;
        self.last_draw_at_clock_msec = self.state.clock_msec;

        let selection_mode = self.selection_mode();
        self.renderer
            .draw(&self.state, &mut self.cache, selection_mode, canvas)
            .map(|updated| {
                updated
                    | (self.state.clock_msec != last_draw_at_clock_msec
                        && self.state.clock_msec <= self.fade_in_msec())
            })
    }

    fn will_redraw(&self) -> bool {
        self.renderer.will_redraw()
    }

    fn texture(&mut self, id: usize) -> Result<&mut Texture<'texture_creator>> {
        self.renderer.texture(id)
    }

    fn is_complete(&self) -> bool {
        self.status != Status::Running
    }
}

fn print_level(current_level: usize, level: &Level) {
    let (difficulty, index) = decode_level_index(current_level);
    println!();
    println!("{} {}", difficulty, index);
    println!(
        "code: {}",
        code_for_level(current_level, None, None).unwrap_or("[invalid]".into())
    );
    println!("{}", level);
}

fn init_terrain_map(
    terrain: &Bitmap,
    level: &Level,
    game_data: &GameData,
    terrain_map: &mut [TerrainProps],
) -> Result<()> {
    for (i, pixel) in terrain_map.iter_mut().enumerate() {
        pixel.set_solid(!terrain.transparency[i]);
    }

    for (index, object) in level.objects.iter().enumerate() {
        let object_info =
            game_data.resolve_object(object.id as usize, level.graphics_set as usize)?;

        let object_x = (object.x + object_info.trigger_left).max(0) as u32;
        let object_y = (object.y + object_info.trigger_top).max(0) as u32;
        let object_width = object_info.trigger_width;
        let object_height = object_info.trigger_height;

        for y in object_y..(object_y + object_height).min(LEVEL_HEIGHT) {
            for x in object_x..(object_x + object_width).min(LEVEL_WIDTH) {
                let props = &mut terrain_map[(y * LEVEL_WIDTH + x) as usize];

                match object_info.interaction_type {
                    InteractionType::Disintegrate => props.set_disintegrate(true),
                    InteractionType::Drown => props.set_drown(true),
                    InteractionType::Exit => props.set_exit(true),
                    InteractionType::OneWayLeft => props.set_one_way_left(true),
                    InteractionType::OneWayRight => props.set_one_way_right(true),
                    InteractionType::Trap => {
                        props.set_trap(true);
                        props.set_object_index(index as u8);
                    }
                    InteractionType::Entrance | InteractionType::None => (),
                }
            }
        }
    }

    Ok(())
}
