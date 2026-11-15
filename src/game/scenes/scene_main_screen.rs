use std::rc::Rc;

use anyhow::{Result, anyhow};
use bitflags::{Flags, bitflags};
use rustlings::{game_data::GameData, sdl::with_texture_canvas};
use sdl3::{
    keyboard::Keycode,
    pixels::PixelFormat,
    render::{Canvas, Texture, TextureCreator},
    video::Window,
};

use crate::{
    geometry::Rect,
    scene::{CursorType, MouseButton, Scene, SceneEvent},
    scenes::common::{FADEIN_MSEC_MENU, FadeController, draw_background},
    state::{self, ChoiceMainScreen, FadeState, GameState, GameStateGlobal, SceneStateMainScreen},
};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 350;

const TEXTURE_ID_SCREEN: usize = 0;

bitflags! {
    #[derive(Clone, Copy)]
    struct Redraw: u32 {
        const STATIC = 0x01;
    }
}

pub struct SceneMainScreen<'texture_creator> {
    game_data: Rc<GameData>,
    global_state: GameStateGlobal,

    state: SceneStateMainScreen,

    texture_screen: Texture<'texture_creator>,
    texture_static: Texture<'texture_creator>,

    redraw: Redraw,

    fade_controller: FadeController,
}

impl<'texture_creator> SceneMainScreen<'texture_creator> {
    pub fn new<T>(
        game_data: Rc<GameData>,
        global_state: GameStateGlobal,
        state: Option<(FadeState, SceneStateMainScreen)>,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        let texture_screen =
            texture_creator.create_texture_target(PixelFormat::RGBA8888, WIDTH, HEIGHT)?;

        let texture_static =
            texture_creator.create_texture_target(PixelFormat::RGBA8888, WIDTH, HEIGHT)?;

        let (fade_state, scene_state) = match state {
            Some((fade_state, scene_state)) => (Some(fade_state), scene_state),
            None => (None, SceneStateMainScreen { choice: None }),
        };

        let fade_controller = FadeController::new(fade_state, FADEIN_MSEC_MENU, FADEIN_MSEC_MENU);

        Ok(Self {
            game_data,
            global_state,
            state: scene_state,
            texture_screen,
            texture_static,
            redraw: Redraw::all(),
            fade_controller,
        })
    }

    pub fn draw_static(&mut self, canvas: &mut Canvas<Window>) -> Result<()> {
        let texture_creator = canvas.texture_creator();

        with_texture_canvas(canvas, &mut self.texture_static, |canvas| {
            draw_background(0, &self.game_data, &texture_creator, canvas)
        })?;

        Ok(())
    }
}

impl<'texture_creator> Scene<'texture_creator> for SceneMainScreen<'texture_creator> {
    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn opacity(&self) -> u8 {
        self.fade_controller.opacity()
    }

    fn set_is_fullscreen(&mut self, _is_fullscreen: bool) {}

    fn set_mouse_enabled(&mut self, _mouse_enabled: bool) {}

    fn cursor_type(&mut self) -> crate::scene::CursorType {
        CursorType::None
    }

    fn dispatch_event(&mut self, event: crate::scene::SceneEvent) {
        if self.state.choice.is_some() {
            return;
        }

        match event {
            SceneEvent::KeyDown {
                keycode: Keycode::Return | Keycode::Space | Keycode::KpEnter | Keycode::F1,
                ..
            }
            | SceneEvent::MouseDown(MouseButton::Left, _) => {
                self.state.choice = Some(ChoiceMainScreen::Play);
                self.fade_controller.start_fadeout();
            }
            SceneEvent::KeyDown {
                keycode: Keycode::Escape,
                ..
            } => self.state.choice = Some(ChoiceMainScreen::Quit),
            _ => (),
        }
    }

    fn tick(&mut self, _canvas: &mut Canvas<Window>, clock_msec: u64) -> Result<()> {
        self.fade_controller.tick(clock_msec);

        Ok(())
    }

    fn next_tick_at_msec(&self) -> Option<u64> {
        self.fade_controller.next_tick_since_creation_at_msec()
    }

    fn is_complete(&self) -> bool {
        self.fade_controller.fadeout_done() || self.state.choice == Some(ChoiceMainScreen::Quit)
    }

    fn texture(&mut self, id: usize) -> Result<&mut Texture<'texture_creator>> {
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
            Rect::new(0, 0, WIDTH, HEIGHT),
        );
    }

    fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<bool> {
        if self.redraw.is_empty() {
            return Ok(false);
        }

        if self.redraw.contains(Redraw::STATIC) {
            self.draw_static(canvas)?;
        }

        with_texture_canvas(canvas, &mut self.texture_screen, |canvas| {
            canvas.copy(&self.texture_static, None, None)?;

            Ok(())
        })?;

        self.redraw.clear();

        Ok(true)
    }

    fn will_redraw(&self) -> bool {
        false
    }

    fn finish(self: Box<Self>) -> Option<GameState> {
        if self.state.choice == Some(ChoiceMainScreen::Quit) {
            return None;
        }

        let new_state = match self.state.choice {
            Some(ChoiceMainScreen::Play) if self.fade_controller.fadeout_done() => GameState {
                global: self.global_state,
                scene: state::Scene::Briefing(None),
            },
            _ => GameState {
                global: self.global_state,
                scene: state::Scene::MainScreen(Some((self.fade_controller.finish(), self.state))),
            },
        };

        Some(new_state)
    }
}
