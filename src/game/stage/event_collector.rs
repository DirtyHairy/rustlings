use std::{
    cmp,
    time::{Duration, Instant},
};

use sdl3::{
    EventPump,
    event::{Event, WindowEvent},
    keyboard::{Keycode, Mod},
    mouse::MouseButton,
    sys::events::{self, SDL_EVENT_WINDOW_ENTER_FULLSCREEN, SDL_EVENT_WINDOW_LEAVE_FULLSCREEN},
};

use crate::scene::SceneEvent;

const INITIAL_CAPACITY: usize = 32;

pub enum GameEvent {
    Quit,
    Redraw,
    RenderReset,
    ToggleFullscreen,
    DispatchSceneEvent(SceneEvent),
    MouseMove { x: f32, y: f32 },
    MouseDown { x: f32, y: f32 },
    MouseUp { x: f32, y: f32 },
    MouseEnter,
    MouseLeave,
    EnterFullscreen,
    LeaveFullscreen,
}

pub struct EventCollector {
    decoded_events: Vec<GameEvent>,
}

impl EventCollector {
    pub fn new() -> Self {
        Self {
            decoded_events: Vec::with_capacity(INITIAL_CAPACITY),
        }
    }

    // Wait until there are relevant events or until the timeout has expired, but wait at least
    // until aggregate_at_least_until
    pub fn collect_events(
        &mut self,
        aggregate_at_least_until: Instant,
        timeout_millis: u64,
        event_pump: &mut EventPump,
    ) {
        let ts_reference = Instant::now();
        self.decoded_events.clear();

        let mut first_iteration = false;

        loop {
            let now = Instant::now();
            let elapsed = now.duration_since(ts_reference).as_millis() as u64;

            let aggregation_time_remaining = aggregate_at_least_until
                .checked_duration_since(now)
                .unwrap_or(Duration::from_millis(0))
                .as_millis() as u64;

            if !first_iteration
                && aggregation_time_remaining == 0
                && (!self.decoded_events.is_empty() || elapsed >= timeout_millis)
            {
                break;
            }

            let wait_timeout = cmp::max(
                timeout_millis.saturating_sub(elapsed),
                aggregation_time_remaining,
            );

            if let Some(event) = event_pump.wait_event_timeout(wait_timeout as u32) {
                if let Some(decoded_event) = decode_sdl_event(&event) {
                    self.decoded_events.push(decoded_event);
                }
            }

            for event in event_pump.poll_iter() {
                if let Some(decoded_event) = decode_sdl_event(&event) {
                    self.decoded_events.push(decoded_event);
                }
            }

            first_iteration = false;
        }
    }

    pub fn decoded_events(&self) -> &[GameEvent] {
        &self.decoded_events
    }
}

fn decode_sdl_event(event: &Event) -> Option<GameEvent> {
    match *event {
        Event::Quit { .. } => Some(GameEvent::Quit),

        Event::Window { win_event, .. } => match win_event {
            WindowEvent::PixelSizeChanged(_, _) => Some(GameEvent::Redraw),
            WindowEvent::MouseEnter => Some(GameEvent::MouseEnter),
            WindowEvent::MouseLeave => Some(GameEvent::MouseLeave),
            _ => None,
        },
        Event::Unknown { type_, .. } if type_ == SDL_EVENT_WINDOW_ENTER_FULLSCREEN.0 => {
            Some(GameEvent::EnterFullscreen)
        }
        Event::Unknown { type_, .. } if type_ == SDL_EVENT_WINDOW_LEAVE_FULLSCREEN.0 => {
            Some(GameEvent::LeaveFullscreen)
        }

        Event::RenderDeviceReset { .. } => Some(GameEvent::RenderReset),
        Event::RenderTargetsReset { .. } => Some(GameEvent::RenderReset),
        Event::Unknown { type_, .. } if type_ == events::SDL_EVENT_RENDER_DEVICE_LOST.0 => {
            Some(GameEvent::RenderReset)
        }

        Event::KeyDown {
            keycode: Some(keycode),
            keymod,
            scancode: Some(scancode),
            repeat: false,
            ..
        } => match (keycode, keymod) {
            (Keycode::R, Mod::LCTRLMOD | Mod::RCTRLMOD) => Some(GameEvent::RenderReset),
            (Keycode::Q, Mod::LALTMOD | Mod::RALTMOD | Mod::LGUIMOD | Mod::RGUIMOD) => {
                Some(GameEvent::Quit)
            }
            (Keycode::Return, Mod::LALTMOD | Mod::RALTMOD | Mod::LGUIMOD | Mod::RGUIMOD) => {
                Some(GameEvent::ToggleFullscreen)
            }
            _ => Some(GameEvent::DispatchSceneEvent(SceneEvent::KeyDown {
                keycode,
                keymod,
                scancode,
            })),
        },

        Event::KeyUp {
            keycode: Some(keycode),
            scancode: Some(scancode),
            repeat: false,
            ..
        } => Some(GameEvent::DispatchSceneEvent(SceneEvent::KeyUp {
            keycode,
            scancode,
        })),

        Event::MouseMotion { x, y, .. } => Some(GameEvent::MouseMove { x, y }),
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } => Some(GameEvent::MouseDown { x, y }),
        Event::MouseButtonUp {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } => Some(GameEvent::MouseUp { x, y }),

        _ => None,
    }
}
