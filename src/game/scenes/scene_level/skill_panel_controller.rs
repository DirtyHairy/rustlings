use rustlings::game_data::{Level, SCREEN_HEIGHT, SKILL_PANEL_HEIGHT, SKILL_TILE_WIDTH, SKILLS};
use sdl3::keyboard::Keycode;

use crate::{
    scene::{MouseCoordinates, SceneEvent},
    state::SceneStateLevel,
};

#[derive(Default)]
pub struct SkillPanelController {
    incrementing: bool,
    incremented: u32,

    decrementing: bool,
    decremented: u32,

    release_rate_min: u32,
}

const SKILL_PANEL_Y: u32 = SCREEN_HEIGHT - SKILL_PANEL_HEIGHT;

impl SkillPanelController {
    pub fn new(level: &Level) -> Self {
        Self {
            release_rate_min: level.parameters.release_rate,
            ..Default::default()
        }
    }

    pub fn dispatch_event(&mut self, event: SceneEvent, state: &mut SceneStateLevel) -> bool {
        match event {
            SceneEvent::MouseDown(MouseCoordinates { x, y, .. }) => {
                self.handle_mouse_down(state, x, y)
            }
            SceneEvent::MouseUp(..) => self.handle_mouse_up(state),
            SceneEvent::KeyDown { keycode, .. } => match keycode {
                Keycode::Plus => {
                    self.start_increment();
                    false
                }
                Keycode::Minus => {
                    self.start_decrement();
                    false
                }
                Keycode::_1 => {
                    state.selected_skill = SKILLS[0];
                    true
                }
                Keycode::_2 => {
                    state.selected_skill = SKILLS[1];
                    true
                }
                Keycode::_3 => {
                    state.selected_skill = SKILLS[2];
                    true
                }
                Keycode::_4 => {
                    state.selected_skill = SKILLS[3];
                    true
                }
                Keycode::_5 => {
                    state.selected_skill = SKILLS[4];
                    true
                }
                Keycode::_6 => {
                    state.selected_skill = SKILLS[5];
                    true
                }
                Keycode::_7 => {
                    state.selected_skill = SKILLS[6];
                    true
                }
                Keycode::_8 => {
                    state.selected_skill = SKILLS[7];
                    true
                }
                Keycode::P => {
                    state.paused = !state.paused;
                    true
                }
                _ => false,
            },
            SceneEvent::KeyUp { keycode, .. } => match keycode {
                Keycode::Plus => {
                    self.stop_increment(state);
                    true
                }
                Keycode::Minus => {
                    self.stop_decrement(state);
                    true
                }
                _ => false,
            },
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

    fn handle_mouse_down(&mut self, state: &mut SceneStateLevel, x: u32, y: u32) -> bool {
        if y < SKILL_PANEL_Y + 16 {
            return false;
        }

        let tile_index = x / SKILL_TILE_WIDTH;

        match tile_index {
            0 => {
                self.start_decrement();
                false
            }
            1 => {
                self.start_increment();
                false
            }
            2..10 => {
                state.selected_skill = SKILLS[(tile_index - 2) as usize];
                true
            }
            10 => {
                state.paused = !state.paused;
                true
            }
            11 => {
                println!("armageddon");
                false
            }
            _ => false,
        }
    }

    fn handle_mouse_up(&mut self, state: &mut SceneStateLevel) -> bool {
        let redraw = self.incrementing || self.decrementing;

        self.stop_decrement(state);
        self.stop_increment(state);

        redraw
    }

    fn start_increment(&mut self) {
        self.incrementing = true;
        self.incremented = 0;
    }

    fn stop_increment(&mut self, state: &mut SceneStateLevel) {
        if self.incrementing && self.incremented == 0 {
            self.increase_release(state);
        }

        self.incrementing = false;
    }

    fn start_decrement(&mut self) {
        self.decrementing = true;
        self.decremented = 0;
    }

    fn stop_decrement(&mut self, state: &mut SceneStateLevel) {
        if self.decrementing && self.decremented == 0 {
            self.decrement_release(state);
        }

        self.decrementing = false;
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
