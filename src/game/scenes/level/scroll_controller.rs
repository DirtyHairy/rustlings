use std::cmp;

use rustlings::game_data::{
    LEVEL_WIDTH, MINIMAP_AREA_HEIGHT, MINIMAP_AREA_WIDTH, MINIMAP_AREA_X, MINIMAP_AREA_Y,
    MINIMAP_FRAME_WIDTH, MINIMAP_VIEW_WIDTH, MINIMAP_VIEW_X, SCREEN_HEIGHT, SCREEN_WIDTH,
    SKILL_PANEL_HEIGHT,
};
use sdl3::keyboard::Scancode;

use crate::{
    scene::{MouseCoordinates, SceneEvent},
    state::SceneStateLevel,
};

#[derive(Default)]
pub struct ScrollController {
    arrow_left_down: bool,
    arrow_right_down: bool,
    mouse_down: bool,

    fast_scroll: bool,
    current_scroll_mode: ScrollMode,
}

#[derive(Default)]
enum ScrollMode {
    #[default]
    None,
    Left,
    Right,
    Drag,
}

const SCROLL_MSEC_PER_PIXEL: u64 = 5; // 3200 msec to scroll over the full width
const FAST_SCROLL_SPEEDUP: usize = 3;
const LEVEL_X_MAX: usize = LEVEL_WIDTH - SCREEN_WIDTH;

impl ScrollController {
    pub fn new() -> Self {
        Default::default()
    }

    fn scroll_mode(&self) -> ScrollMode {
        if self.mouse_down {
            return ScrollMode::Drag;
        }

        if !(self.arrow_left_down ^ self.arrow_right_down) {
            return ScrollMode::None;
        }

        if self.arrow_left_down {
            ScrollMode::Left
        } else {
            ScrollMode::Right
        }
    }

    pub fn dispatch_event(&mut self, event: SceneEvent, state: &mut SceneStateLevel) -> bool {
        match event {
            SceneEvent::KeyDown { scancode, .. } => {
                match scancode {
                    Scancode::Left => self.arrow_left_down = true,
                    Scancode::Right => self.arrow_right_down = true,
                    Scancode::LShift | Scancode::RShift => self.fast_scroll = true,
                    _ => (),
                };
                false
            }
            SceneEvent::KeyUp { scancode, .. } => {
                match scancode {
                    Scancode::Left => self.arrow_left_down = false,
                    Scancode::Right => self.arrow_right_down = false,
                    Scancode::LShift | Scancode::RShift => self.fast_scroll = false,
                    _ => (),
                };
                false
            }
            SceneEvent::MouseMove(coordinates) => {
                if self.mouse_down {
                    self.update_from_drag(&coordinates, state);
                    true
                } else {
                    false
                }
            }
            SceneEvent::MouseDown(coordinates) => {
                if in_minimap(coordinates) {
                    self.mouse_down = true;
                    self.update_from_drag(&coordinates, state);
                    true
                } else {
                    false
                }
            }
            SceneEvent::MouseUp(coordinates) => {
                if self.mouse_down {
                    self.update_from_drag(&coordinates, state);
                    self.mouse_down = false;
                    true
                } else {
                    false
                }
            }
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
            _ => false,
        };

        self.current_scroll_mode = self.scroll_mode();

        dirty
    }

    pub fn next_tick_at_msec(&self, state: &SceneStateLevel) -> Option<u64> {
        match self.scroll_mode() {
            ScrollMode::Left | ScrollMode::Right => Some(
                ((state.current_clock_msec / SCROLL_MSEC_PER_PIXEL) + 1) * SCROLL_MSEC_PER_PIXEL,
            ),
            _ => None,
        }
    }

    fn update_from_drag(&mut self, coordinates: &MouseCoordinates, state: &mut SceneStateLevel) {
        let x =
            (coordinates.x_frac - MINIMAP_VIEW_X as f32 - ((MINIMAP_FRAME_WIDTH - 2) / 2) as f32)
                / MINIMAP_VIEW_WIDTH as f32
                * LEVEL_WIDTH as f32;

        state.level_x = (x.round() as isize).clamp(0, LEVEL_X_MAX as isize) as usize;
    }
}

fn in_minimap(coordinates: MouseCoordinates) -> bool {
    (coordinates.x >= MINIMAP_AREA_X)
        && (coordinates.x < MINIMAP_AREA_X + MINIMAP_AREA_WIDTH)
        && (coordinates.y >= MINIMAP_AREA_Y + SCREEN_HEIGHT - SKILL_PANEL_HEIGHT)
        && (coordinates.y
            < MINIMAP_AREA_Y + MINIMAP_AREA_HEIGHT + SCREEN_HEIGHT - SKILL_PANEL_HEIGHT)
}
