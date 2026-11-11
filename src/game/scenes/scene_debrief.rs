use std::rc::Rc;

use anyhow::{Result, anyhow};
use rustlings::{
    game_data::{GameData, LevelParameters, NUM_LEVELS, decode_level_index},
    sdl::with_texture_canvas,
};
use sdl3::{
    keyboard::Keycode,
    pixels::PixelFormat,
    render::{Canvas, Texture, TextureCreator},
    video::Window,
};

use crate::{
    code::code_for_level,
    geometry::Rect,
    scene::{Compositor, CursorType, MouseButton, Scene, SceneEvent},
    scenes::common::{
        FADEIN_MSEC_MENU, FADEOUT_MSEC_MENU, FadeController, TextGrid, draw_background,
    },
    state::{
        self, ChoiceDebrief, FadeState, GameState, GameStateGlobal, Scene::Briefing,
        SceneStateDebrief,
    },
};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 350;

const TEXTURE_ID_SCREEN: usize = 0;

pub struct SceneDebrief<'texture_creator> {
    game_data: Rc<GameData>,
    global_state: GameStateGlobal,

    state: SceneStateDebrief,

    texture_screen: Texture<'texture_creator>,
    draw_complete: bool,

    fade_controller: FadeController,
    level_parameters: LevelParameters,
}

impl<'texture_creator> SceneDebrief<'texture_creator> {
    pub fn new<T>(
        game_data: Rc<GameData>,
        global_state: GameStateGlobal,
        state: Option<(FadeState, SceneStateDebrief)>,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        let texture_screen =
            texture_creator.create_texture_target(PixelFormat::RGBA8888, WIDTH, HEIGHT)?;

        let (fade_state, scene_state) = match state {
            Some((fade_state, scene_state)) => (Some(fade_state), scene_state),
            None => (None, SceneStateDebrief { choice: None }),
        };

        let level = game_data.resolve_level(global_state.current_level)?;

        let fade_controller = FadeController::new(fade_state, FADEIN_MSEC_MENU, FADEOUT_MSEC_MENU);

        Ok(Self {
            game_data,
            global_state,
            fade_controller,
            state: scene_state,
            texture_screen,
            draw_complete: false,
            level_parameters: level.parameters,
        })
    }

    fn level_solved(&self) -> bool {
        self.global_state.last_saved_percent >= self.level_parameters.percent_required()
    }

    fn next_level(&self) -> usize {
        (self.global_state.current_level + 1) % NUM_LEVELS
    }

    fn write_debrief(&self, text_grid: &mut TextGrid) {
        if self.global_state.last_time_remaining_seconds > 0 {
            text_grid.write(6, 1, "All lemmings accounted for.");
        } else {
            text_grid.write(12, 1, "Your time is up!");
        }

        let saved_required = self.level_parameters.percent_required();
        let saved_percent = self.global_state.last_saved_percent;

        text_grid.write(12, 3, &format!("You rescued {}%", saved_percent));
        text_grid.write(12, 4, &format!("You needed  {}%", saved_required));

        if saved_percent >= 100 {
            text_grid.write(2, 7, "Superb! You rescued every lemming on");
            text_grid.write(2, 8, "that level. Can you do it again....?");
        } else if saved_percent == 0 {
            text_grid.write(3, 7, "ROCK BOTTOM! I hope for your sake");
            text_grid.write(7, 8, "that you nuked that level.");
        } else if saved_percent < saved_required / 2 {
            text_grid.write(2, 7, "Better rethink your strategy before");
            text_grid.write(7, 8, "you try this level again!");
        } else if saved_percent < saved_required.saturating_sub(5) {
            text_grid.write(2, 7, "A little more practice on this level");
            text_grid.write(7, 8, "is definitely recommended.");
        } else if saved_percent < saved_required.saturating_sub(1) {
            text_grid.write(3, 7, "You got pretty close that time.");
            text_grid.write(2, 8, "Now try again for that few % extra");
        } else if saved_percent == saved_required.saturating_sub(1) {
            text_grid.write(1, 7, "OH NO, So near and yet so far (teehee)");
            text_grid.write(10, 8, "Maybe this time.....");
        } else if saved_percent == saved_required {
            text_grid.write(2, 7, "RIGHT ON. You can't get much closer");
            text_grid.write(3, 8, "than that. Let's try the next....");
        } else if saved_percent < saved_required + 20 {
            text_grid.write(1, 7, "That level seemed no problem to you on");
            text_grid.write(1, 8, "that attempt. Onto the next....");
        } else {
            text_grid.write(4, 7, "You totally stormed that level!");
            text_grid.write(1, 8, "Let's see if you can storm the next...");
        }

        if self.level_solved() {
            let (_, level_index) = decode_level_index(self.next_level());
            let code = code_for_level(
                self.next_level(),
                Some(saved_percent),
                Some(self.global_state.code_salt),
            )
            .expect("internal error during code generation");

            text_grid.write(
                4,
                14,
                &format!("Your Access Code for Level {}", level_index),
            );
            text_grid.write(12, 15, &format!("is {}", code));

            text_grid.write(1, 18, "Press left mouse button for next level");
        } else {
            text_grid.write(1, 18, "Press left mouse button to retry level");
        }

        text_grid.write(3, 19, "Press right mouse button for menu");
    }
}

impl<'texture_creator> Scene<'texture_creator> for SceneDebrief<'texture_creator> {
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

    fn cursor_type(&mut self) -> CursorType {
        CursorType::None
    }

    fn dispatch_event(&mut self, event: SceneEvent) {
        if self.state.choice.is_some() {
            return;
        }

        match event {
            SceneEvent::KeyDown {
                keycode: Keycode::Space | Keycode::Return | Keycode::KpEnter,
                ..
            }
            | SceneEvent::MouseDown(MouseButton::Left, _) => {
                self.fade_controller.start_fadeout();
                self.state.choice = Some(ChoiceDebrief::Continue);
            }
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
        self.fade_controller.fadeout_done()
    }

    fn texture(&mut self, id: usize) -> Result<&mut Texture<'texture_creator>> {
        match id {
            TEXTURE_ID_SCREEN => Ok(&mut self.texture_screen),
            _ => Err(anyhow!("no texture with ID {}", id)),
        }
    }

    fn register_layers(&self, compositor: &mut dyn Compositor) {
        compositor.add_layer(
            TEXTURE_ID_SCREEN,
            WIDTH,
            HEIGHT,
            Rect::new(0, 0, WIDTH, HEIGHT),
        );
    }

    fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<bool> {
        if self.draw_complete {
            return Ok(false);
        }

        let texture_creator = &canvas.texture_creator();

        let mut text_grid = TextGrid::new(WIDTH, HEIGHT);
        self.write_debrief(&mut text_grid);

        with_texture_canvas(canvas, &mut self.texture_screen, |canvas| {
            draw_background(0, &self.game_data, texture_creator, canvas)?;
            text_grid.render(&self.game_data, texture_creator, canvas)?;

            Ok(())
        })?;

        self.draw_complete = true;

        Ok(true)
    }

    fn will_redraw(&self) -> bool {
        false
    }

    fn finish(self: Box<Self>) -> GameState {
        match self.state.choice {
            Some(ChoiceDebrief::Continue) if self.fade_controller.fadeout_done() => GameState {
                global: GameStateGlobal {
                    current_level: if self.level_solved() {
                        self.next_level()
                    } else {
                        self.global_state.current_level
                    },
                    ..self.global_state
                },
                scene: Briefing(None),
            },
            _ => GameState {
                global: self.global_state,
                scene: state::Scene::Debrief(Some((self.fade_controller.finish(), self.state))),
            },
        }
    }
}
