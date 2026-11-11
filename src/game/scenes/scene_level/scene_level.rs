use std::{collections::VecDeque, rc::Rc};

use anyhow::Result;
use rustlings::game_data::{
    Bitmap, ENGINE_TICK_MSEC, GameData, LEVEL_HEIGHT, LEVEL_WIDTH, Level, LevelParameters,
    SCREEN_HEIGHT, SCREEN_WIDTH, Skill, file::ground::InteractionType,
};
use sdl3::{
    keyboard::{Keycode, Mod},
    render::{Canvas, Texture, TextureCreator},
    video::Window,
};

use crate::{
    scene::{CursorType, MouseButton, Scene, SceneEvent},
    scenes::{
        common::{FADEIN_MSEC_LEVEL, FADEOUT_MSEC_LEVEL, FadeController},
        scene_level::{
            cache::Cache,
            renderer::{Redraw, Renderer},
            selection_controller::{SelectionController, SelectionMode},
            simulation::{SelectionResult, Simulation},
            skill_panel_controller::SkillPanelController,
        },
    },
    state::{self, FadeState, GameStateGlobal, TerrainProps},
};
use crate::{
    scenes::scene_level::scroll_controller::ScrollController,
    state::{GameState, SceneStateLevel},
};

pub struct SceneLevel<'texture_creator> {
    global_state: GameStateGlobal,
    state: Box<SceneStateLevel>,

    renderer: Renderer<'texture_creator>,
    scroll_controller: ScrollController,
    skill_panel_controller: SkillPanelController,
    selection_controller: SelectionController,
    fade_controller: FadeController,
    simulation: Simulation,

    level_parameters: LevelParameters,
    clock_offset_msec: u64,

    shift_down: bool,
    right_mouse_down: bool,

    pause_tick_scheduled: bool,

    cache: Cache,
}

impl<'texture_creator> SceneLevel<'texture_creator> {
    pub fn new<T>(
        game_data: Rc<GameData>,
        global_state: GameStateGlobal,
        state: Option<(FadeState, Box<SceneStateLevel>)>,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        let level = game_data.resolve_level(global_state.current_level)?;
        let simulation = Simulation::new(Rc::clone(&game_data), &level)?;

        let (fade_state, scene_state) = match state {
            Some((fade_state, scene_state)) => (Some(fade_state), scene_state),
            None => {
                let mut scene_state = Box::from(SceneStateLevel {
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

                init_terrain_map(
                    &scene_state.terrain,
                    &level,
                    &game_data,
                    &mut scene_state.terrain_map,
                )?;
                simulation.initialize(&mut scene_state);

                (None, scene_state)
            }
        };

        let renderer = Renderer::new(&level, &scene_state, Rc::clone(&game_data), texture_creator)?;
        let skill_panel_controller = SkillPanelController::new(&level);
        let scroll_controller = ScrollController::new();
        let selection_controller = SelectionController::new();
        let fade_controller =
            FadeController::new(fade_state, FADEIN_MSEC_LEVEL, FADEOUT_MSEC_LEVEL);

        Ok(SceneLevel {
            clock_offset_msec: scene_state.clock_msec,
            global_state,
            state: scene_state,
            renderer,
            scroll_controller,
            selection_controller,
            skill_panel_controller,
            fade_controller,
            simulation,
            level_parameters: level.parameters,
            shift_down: false,
            right_mouse_down: false,
            pause_tick_scheduled: false,
            cache: Default::default(),
        })
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

    fn level_complete(&self) -> bool {
        let all_released = (self.state.lemmings_out == self.level_parameters.released)
            || self.state.armageddon.is_some();

        self.state.remaining_time_seconds == 0 || (all_released && self.state.lemmings.is_empty())
    }
}

impl<'texture_creator> Scene<'texture_creator> for SceneLevel<'texture_creator> {
    fn finish(self: Box<Self>) -> GameState {
        if self.fade_controller.fadeout_done() {
            GameState {
                global: GameStateGlobal {
                    last_saved_percent: ((self.state.lemmings_in * 100)
                        / self.level_parameters.released)
                        .min(100) as u8,
                    last_time_remaining_seconds: self.state.remaining_time_seconds,
                    ..self.global_state
                },
                scene: state::Scene::Debrief(None),
            }
        } else {
            GameState {
                global: self.global_state,
                scene: state::Scene::Level(Some((self.fade_controller.finish(), self.state))),
            }
        }
    }

    fn width(&self) -> u32 {
        SCREEN_WIDTH
    }

    fn height(&self) -> u32 {
        SCREEN_HEIGHT
    }

    fn opacity(&self) -> u8 {
        self.fade_controller.opacity()
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
            .selected_lemming_for_ui(self.selection_mode(), &mut self.cache)
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
                keycode: Keycode::Space,
                keymod,
                ..
            } if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) => {
                if self.state.paused {
                    self.pause_tick_scheduled = true;
                }
            }
            SceneEvent::KeyDown {
                keycode: Keycode::LShift | Keycode::RShift,
                keymod,
                ..
            } if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD | Mod::NOMOD) => {
                self.shift_down = true
            }
            SceneEvent::KeyUp {
                keycode: Keycode::LShift | Keycode::RShift,
                ..
            } => self.shift_down = false,
            SceneEvent::KeyDown {
                keycode: Keycode::Escape,
                ..
            } if !self.fade_controller.fadeout_started() => {
                self.fade_controller.start_fadeout();
            }

            SceneEvent::MouseDown(MouseButton::Right, _) => self.right_mouse_down = true,
            SceneEvent::MouseUp(MouseButton::Right, _) => self.right_mouse_down = false,

            SceneEvent::MouseDown(MouseButton::Left, _) => self.assign_skill(),
            SceneEvent::KeyDown {
                keycode: Keycode::Return,
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

    fn tick(&mut self, canvas: &mut Canvas<Window>, mut clock_msec: u64) -> Result<()> {
        self.fade_controller.tick(clock_msec);

        clock_msec += self.clock_offset_msec;

        let clock_msec_old = self.state.clock_msec;
        self.state.clock_msec = clock_msec;

        if self.state.paused {
            self.state.simulation_clock_offset += clock_msec_old as i64 - clock_msec as i64;
        }

        if clock_msec <= clock_msec_old {
            return Ok(());
        }

        if self
            .scroll_controller
            .tick(clock_msec, clock_msec_old, &mut self.state)
        {
            self.renderer.mark_for_redraw(Redraw::SCREEN);
        }

        let engine_ticks_old = clock_msec_old / ENGINE_TICK_MSEC;
        let engine_ticks = clock_msec / ENGINE_TICK_MSEC;

        for _ in engine_ticks_old..engine_ticks {
            if !self.state.paused || self.pause_tick_scheduled {
                self.renderer
                    .apply_diff(canvas, self.simulation.get_diff())?;
                self.simulation.clear_diff();

                self.simulation_tick();
                self.pause_tick_scheduled = false;
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

        if self.level_complete() && !self.fade_controller.fadeout_started() {
            self.fade_controller.start_fadeout();
        }

        Ok(())
    }

    fn next_tick_at_msec(&self) -> Option<u64> {
        let mut next_tick = ((self.state.clock_msec / ENGINE_TICK_MSEC) + 1) * ENGINE_TICK_MSEC;

        if let Some(next_tick_fade) = self.fade_controller.next_tick_at_msec() {
            next_tick = next_tick.min(next_tick_fade);
        }

        if let Some(next_tick_scroll) = self.scroll_controller.next_tick_at_msec(&self.state) {
            next_tick = next_tick.min(next_tick_scroll);
        }

        Some(next_tick.saturating_sub(self.clock_offset_msec))
    }

    fn register_layers(&self, compositor: &mut dyn crate::scene::Compositor) {
        self.renderer.register_layers(compositor);
    }

    fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<bool> {
        let selection_mode = self.selection_mode();
        self.renderer
            .draw(&self.state, &mut self.cache, selection_mode, canvas)
    }

    fn will_redraw(&self) -> bool {
        self.renderer.will_redraw()
    }

    fn texture(&mut self, id: usize) -> Result<&mut Texture<'texture_creator>> {
        self.renderer.texture(id)
    }

    fn is_complete(&self) -> bool {
        self.fade_controller.fadeout_done()
    }
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

    for steel_area in level.steel_areas.iter() {
        let steel_x = steel_area.x.max(0) as u32;
        let steel_y = steel_area.y.max(0) as u32;

        for y in steel_y..(steel_y + steel_area.height).min(LEVEL_HEIGHT) {
            for x in steel_x..(steel_x + steel_area.width).min(LEVEL_WIDTH) {
                terrain_map[(y * LEVEL_WIDTH + x) as usize].set_steel(true);
            }
        }
    }

    Ok(())
}
