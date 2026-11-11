use core::time::Duration;
use std::{cmp, time::Instant};

use anyhow::anyhow;
use sdl3::{
    EventPump,
    event::{DisplayEvent, Event, WindowEvent},
    keyboard::{Keycode, Mod},
    mouse::MouseButton as SdlMouseButton,
    sys::events::{self, SDL_EVENT_WINDOW_ENTER_FULLSCREEN, SDL_EVENT_WINDOW_LEAVE_FULLSCREEN},
};

use crate::scene::SceneEvent;

const INITIAL_CAPACITY: usize = 32;

#[derive(Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
}

impl TryFrom<SdlMouseButton> for MouseButton {
    type Error = anyhow::Error;

    fn try_from(btn: SdlMouseButton) -> Result<Self, Self::Error> {
        match btn {
            SdlMouseButton::Left => Ok(Self::Left),
            SdlMouseButton::Right => Ok(Self::Right),
            _ => Err(anyhow!("unmapped button")),
        }
    }
}

pub enum GameEvent {
    Quit,
    Redraw,
    RenderReset,
    ToggleFullscreen,
    DispatchSceneEvent(SceneEvent),
    MouseMove { x: f32, y: f32 },
    MouseDown { x: f32, y: f32, button: MouseButton },
    MouseUp { x: f32, y: f32, button: MouseButton },
    MouseEnter,
    MouseLeave,
    EnterFullscreen,
    LeaveFullscreen,
    ModeChanged,
    SetSpeed(u32),
}

enum GameEvents {
    None,
    One(GameEvent),
    Two(GameEvent, GameEvent),
}

impl From<Option<GameEvent>> for GameEvents {
    fn from(value: Option<GameEvent>) -> Self {
        value.map_or(GameEvents::None, GameEvents::One)
    }
}

pub enum MouseOwnerChange {
    Enter,
    Leave,
}

#[derive(Default)]
pub struct EventAggregate {
    pub mouse_coordinates: Option<(f32, f32)>,
    pub mouse_owner_change: Option<MouseOwnerChange>,
}

pub struct EventCollector {
    decoded_events: Vec<GameEvent>,
    aggregate: EventAggregate,
}

impl EventCollector {
    pub fn new() -> Self {
        Self {
            decoded_events: Vec::with_capacity(INITIAL_CAPACITY),
            aggregate: Default::default(),
        }
    }

    // Wait until there are relevant events or until the timeout has expired, but wait at least
    // until aggregate_at_least_until
    pub fn collect_events(
        &mut self,
        aggregate_at_least_until: Instant,
        timeout_millis: Option<u64>,
        event_pump: &mut EventPump,
    ) {
        let ts_reference = Instant::now();

        self.decoded_events.clear();
        self.aggregate = Default::default();

        let mut first_iteration = true;

        loop {
            let now = Instant::now();
            let elapsed = now.duration_since(ts_reference).as_millis() as u64;

            let aggregation_time_remaining = aggregate_at_least_until
                .checked_duration_since(now)
                .unwrap_or(Duration::ZERO)
                .as_millis() as u64;

            if !first_iteration
                && aggregation_time_remaining == 0
                && (!self.decoded_events.is_empty()
                    || timeout_millis.is_some_and(|timeout| elapsed >= timeout))
            {
                break;
            }

            let maybe_event = if let Some(timeout) = timeout_millis {
                let wait_timeout =
                    cmp::max(timeout.saturating_sub(elapsed), aggregation_time_remaining);

                event_pump.wait_event_timeout(Duration::from_millis(wait_timeout))
            } else {
                let mut maybe_evt: Option<Event>;

                loop {
                    maybe_evt = event_pump.wait_event_timeout(Duration::from_millis(100));
                    if maybe_evt.is_some() {
                        break;
                    }
                }

                maybe_evt
            };

            for event in maybe_event.into_iter().chain(event_pump.poll_iter()) {
                match decode_sdl_event(&event) {
                    GameEvents::None => (),
                    GameEvents::One(evt) => {
                        self.aggregate_event(&evt);
                        self.decoded_events.push(evt);
                    }
                    GameEvents::Two(evt1, evt2) => {
                        for evt in [evt1, evt2] {
                            self.aggregate_event(&evt);
                            self.decoded_events.push(evt);
                        }
                    }
                }
            }

            first_iteration = false;
        }
    }

    pub fn decoded_events(&self) -> &[GameEvent] {
        &self.decoded_events
    }

    pub fn aggregated_events(&self) -> &EventAggregate {
        &self.aggregate
    }

    fn aggregate_event(&mut self, event: &GameEvent) {
        match event {
            GameEvent::MouseMove { x, y }
            | GameEvent::MouseDown { x, y, .. }
            | GameEvent::MouseUp { x, y, .. } => {
                self.aggregate.mouse_coordinates = Some((*x, *y));
            }
            GameEvent::MouseEnter => {
                self.aggregate.mouse_owner_change = Some(MouseOwnerChange::Enter);
            }
            GameEvent::MouseLeave => {
                self.aggregate.mouse_owner_change = Some(MouseOwnerChange::Leave);
            }
            _ => (),
        }
    }
}

fn decode_sdl_event(event: &Event) -> GameEvents {
    let mod_all =
        Mod::LALTMOD | Mod::RALTMOD | Mod::LGUIMOD | Mod::RGUIMOD | Mod::LCTRLMOD | Mod::RCTRLMOD;

    match *event {
        Event::Quit { .. } => GameEvents::One(GameEvent::Quit),

        Event::Window { win_event, .. } => match win_event {
            WindowEvent::PixelSizeChanged(_, _) => GameEvents::One(GameEvent::Redraw),
            WindowEvent::MouseEnter => GameEvents::One(GameEvent::MouseEnter),
            WindowEvent::MouseLeave => GameEvents::One(GameEvent::MouseLeave),
            _ => GameEvents::None,
        },
        Event::Unknown { type_, .. } if type_ == SDL_EVENT_WINDOW_ENTER_FULLSCREEN.0 => {
            GameEvents::One(GameEvent::EnterFullscreen)
        }
        Event::Unknown { type_, .. } if type_ == SDL_EVENT_WINDOW_LEAVE_FULLSCREEN.0 => {
            GameEvents::One(GameEvent::LeaveFullscreen)
        }

        Event::RenderDeviceReset { .. } => GameEvents::One(GameEvent::RenderReset),
        Event::RenderTargetsReset { .. } => GameEvents::One(GameEvent::RenderReset),
        Event::Unknown { type_, .. } if type_ == events::SDL_EVENT_RENDER_DEVICE_LOST.0 => {
            GameEvents::One(GameEvent::RenderReset)
        }

        Event::Display {
            display_event: DisplayEvent::Moved,
            ..
        }
        | Event::Display {
            display_event: DisplayEvent::CurrentModeChanged,
            ..
        }
        | Event::Display {
            display_event: DisplayEvent::DesktopModeChanged,
            ..
        } => GameEvents::One(GameEvent::ModeChanged),
        Event::KeyDown {
            keycode: Some(keycode),
            keymod,
            scancode: Some(scancode),
            repeat: false,
            ..
        } => match (keycode, keymod) {
            (Keycode::R, m) if m.intersects(mod_all) => GameEvents::One(GameEvent::RenderReset),
            (Keycode::Q, m) if m.intersects(mod_all) => GameEvents::One(GameEvent::Quit),
            (Keycode::Return, m) if m.intersects(mod_all) => {
                GameEvents::One(GameEvent::ToggleFullscreen)
            }
            (Keycode::Space, m) if m.intersects(mod_all) => GameEvents::One(GameEvent::SetSpeed(4)),
            _ => GameEvents::One(GameEvent::DispatchSceneEvent(SceneEvent::KeyDown {
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
        } => match keycode {
            Keycode::Space => GameEvents::Two(
                GameEvent::SetSpeed(1),
                GameEvent::DispatchSceneEvent(SceneEvent::KeyUp { keycode, scancode }),
            ),
            _ => GameEvents::One(GameEvent::DispatchSceneEvent(SceneEvent::KeyUp {
                keycode,
                scancode,
            })),
        },

        Event::MouseMotion { x, y, .. } => GameEvents::One(GameEvent::MouseMove { x, y }),
        Event::MouseButtonDown {
            mouse_btn, x, y, ..
        } => MouseButton::try_from(mouse_btn)
            .ok()
            .map(|button| GameEvent::MouseDown { x, y, button })
            .into(),
        Event::MouseButtonUp {
            mouse_btn, x, y, ..
        } => MouseButton::try_from(mouse_btn)
            .ok()
            .map(|button| GameEvent::MouseUp { x, y, button })
            .into(),
        _ => GameEvents::None,
    }
}
