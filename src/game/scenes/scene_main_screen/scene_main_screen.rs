use std::rc::Rc;

use anyhow::{Result, anyhow};
use bitflags::{Flags, bitflags};
use rustlings::{
    game_data::{GameData, LEVELS_PER_DIFFICULTY, file::palette::MENU_SCREEN_PALETTE},
    sdl::{SdlAtlas, SdlAtlasBuilder, with_texture_canvas},
};
use sdl3::{
    keyboard::Keycode,
    pixels::PixelFormat,
    render::{Canvas, Texture, TextureCreator},
    sys::blendmode::SDL_BLENDMODE_NONE,
    video::Window,
};

use crate::{
    geometry::Rect,
    scene::{CursorType, MouseButton, Scene, SceneEvent},
    scenes::{
        common::{
            FADEIN_MUSEC_MENU, FADEOUT_MUSEC_MENU, FadeController, TextGrid, draw_background,
        },
        scene_main_screen::{eyes::Eyes, scroller::Scroller},
    },
    state::{
        self, AudioSetting, ChoiceMainScreen, FadeState, GameState, GameStateGlobal,
        SceneStateMainScreen,
    },
    timing::RenderConfiguration,
};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 350;

const TEXTURE_ID_SCREEN: usize = 0;

bitflags! {
    #[derive(Clone, Copy)]
    struct Redraw: u32 {
        const SCROLLER = 0x02;
        const EYES = 0x04;
        const DIFFICULTY = 0x08;
        const AUDIO_SETTING = 0x10;
    }
}

pub struct SceneMainScreen<'texture_creator> {
    game_data: Rc<GameData>,
    global_state: GameStateGlobal,

    state: Box<SceneStateMainScreen>,

    texture_screen: Texture<'texture_creator>,

    redraw: Redraw,

    fade_controller: FadeController,
    scroller: Scroller<'texture_creator>,
    eyes: Eyes<'texture_creator>,

    atlas: SdlAtlas<'texture_creator>,
    atlas_index_difficulty: usize,
    atlas_index_audio_settings: usize,
    atlas_index_menu_signs: usize,
}

impl<'texture_creator> SceneMainScreen<'texture_creator> {
    pub fn new<T>(
        game_data: Rc<GameData>,
        global_state: GameStateGlobal,
        state: Option<(FadeState, Box<SceneStateMainScreen>)>,
        render_configuration: &RenderConfiguration,
        canvas: &mut Canvas<Window>,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        let texture_screen =
            texture_creator.create_texture_target(PixelFormat::RGBA8888, WIDTH, HEIGHT)?;

        let (fade_state, mut scene_state) = match state {
            Some((fade_state, scene_state)) => (Some(fade_state), scene_state),
            None => (
                None,
                Box::new(SceneStateMainScreen {
                    choice: None,
                    scroller: Default::default(),
                    eyes: Default::default(),
                }),
            ),
        };

        let mut atlas_builder = SdlAtlasBuilder::new();
        let atlas_index_difficulty = atlas_builder.add_sprite(&game_data.main_menu.ratings);
        let atlas_index_audio_settings =
            atlas_builder.add_sprite(&game_data.main_menu.audio_settings);
        let atlas_index_audio_signs = atlas_builder.add_sprite(&game_data.main_menu.signs);

        let mut atlas = atlas_builder.build(&MENU_SCREEN_PALETTE, texture_creator)?;
        atlas.apply_blend_mode(SDL_BLENDMODE_NONE);

        let scroller = Scroller::new(
            texture_creator,
            &game_data,
            &mut scene_state.scroller,
            render_configuration,
        )?;

        let eyes = Eyes::new(
            texture_creator,
            &game_data,
            &mut scene_state.eyes,
            render_configuration,
        )?;

        let fade_controller =
            FadeController::new(fade_state, FADEIN_MUSEC_MENU, FADEOUT_MUSEC_MENU);

        let mut scene = Self {
            game_data,
            global_state,
            state: scene_state,
            texture_screen,
            redraw: Redraw::all(),
            fade_controller,
            scroller,
            eyes,
            atlas,
            atlas_index_audio_settings,
            atlas_index_difficulty,
            atlas_index_menu_signs: atlas_index_audio_signs,
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

        with_texture_canvas(canvas, &mut self.texture_screen, |canvas| {
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
            SceneEvent::KeyDown {
                keycode: Keycode::Up,
                ..
            } => {
                self.global_state.difficulty = self.global_state.difficulty.next();
                self.global_state.current_level =
                    LEVELS_PER_DIFFICULTY * self.global_state.difficulty as usize;

                self.redraw.insert(Redraw::DIFFICULTY);
            }
            SceneEvent::KeyDown {
                keycode: Keycode::Down,
                ..
            } => {
                self.global_state.difficulty = self.global_state.difficulty.previous();
                self.global_state.current_level =
                    LEVELS_PER_DIFFICULTY * self.global_state.difficulty as usize;

                self.redraw.insert(Redraw::DIFFICULTY);
            }
            SceneEvent::KeyDown {
                keycode: Keycode::F3,
                ..
            } => {
                self.global_state.audio_setting = self.global_state.audio_setting.next();
                self.redraw.insert(Redraw::AUDIO_SETTING);
            }
            _ => (),
        }
    }

    fn tick(&mut self, _canvas: &mut Canvas<Window>, clock_musec: u64) -> Result<()> {
        self.fade_controller.tick(clock_musec);

        if self.scroller.tick(clock_musec, &mut self.state.scroller) {
            self.redraw.insert(Redraw::SCROLLER);
        }

        if self.eyes.tick(clock_musec, &mut self.state.eyes) {
            self.redraw.insert(Redraw::EYES);
        }

        Ok(())
    }

    fn next_tick_at_musec(&self) -> Option<u64> {
        let next_tick_fade = self.fade_controller.next_tick();
        let next_tick_scroller = self.scroller.next_tick_at_musec(&self.state.scroller);
        let next_tick_eyes = self.eyes.next_tick_at_musec(&self.state.eyes);

        [
            next_tick_fade,
            Some(next_tick_scroller),
            Some(next_tick_eyes),
        ]
        .into_iter()
        .flatten()
        .min()
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
            if self.redraw.intersects(Redraw::EYES) {
                self.eyes.render(&self.state.eyes, canvas)?;
            }

            if self.redraw.intersects(Redraw::SCROLLER) {
                self.scroller.render(&self.state.scroller, canvas)?;
            }

            if self.redraw.intersects(Redraw::DIFFICULTY) {
                self.atlas.blit(
                    canvas,
                    self.atlas_index_difficulty,
                    0x1e6,
                    0x91,
                    3 - self.global_state.difficulty as usize,
                    false,
                    false,
                )?;
            }

            if self.redraw.intersects(Redraw::AUDIO_SETTING) {
                self.atlas.blit(
                    canvas,
                    self.atlas_index_menu_signs,
                    324,
                    120,
                    2,
                    false,
                    false,
                )?;

                if self.global_state.audio_setting != AudioSetting::None {
                    let iframe = if self.global_state.audio_setting == AudioSetting::Music {
                        0
                    } else {
                        1
                    };
                    self.atlas.blit(
                        canvas,
                        self.atlas_index_audio_settings,
                        0x15f,
                        0x92,
                        iframe,
                        false,
                        false,
                    )?;
                }
            }

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
