use std::cmp;

use rustlings::game_data::{LEVEL_WIDTH, SCREEN_WIDTH};
use sdl3::keyboard::Scancode;

use crate::{scene::SceneEvent, state::SceneStateLevel};

#[derive(Default)]
pub struct ScrollController {
    arrow_left_down: bool,
    arrow_right_down: bool,
    fast_scroll: bool,
    current_scroll_mode: ScrollMode,
}

#[derive(Default)]
enum ScrollMode {
    #[default]
    None,
    Left,
    Right,
}

const SCROLL_MSEC_PER_PIXEL: u64 = 5; // 3200 msec to scroll over the full width
const FAST_SCROLL_SPEEDUP: usize = 3;
const LEVEL_X_MAX: usize = LEVEL_WIDTH - SCREEN_WIDTH;

impl ScrollController {
    pub fn new() -> Self {
        Default::default()
    }

    fn scroll_mode(&self) -> ScrollMode {
        if !(self.arrow_left_down ^ self.arrow_right_down) {
            return ScrollMode::None;
        }

        if self.arrow_left_down {
            ScrollMode::Left
        } else {
            ScrollMode::Right
        }
    }

    pub fn dispatch_event(&mut self, event: SceneEvent) {
        match event {
            SceneEvent::KeyDown { scancode, .. } => match scancode {
                Scancode::Left => self.arrow_left_down = true,
                Scancode::Right => self.arrow_right_down = true,
                Scancode::LShift | Scancode::RShift => self.fast_scroll = true,
                _ => (),
            },
            SceneEvent::KeyUp { scancode, .. } => match scancode {
                Scancode::Left => self.arrow_left_down = false,
                Scancode::Right => self.arrow_right_down = false,
                Scancode::LShift | Scancode::RShift => self.fast_scroll = false,
                _ => (),
            },
        }
    }

    pub fn tick(&mut self, clock_msec: u64, state: &mut SceneStateLevel) -> bool {
        let scroll_ticks_current = state.current_clock_msec / SCROLL_MSEC_PER_PIXEL;
        let scroll_ticks_new = clock_msec / SCROLL_MSEC_PER_PIXEL;
        let scroll_speedup = if self.fast_scroll {
            FAST_SCROLL_SPEEDUP
        } else {
            1
        };

        let current_level_x = state.level_x;
        let dirty = match self.current_scroll_mode {
            ScrollMode::Left => {
                state.level_x = state.level_x.saturating_sub(
                    scroll_ticks_new.saturating_sub(scroll_ticks_current) as usize * scroll_speedup,
                );

                current_level_x != state.level_x
            }
            ScrollMode::Right => {
                state.level_x = cmp::min(
                    LEVEL_X_MAX,
                    state.level_x
                        + (scroll_ticks_new.saturating_sub(scroll_ticks_current)) as usize
                            * scroll_speedup,
                );

                current_level_x != state.level_x
            }
            ScrollMode::None => false,
        };

        self.current_scroll_mode = self.scroll_mode();

        dirty
    }

    pub fn next_tick_at_msec(&self, state: &SceneStateLevel) -> Option<u64> {
        match self.scroll_mode() {
            ScrollMode::None => None,
            _ => Some(
                ((state.current_clock_msec / SCROLL_MSEC_PER_PIXEL) + 1) * SCROLL_MSEC_PER_PIXEL,
            ),
        }
    }
}
