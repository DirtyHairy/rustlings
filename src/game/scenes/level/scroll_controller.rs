use std::cmp;

use rustlings::game_data::LEVEL_WIDTH;
use sdl3::keyboard::{Keycode, Mod};

use crate::{scene::SceneEvent, state::SceneStateLevel};

pub struct ScrollController {
    arrow_left_down: bool,
    arrow_right_down: bool,
    fast_scroll: bool,
    current_scroll_mode: ScrollMode,
}

enum ScrollMode {
    Left,
    Right,
    None,
}

const SCROLL_MSEC_PER_PIXEL: u64 = 5; // 3200 msec to scroll over the full width
const FAST_SCROLL_SPEEDUP: usize = 3;
const LEVEL_X_MAX: usize = LEVEL_WIDTH - 320 - 1;

impl ScrollController {
    pub fn new() -> Self {
        ScrollController {
            arrow_left_down: false,
            arrow_right_down: false,
            fast_scroll: false,
            current_scroll_mode: ScrollMode::None,
        }
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
            SceneEvent::KeyDown {
                keycode,
                keymod: Mod::NOMOD,
            } => match keycode {
                Keycode::Left => {
                    self.arrow_left_down = true;
                    self.fast_scroll = false;
                }
                Keycode::Right => {
                    self.arrow_right_down = true;
                    self.fast_scroll = false;
                }
                _ => (),
            },
            SceneEvent::KeyDown {
                keycode,
                keymod: Mod::LSHIFTMOD | Mod::RSHIFTMOD,
            } => match keycode {
                Keycode::Left => {
                    self.arrow_left_down = true;
                    self.fast_scroll = true;
                }
                Keycode::Right => {
                    self.arrow_right_down = true;
                    self.fast_scroll = true;
                }
                _ => (),
            },
            SceneEvent::KeyUp { keycode, .. } => match keycode {
                Keycode::Left => self.arrow_left_down = false,
                Keycode::Right => self.arrow_right_down = false,
                _ => (),
            },
            _ => (),
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

        let dirty = match self.current_scroll_mode {
            ScrollMode::Left => {
                state.level_x = cmp::max(
                    0,
                    state.level_x as isize
                        - (scroll_ticks_new - scroll_ticks_current) as isize
                            * scroll_speedup as isize,
                ) as usize;

                true
            }
            ScrollMode::Right => {
                state.level_x = cmp::min(
                    LEVEL_X_MAX,
                    state.level_x
                        + (scroll_ticks_new - scroll_ticks_current) as usize * scroll_speedup,
                );

                true
            }
            _ => false,
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
