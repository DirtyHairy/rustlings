use std::{cmp, rc::Rc};

use anyhow::Result;
use rustlings::game_data::{
    GameData, Level, LevelParameters, NUM_LEVELS, SCREEN_HEIGHT, SCREEN_WIDTH, decode_level_index,
};
use sdl3::{
    keyboard::{Keycode, Mod},
    render::{Canvas, Texture, TextureCreator},
    video::Window,
};

use crate::{
    code::code_for_level,
    scene::{CursorType, Scene, SceneEvent},
    scenes::scene_level::{
        renderer::{Redraw, Renderer},
        simulation::Simulation,
        skill_panel_controller::SkillPanelController,
    },
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
    state: SceneStateLevel,
    status: Status,

    renderer: Renderer<'texture_creator>,
    scroll_controller: ScrollController,
    skill_panel_controller: SkillPanelController,
    simulation: Simulation,

    level_parameters: LevelParameters,
    clock_offset_msec: u64,

    last_draw_at_clock_msec: u64,
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
                let mut state = SceneStateLevel {
                    level_x: level.start_x as usize,
                    terrain: game_data.compose_terrain(&level)?,
                    object_state: vec![Default::default(); level.objects.len()],
                    lemmings: vec![Default::default(); level.parameters.released as usize],
                    remaining_skills: level.parameters.skills.map(|x| x as usize),
                    release_rate: level.parameters.release_rate as usize,
                    remaining_time_seconds: level.parameters.time_limit as usize * 60,
                    ..Default::default()
                };

                simulation.initialize(&mut state);
                state
            }
        };

        let renderer = Renderer::new(&level, &state, Rc::clone(&game_data), texture_creator)?;
        let skill_panel_controller = SkillPanelController::new(&level);
        let scroll_controller = ScrollController::new();

        Ok(SceneLevel {
            clock_offset_msec: state.clock_msec,
            game_state,
            state,
            status: Status::Running,
            renderer,
            scroll_controller,
            skill_panel_controller,
            simulation,
            level_parameters: level.parameters,
            last_draw_at_clock_msec: 0,
        })
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

    fn width(&self) -> usize {
        SCREEN_WIDTH
    }

    fn height(&self) -> usize {
        SCREEN_HEIGHT
    }

    fn aspect(&self) -> f32 {
        1.2
    }

    fn opacity(&self) -> u8 {
        ((self.state.clock_msec * 255) / FADE_IN_MSEC).min(255) as u8
    }

    fn set_is_fullscreen(&mut self, is_fullscreen: bool) {
        self.scroll_controller.set_is_fullscreen(is_fullscreen);
    }

    fn set_mouse_enabled(&mut self, mouse_enabled: bool) {
        self.scroll_controller.set_mouse_enabled(mouse_enabled);
    }

    fn cursor_type(&self) -> CursorType {
        CursorType::Crosshair
    }

    fn dispatch_event(&mut self, event: SceneEvent) {
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
            _ => {
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

        let engine_ticks_old = clock_msec_old / ENGINE_TICK_MSEC;
        let engine_ticks = clock_msec / ENGINE_TICK_MSEC;

        for _ in engine_ticks_old..engine_ticks {
            if !self.state.paused {
                self.simulation.tick(&mut self.state);
                self.renderer.mark_for_redraw(Redraw::LEVEL);
                self.renderer.mark_for_redraw(Redraw::SKILL_PANEL);
            }

            if self.skill_panel_controller.tick(&mut self.state) {
                self.renderer.mark_for_redraw(Redraw::SKILL_PANEL);
            }
        }

        let remaining_time_seconds = ((self.level_parameters.time_limit * 60) as usize)
            .saturating_sub(
                ((clock_msec as i64 + self.state.simulation_clock_offset).max(0) / 1000) as usize,
            );

        if remaining_time_seconds != self.state.remaining_time_seconds {
            self.state.remaining_time_seconds = remaining_time_seconds;
            self.renderer.mark_for_redraw(Redraw::SKILL_PANEL);
        }
    }

    fn next_tick_at_msec(&self) -> u64 {
        let next_tick_engine = ((self.state.clock_msec / ENGINE_TICK_MSEC) + 1) * ENGINE_TICK_MSEC;

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

        self.renderer.draw(&self.state, canvas).map(|updated| {
            updated
                | (self.state.clock_msec != last_draw_at_clock_msec
                    && self.state.clock_msec <= FADE_IN_MSEC)
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
