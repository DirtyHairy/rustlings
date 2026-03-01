use std::{cmp, rc::Rc};

use anyhow::Result;
use rustlings::game_data::{GameData, SCREEN_HEIGHT, SCREEN_WIDTH};
use sdl3::{
    render::{Canvas, Texture, TextureCreator},
    video::Window,
};

use crate::{scene::Scene, scenes::level::ScrollController};
use crate::{
    scenes::level::{Redraw, Renderer},
    state::{GameState, SceneState, SceneStateLevel},
};

const ENGINE_TICK_MSEC: u64 = 66; // 15.15 FPS

pub struct SceneLevel<'texture_creator> {
    game_state: GameState,
    state: SceneStateLevel,

    renderer: Renderer<'texture_creator>,
    scroll_controller: ScrollController,
}

impl<'texture_creator> SceneLevel<'texture_creator> {
    pub fn new<T>(
        game_data: Rc<GameData>,
        game_state: GameState,
        scene_state: SceneState,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        let level = game_data.resolve_level(game_state.current_level)?;

        let state = match scene_state {
            SceneState::Level(state_level) => state_level,
            _ => SceneStateLevel {
                level_x: level.start_x as usize,
                terrain: game_data.compose_terrain(&level)?,
                current_clock_msec: 0,
            },
        };

        let renderer = Renderer::new(&level, &state, game_data.clone(), texture_creator)?;

        Ok(SceneLevel {
            game_state,
            state,
            renderer,
            scroll_controller: ScrollController::new(),
        })
    }
}

impl<'texture_creator> Scene<'texture_creator> for SceneLevel<'texture_creator> {
    fn finish(mut self: Box<Self>) -> (GameState, SceneState) {
        self.state.current_clock_msec %= ENGINE_TICK_MSEC;

        (self.game_state, SceneState::Level(self.state))
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

    fn dispatch_event(&mut self, event: crate::scene::SceneEvent) {
        self.scroll_controller.dispatch_event(event);
    }

    fn tick(&mut self, clock_msec: u64) {
        if clock_msec <= self.state.current_clock_msec {
            return;
        }

        if self.scroll_controller.tick(clock_msec, &mut self.state) {
            self.renderer.mark_for_redraw(Redraw::SCREEN);
        }

        self.state.current_clock_msec = clock_msec;
    }

    fn next_tick_at_msec(&self) -> u64 {
        let next_tick_engine =
            ((self.state.current_clock_msec / ENGINE_TICK_MSEC) + 1) * ENGINE_TICK_MSEC;

        match self.scroll_controller.next_tick_at_msec(&self.state) {
            None => next_tick_engine,
            Some(next_tick_scroll) => cmp::min(next_tick_engine, next_tick_scroll),
        }
    }

    fn register_layers(&self, compositor: &mut dyn crate::scene::Compositor) {
        self.renderer.register_layers(compositor);
    }

    fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<bool> {
        self.renderer.draw(&self.state, canvas)
    }

    fn texture(&mut self, id: usize) -> Result<&mut Texture<'texture_creator>> {
        self.renderer.texture(id)
    }
}
