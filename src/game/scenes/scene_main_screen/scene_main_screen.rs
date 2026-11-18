use std::rc::Rc;

use anyhow::{Result, anyhow};
use bitflags::{Flags, bitflags};
use rustlings::{
    game_data::{GameData, file::palette::MENU_SCREEN_PALETTE},
    sdl::{SdlAtlasBuilder, with_texture_canvas},
};
use sdl3::{
    keyboard::Keycode,
    pixels::PixelFormat,
    rect::Rect as SdlRect,
    render::{BlendMode, Canvas, Texture, TextureCreator},
    video::Window,
};

use crate::{
    geometry::Rect,
    scene::{CursorType, MouseButton, Scene, SceneEvent},
    scenes::{
        common::{FADEIN_MSEC_MENU, FadeController, TextGrid, draw_background},
        scene_main_screen::scroller::Scroller,
    },
    state::{self, ChoiceMainScreen, FadeState, GameState, GameStateGlobal, SceneStateMainScreen},
};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 350;

const TEXTURE_ID_SCREEN: usize = 0;

bitflags! {
    #[derive(Clone, Copy)]
    struct Redraw: u32 {
        const STATIC = 0x01;
        const SCROLLER = 0x02;
    }
}

pub struct SceneMainScreen<'texture_creator> {
    game_data: Rc<GameData>,
    global_state: GameStateGlobal,

    clock_offset: u64,

    state: Box<SceneStateMainScreen>,

    texture_screen: Texture<'texture_creator>,
    texture_static: Texture<'texture_creator>,

    redraw: Redraw,

    fade_controller: FadeController,
    scroller: Scroller<'texture_creator>,
}

impl<'texture_creator> SceneMainScreen<'texture_creator> {
    pub fn new<T>(
        game_data: Rc<GameData>,
        global_state: GameStateGlobal,
        state: Option<(FadeState, Box<SceneStateMainScreen>)>,
        canvas: &mut Canvas<Window>,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        let texture_screen =
            texture_creator.create_texture_target(PixelFormat::RGBA8888, WIDTH, HEIGHT)?;

        let texture_static =
            texture_creator.create_texture_target(PixelFormat::RGBA8888, WIDTH, HEIGHT)?;

        let (fade_state, scene_state) = match state {
            Some((fade_state, scene_state)) => (Some(fade_state), scene_state),
            None => (
                None,
                Box::new(SceneStateMainScreen {
                    choice: None,
                    scroller: Default::default(),
                    clock_msec: 0,
                }),
            ),
        };

        let scroller = Scroller::new(texture_creator, &game_data)?;
        let fade_controller = FadeController::new(fade_state, FADEIN_MSEC_MENU, FADEIN_MSEC_MENU);

        let mut scene = Self {
            game_data,
            global_state,
            clock_offset: scene_state.clock_msec,
            state: scene_state,
            texture_screen,
            texture_static,
            redraw: Redraw::all(),
            fade_controller,
            scroller,
        };

        scene.draw_static(canvas, texture_creator)?;

        Ok(scene)
    }

    pub fn draw_static<T>(
        &mut self,
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<T>,
    ) -> Result<()> {
        let logo_shadow = self.game_data.main_menu.logo.clone_mapped(|_, t| (0, t));
        let signs_shadow = self.game_data.main_menu.signs.clone_mapped(|_, t| (0, t));

        let mut atlas_builder = SdlAtlasBuilder::new();

        let index_logo = atlas_builder.add_sprite(&self.game_data.main_menu.logo);
        atlas_builder.add_sprite(&logo_shadow);

        let index_signs = atlas_builder.add_sprite(&self.game_data.main_menu.signs);
        atlas_builder.add_sprite(&signs_shadow);

        let atlas = atlas_builder.build(&MENU_SCREEN_PALETTE, texture_creator)?;

        let mut text_grid = TextGrid::new(WIDTH, HEIGHT);
        text_grid.write(8, 17, "(C)MCMXCI Psygnosis Ltd");
        text_grid.write(11, 18, "A DMA Design Game");

        with_texture_canvas(canvas, &mut self.texture_static, |canvas| {
            draw_background(0, &self.game_data, texture_creator, canvas)?;

            atlas.blit(canvas, index_logo + 1, 0, 20, 0, false, false)?;
            atlas.blit(canvas, index_logo, 8, 10, 0, false, false)?;

            for (i, (x, y)) in [
                (68, 120),
                (196, 120),
                (324, 120),
                (452, 120),
                (196, 197),
                (324, 197),
            ]
            .into_iter()
            .enumerate()
            {
                atlas.blit(canvas, index_signs + 1, x - 8, y + 8, i, false, false)?;
                atlas.blit(canvas, index_signs, x, y, i, false, false)?;
            }

            text_grid.render(&self.game_data, texture_creator, canvas)?;

            Ok(())
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

    fn emulate_right_mouse_buttom(&self) -> bool {
        true
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

    fn tick(&mut self, _canvas: &mut Canvas<Window>, mut clock_msec: u64) -> Result<()> {
        self.fade_controller.tick(clock_msec);

        clock_msec += self.clock_offset;
        self.state.clock_msec = clock_msec;

        if self.scroller.tick(clock_msec, &mut self.state.scroller) {
            self.redraw.insert(Redraw::SCROLLER);
        }

        Ok(())
    }

    fn next_tick_at_msec(&self) -> Option<u64> {
        let next_tick_fade = self
            .fade_controller
            .next_tick_since_creation_at_msec()
            .map(|t| t + self.clock_offset);

        let next_tick_scroller = self.scroller.next_tick_at_msec(&self.state.scroller);

        [next_tick_fade, Some(next_tick_scroller)]
            .into_iter()
            .flatten()
            .min()
            .map(|t| t.saturating_sub(self.clock_offset))
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

        if self.redraw.contains(Redraw::SCROLLER) {
            self.scroller.render(&self.state.scroller, canvas)?;
        }

        with_texture_canvas(canvas, &mut self.texture_screen, |canvas| {
            self.texture_static.set_blend_mode(BlendMode::None);
            canvas.copy(&self.texture_static, None, None)?;

            let scroller_texture = self.scroller.texture();
            scroller_texture.set_blend_mode(BlendMode::None);
            canvas.copy(scroller_texture, None, SdlRect::new(0, 334, 640, 16))?;

            Ok(())
        })?;

        self.redraw.clear();

        Ok(true)
    }

    fn will_redraw(&self) -> bool {
        !self.redraw.is_empty()
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
