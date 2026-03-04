use std::time::Instant;

use rustlings::sdl3_aux::SDL_EVENT_RENDER_DEVICE_LOST;
use sdl3::{
    EventPump,
    event::{Event, WindowEvent},
    keyboard::{Keycode, Mod},
};

use crate::scene::SceneEvent;

const INITIAL_CAPACITY: usize = 32;

pub enum DecodedEvent {
    Quit,
    Redraw,
    RenderReset,
    ToggleFullscreen,
    DispatchSceneEvent(SceneEvent),
    MouseMove { x: f32, y: f32 },
}

pub struct EventCollector {
    decoded_events: Vec<DecodedEvent>,
}

impl EventCollector {
    pub fn new() -> Self {
        Self {
            decoded_events: Vec::with_capacity(INITIAL_CAPACITY),
        }
    }

    pub fn collect_events(
        &mut self,
        aggregate_at_least_until: Instant,
        timeout_millis: u64,
        event_pump: &mut EventPump,
    ) {
        let ts_reference = Instant::now();
        let mut elapsed: u64 = 0;

        self.decoded_events.clear();

        loop {
            if let Some(event) = event_pump.wait_event_timeout((timeout_millis - elapsed) as u32) {
                match decode_sdl_event(&event) {
                    Some(decoded_event) => self.decoded_events.push(decoded_event),
                    None => (),
                }
            }

            for event in event_pump.poll_iter() {
                match decode_sdl_event(&event) {
                    Some(decoded_event) => self.decoded_events.push(decoded_event),
                    None => (),
                }
            }

            let now = Instant::now();
            elapsed = now.duration_since(ts_reference).as_millis() as u64;
            if self.decoded_events.len() > 0 && now >= aggregate_at_least_until
                || elapsed >= timeout_millis
            {
                break;
            }
        }
    }

    pub fn decoded_events(&self) -> &[DecodedEvent] {
        &self.decoded_events
    }
}

fn decode_sdl_event(event: &Event) -> Option<DecodedEvent> {
    match *event {
        Event::Quit { .. } => Some(DecodedEvent::Quit),
        Event::Window { win_event, .. } => match win_event {
            WindowEvent::PixelSizeChanged(_, _) => Some(DecodedEvent::Redraw),
            _ => None,
        },
        Event::RenderDeviceReset { .. } => Some(DecodedEvent::RenderReset),
        Event::RenderTargetsReset { .. } => Some(DecodedEvent::RenderReset),
        Event::Unknown {
            type_: SDL_EVENT_RENDER_DEVICE_LOST,
            ..
        } => Some(DecodedEvent::RenderReset),

        Event::KeyDown {
            keycode: Some(keycode),
            keymod,
            scancode: Some(scancode),
            repeat: false,
            ..
        } => match (keycode, keymod) {
            (Keycode::R, Mod::LCTRLMOD | Mod::RCTRLMOD) => Some(DecodedEvent::RenderReset),
            (Keycode::Q, Mod::LALTMOD | Mod::RALTMOD | Mod::LGUIMOD | Mod::RGUIMOD) => {
                Some(DecodedEvent::Quit)
            }
            (Keycode::Return, Mod::LALTMOD | Mod::RALTMOD | Mod::LGUIMOD | Mod::RGUIMOD) => {
                Some(DecodedEvent::ToggleFullscreen)
            }
            _ => Some(DecodedEvent::DispatchSceneEvent(SceneEvent::KeyDown {
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
        } => Some(DecodedEvent::DispatchSceneEvent(SceneEvent::KeyUp {
            keycode,
            scancode,
        })),

        Event::MouseMotion { x, y, .. } => Some(DecodedEvent::MouseMove { x, y }),

        _ => None,
    }
}
