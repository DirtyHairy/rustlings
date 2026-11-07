use anyhow::bail;
use rustlings::game_data::NUM_LEVELS;
use sdl3::keyboard::Keycode;

use crate::{
    scene::{CursorType, MouseButton, Scene, SceneEvent},
    state::{self, GameState, GameStateGlobal},
};

#[derive(PartialEq, strum::Display)]
enum Status {
    Running,
    DoneLaunchLevel,
    DonePreviousLevel,
    DoneNextLevel,
}

pub struct SceneBriefing {
    global_state: GameStateGlobal,
    status: Status,
}

impl SceneBriefing {
    pub fn new(global_state: GameStateGlobal) -> Self {
        Self {
            global_state,
            status: Status::Running,
        }
    }
}

impl<'texture_creator> Scene<'texture_creator> for SceneBriefing {
    fn width(&self) -> u32 {
        640
    }

    fn height(&self) -> u32 {
        350
    }

    fn opacity(&self) -> u8 {
        255
    }

    fn set_is_fullscreen(&mut self, _is_fullscreen: bool) {}

    fn set_mouse_enabled(&mut self, _mouse_enabled: bool) {}

    fn cursor_type(&mut self) -> CursorType {
        CursorType::None
    }

    fn dispatch_event(&mut self, event: crate::scene::SceneEvent) {
        match event {
            SceneEvent::KeyDown {
                keycode: Keycode::PageDown,
                ..
            } => {
                if self.status == Status::Running {
                    self.status = Status::DonePreviousLevel;
                }
            }
            SceneEvent::KeyDown {
                keycode: Keycode::PageUp,
                ..
            } => {
                if self.status == Status::Running {
                    self.status = Status::DoneNextLevel;
                }
            }
            SceneEvent::KeyDown {
                keycode: Keycode::Space | Keycode::Return | Keycode::KpEnter,
                ..
            } => {
                if self.status == Status::Running {
                    self.status = Status::DoneLaunchLevel
                }
            }
            SceneEvent::MouseDown(MouseButton::Left, _) => self.status = Status::DoneLaunchLevel,
            _ => (),
        };
    }

    fn tick(
        &mut self,
        _canvas: &mut sdl3::render::Canvas<sdl3::video::Window>,
        _clock_msec: u64,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn next_tick_at_msec(&self) -> Option<u64> {
        None
    }

    fn is_complete(&self) -> bool {
        self.status != Status::Running
    }

    fn texture(
        &mut self,
        id: usize,
    ) -> anyhow::Result<&mut sdl3::render::Texture<'texture_creator>> {
        bail!("no texture with ID {}", id)
    }

    fn register_layers(&self, _compositor: &mut dyn crate::scene::Compositor) {}

    fn draw(
        &mut self,
        _canvas: &mut sdl3::render::Canvas<sdl3::video::Window>,
    ) -> anyhow::Result<bool> {
        Ok(false)
    }

    fn will_redraw(&self) -> bool {
        false
    }

    fn finish(self: Box<Self>) -> state::GameState {
        println!("finish {}", self.status);

        match self.status {
            Status::DoneLaunchLevel => GameState {
                global: self.global_state,
                scene: state::Scene::Level(None),
            },
            Status::DoneNextLevel => GameState {
                global: GameStateGlobal {
                    current_level: (self.global_state.current_level + 1) % NUM_LEVELS,
                },
                scene: state::Scene::Briefing,
            },
            Status::DonePreviousLevel => GameState {
                global: GameStateGlobal {
                    current_level: (self.global_state.current_level + NUM_LEVELS - 1) % NUM_LEVELS,
                },
                scene: state::Scene::Briefing,
            },
            Status::Running => GameState {
                global: self.global_state,
                scene: state::Scene::Briefing,
            },
        }
    }
}
