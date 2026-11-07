use std::rc::Rc;

use anyhow::{Result, anyhow};
use rustlings::{
    game_data::{GameData, NUM_LEVELS, file::palette::MENU_SCREEN_PALETTE},
    sdl::{texture_from_bitmap, with_texture_canvas},
};
use sdl3::{
    keyboard::Keycode,
    pixels::{Color, PixelFormat},
    rect::Rect,
    render::{Texture, TextureCreator},
};

use crate::{
    scene::{CursorType, MouseButton, Scene, SceneEvent},
    state::{self, GameState, GameStateGlobal},
};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 350;
const BACKGROUND_START: u32 = 72;

const TEXTURE_ID_SCREEN: usize = 0;

#[derive(PartialEq)]
enum Status {
    Running,
    DoneLaunchLevel,
    DonePreviousLevel,
    DoneNextLevel,
}

pub struct SceneBriefing<'texture_creator> {
    game_data: Rc<GameData>,
    global_state: GameStateGlobal,
    status: Status,

    texture_screen: Texture<'texture_creator>,
    static_draw_complete: bool,
}

impl<'texture_creator> SceneBriefing<'texture_creator> {
    pub fn new<T>(
        game_data: Rc<GameData>,
        global_state: GameStateGlobal,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        let texture_screen =
            texture_creator.create_texture_target(PixelFormat::RGBA8888, WIDTH, HEIGHT)?;

        Ok(Self {
            game_data,
            global_state,
            static_draw_complete: false,
            status: Status::Running,

            texture_screen,
        })
    }
}

impl<'texture_creator> Scene<'texture_creator> for SceneBriefing<'texture_creator> {
    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
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
        match id {
            TEXTURE_ID_SCREEN => Ok(&mut self.texture_screen),
            _ => Err(anyhow!("no texture with ID {}", id)),
        }
    }

    fn register_layers(&self, compositor: &mut dyn crate::scene::Compositor) {
        compositor.add_layer(
            TEXTURE_ID_SCREEN,
            WIDTH,
            HEIGHT,
            crate::geometry::Rect {
                x: 0,
                y: 0,
                width: WIDTH,
                height: HEIGHT,
            },
        );
    }

    fn draw(
        &mut self,
        canvas: &mut sdl3::render::Canvas<sdl3::video::Window>,
    ) -> anyhow::Result<bool> {
        if self.static_draw_complete {
            return Ok(false);
        }

        let texture_creator = &canvas.texture_creator();

        let texture_background = texture_from_bitmap(
            &self.game_data.screen_background,
            &MENU_SCREEN_PALETTE,
            texture_creator,
            sdl3::render::TextureAccess::Static,
        )?;

        let bg_width = self.game_data.screen_background.width;
        let bh_height = self.game_data.screen_background.height;

        with_texture_canvas(canvas, &mut self.texture_screen, |canvas| {
            canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
            canvas.set_blend_mode(sdl3::render::BlendMode::None);
            canvas.clear();

            let mut y = BACKGROUND_START as i32;
            while y < HEIGHT as i32 {
                let mut x = 0;
                while x < WIDTH as i32 {
                    canvas.copy(
                        &texture_background,
                        None,
                        Rect::new(x, y, bg_width, bh_height),
                    )?;

                    x += bg_width as i32;
                }

                y += bh_height as i32;
            }

            Ok(())
        })?;

        self.static_draw_complete = true;

        Ok(true)
    }

    fn will_redraw(&self) -> bool {
        false
    }

    fn finish(self: Box<Self>) -> state::GameState {
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
