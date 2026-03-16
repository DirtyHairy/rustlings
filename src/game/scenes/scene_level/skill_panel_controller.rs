use std::ops::Add;

use rustlings::game_data::{Level, SCREEN_HEIGHT, SKILL_PANEL_HEIGHT, SKILL_TILE_WIDTH, SKILLS};

use crate::{
    scene::{MouseCoordinates, SceneEvent},
    state::SceneStateLevel,
};

#[derive(Default)]
pub struct SkillPanelController {
    incrementing: bool,
    incremented: usize,

    decrementing: bool,
    decremented: usize,

    release_rate_min: usize,
}

const SKILL_PANEL_Y: usize = SCREEN_HEIGHT - SKILL_PANEL_HEIGHT;

impl SkillPanelController {
    pub fn new(level: &Level) -> Self {
        Self {
            release_rate_min: level.parameters.release_rate as usize,
            ..Default::default()
        }
    }

    pub fn dispatch_event(&mut self, event: SceneEvent, state: &mut SceneStateLevel) -> bool {
        match event {
            SceneEvent::MouseDown(MouseCoordinates { x, y, .. }) => {
                self.handle_mouse_down(x, y, state)
            }
            SceneEvent::MouseUp(..) => self.handle_mouse_up(state),
            _ => false,
        }
    }

    pub fn tick(&mut self, state: &mut SceneStateLevel) -> bool {
        if self.incrementing {
            self.increase_release(state);
        }

        if self.decrementing {
            self.decrement_release(state);
        }

        self.incrementing || self.decrementing
    }

    fn handle_mouse_down(&mut self, x: usize, y: usize, state: &mut SceneStateLevel) -> bool {
        if y < SKILL_PANEL_Y + 16 {
            return false;
        }

        let tile_index = x / SKILL_TILE_WIDTH;

        match tile_index {
            0 => {
                self.decrementing = true;
                self.decremented = 0;

                true
            }
            1 => {
                self.incrementing = true;
                self.incremented = 0;

                true
            }
            2..10 => {
                println!("selected {}", SKILLS[tile_index - 2]);
                false
            }
            10 => {
                println!("pause");
                false
            }
            11 => {
                println!("armageddon");
                false
            }
            _ => false,
        }
    }

    fn handle_mouse_up(&mut self, state: &mut SceneStateLevel) -> bool {
        if self.incrementing && self.incremented == 0 {
            self.increase_release(state);
        }

        if self.decrementing && self.incremented == 0 {
            self.decrement_release(state);
        }

        let redraw = self.incrementing || self.decrementing;

        self.decrementing = false;
        self.incrementing = false;

        redraw
    }

    fn increase_release(&mut self, state: &mut SceneStateLevel) {
        state.release_rate = state.release_rate.saturating_add(1).min(99);
        self.incremented = self.incremented.saturating_add(1);
    }

    fn decrement_release(&mut self, state: &mut SceneStateLevel) {
        state.release_rate = state
            .release_rate
            .saturating_sub(1)
            .max(self.release_rate_min);

        self.decremented = self.decremented.saturating_add(1);
    }
}
