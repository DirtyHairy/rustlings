use std::rc::Rc;

use anyhow::{Result, anyhow};
use rustlings::{
    game_data,
    game_data::{
        ENGINE_TICK_MSEC, GameData, LEVEL_HEIGHT, LEVEL_WIDTH, Level, NUM_LEVELS,
        decode_level_index, file::ground::InteractionType,
    },
    sdl::with_texture_canvas,
};
use sdl3::{
    keyboard::Keycode,
    pixels::PixelFormat,
    render::{Canvas, Texture, TextureCreator},
    video::Window,
};

use crate::{
    geometry,
    scene::{CursorType, MouseButton, Scene, SceneEvent},
    scenes::common::{
        FADEIN_MSEC_MENU, FADEOUT_MSEC_MENU, FadeController, LevelRenderer, TextGrid,
        draw_background,
    },
    state::{
        self, ChoiceBriefing, FadeState, GameState, GameStateGlobal, ObjectState,
        SceneStateBriefing,
    },
};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 350;
const BACKGROUND_START: u32 = 78;

const TEXTURE_ID_SCREEN: usize = 0;
const TEXTURE_ID_LEVEL: usize = 1;

struct Object {
    animate: bool,
    first_frame: usize,
    animation_frames: usize,
}

impl Object {
    fn new(object: &game_data::Object, level: &Level, game_data: &GameData) -> Result<Self> {
        let info = game_data.resolve_object(object.id as usize, level.graphics_set as usize)?;

        let animate = info.animation_loops && info.interaction_type != InteractionType::Entrance;

        let first_frame = if info.interaction_type == InteractionType::Entrance {
            info.animation_start
        } else {
            0
        };

        Ok(Object {
            animate,
            first_frame,
            animation_frames: info.animation_end,
        })
    }
}

impl ObjectState {
    fn new(object: &Object) -> Self {
        ObjectState {
            triggered: false,
            frame: object.first_frame,
        }
    }
}

pub struct SceneBriefing<'texture_creator> {
    game_data: Rc<GameData>,
    global_state: GameStateGlobal,

    texture_screen: Texture<'texture_creator>,
    static_draw_complete: bool,
    needs_level_draw: bool,

    state: SceneStateBriefing,
    level: Level,
    objects: Vec<Object>,
    clock_offset_msec: u64,

    level_renderer: LevelRenderer<'texture_creator>,
    fade_controller: FadeController,
}

impl<'texture_creator> SceneBriefing<'texture_creator> {
    pub fn new<T>(
        game_data: Rc<GameData>,
        global_state: GameStateGlobal,
        state: Option<(FadeState, Option<SceneStateBriefing>)>,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        let texture_screen =
            texture_creator.create_texture_target(PixelFormat::RGBA8888, WIDTH, HEIGHT)?;

        let level = game_data.resolve_level(global_state.current_level)?;
        let objects: Vec<Object> = level
            .objects
            .iter()
            .map(|o| Object::new(o, &level, &game_data))
            .collect::<Result<_>>()?;

        let (fade_state, scene_state) = match state {
            Some((fade_state, Some(scene_state))) => (Some(fade_state), scene_state),
            Some((fade_state, None)) => (
                Some(fade_state),
                SceneStateBriefing {
                    clock_msec: 0,
                    choice: None,
                    objects: objects.iter().map(ObjectState::new).collect(),
                },
            ),
            None => (
                None,
                SceneStateBriefing {
                    clock_msec: 0,
                    choice: None,
                    objects: objects.iter().map(ObjectState::new).collect(),
                },
            ),
        };

        let terrain = game_data.compose_terrain(&level)?;
        let level_renderer =
            LevelRenderer::new(&level, &terrain, Rc::clone(&game_data), texture_creator)?;

        let fade_controller = FadeController::new(fade_state, FADEIN_MSEC_MENU, FADEOUT_MSEC_MENU);

        Ok(Self {
            clock_offset_msec: scene_state.clock_msec,

            game_data,
            global_state,
            static_draw_complete: false,
            needs_level_draw: true,

            texture_screen,

            level,
            state: scene_state,
            objects,

            level_renderer,
            fade_controller,
        })
    }

    fn draw_static(&mut self, canvas: &mut Canvas<Window>) -> Result<bool> {
        if self.static_draw_complete {
            return Ok(false);
        }

        let texture_creator = &canvas.texture_creator();

        let mut text_grid = TextGrid::new(WIDTH, HEIGHT);
        self.write_briefing(&mut text_grid);

        with_texture_canvas(canvas, &mut self.texture_screen, |canvas| {
            draw_background(BACKGROUND_START, &self.game_data, texture_creator, canvas)?;
            text_grid.render(&self.game_data, texture_creator, canvas)?;

            Ok(())
        })?;

        self.static_draw_complete = true;

        Ok(true)
    }

    fn draw_level(&mut self, canvas: &mut Canvas<Window>) -> Result<bool> {
        if !self.needs_level_draw {
            return Ok(false);
        }

        self.level_renderer
            .draw(&self.state.objects, None, canvas)?;

        self.needs_level_draw = false;

        Ok(true)
    }

    fn write_briefing(&self, grid: &mut TextGrid) {
        let level = &self.level;

        let (difficulty, index) = decode_level_index(self.global_state.current_level);
        let name_indent = (40_u32.saturating_sub(level.parameters.name.len() as u32) / 2).max(9);

        grid.write(0, 5, &format!("Level {}", index));
        grid.write(name_indent, 5, &level.parameters.name);
        grid.write(
            10,
            8,
            &format!("Number of Lemmings {}", level.parameters.released),
        );
        grid.write(
            10,
            10,
            &format!(
                "{:4} To Be Saved",
                format!("{}%", level.parameters.percent_required())
            ),
        );
        grid.write(
            10,
            12,
            &format!("Release Rate {}", level.parameters.release_rate),
        );
        grid.write(
            10,
            14,
            &format!("Time {} Minutes", level.parameters.time_limit),
        );
        grid.write(10, 16, &format!("Rating  {}", difficulty));
        grid.write(5, 19, "Press mouse button to continue");
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
        self.fade_controller.opacity()
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
            } if self.state.choice.is_none() => {
                self.state.choice = Some(ChoiceBriefing::PreviousLevel);
            }
            SceneEvent::KeyDown {
                keycode: Keycode::PageUp,
                ..
            } if self.state.choice.is_none() => self.state.choice = Some(ChoiceBriefing::NextLevel),
            SceneEvent::KeyDown {
                keycode: Keycode::Space | Keycode::Return | Keycode::KpEnter,
                ..
            }
            | SceneEvent::MouseDown(MouseButton::Left, _)
                if self.state.choice.is_none() =>
            {
                self.fade_controller.start_fadeout();
                self.state.choice = Some(ChoiceBriefing::Launch);
            }
            _ => (),
        };
    }

    fn tick(&mut self, _canvas: &mut Canvas<Window>, mut clock_msec: u64) -> Result<()> {
        self.fade_controller.tick(clock_msec);

        clock_msec += self.clock_offset_msec;

        let clock_msec_old = self.state.clock_msec;
        self.state.clock_msec = clock_msec;

        let engine_ticks_old = clock_msec_old / ENGINE_TICK_MSEC;
        let engine_ticks = clock_msec / ENGINE_TICK_MSEC;

        let delta = engine_ticks.saturating_sub(engine_ticks_old) as usize;
        if delta > 0 {
            for (object_state, object) in self.state.objects.iter_mut().zip(&self.objects) {
                if !object.animate {
                    continue;
                }

                object_state.frame = (object_state.frame + delta) % (object.animation_frames);
            }

            self.needs_level_draw = true;
        }

        Ok(())
    }

    fn next_tick_at_msec(&self) -> Option<u64> {
        let mut next_tick = (self.state.clock_msec / ENGINE_TICK_MSEC + 1) * ENGINE_TICK_MSEC;

        if let Some(next_tick_fade) = self.fade_controller.next_tick_at_msec() {
            next_tick = next_tick.min(next_tick_fade + self.clock_offset_msec);
        }

        Some(next_tick.saturating_sub(self.clock_offset_msec))
    }

    fn is_complete(&self) -> bool {
        self.state.choice.is_some()
            && (!self.fade_controller.fadeout_started() || self.fade_controller.fadeout_done())
    }

    fn texture(&mut self, id: usize) -> Result<&mut sdl3::render::Texture<'texture_creator>> {
        match id {
            TEXTURE_ID_SCREEN => Ok(&mut self.texture_screen),
            TEXTURE_ID_LEVEL => Ok(self.level_renderer.texture()),
            _ => Err(anyhow!("no texture with ID {}", id)),
        }
    }

    fn register_layers(&self, compositor: &mut dyn crate::scene::Compositor) {
        compositor.add_layer(
            TEXTURE_ID_SCREEN,
            WIDTH,
            HEIGHT,
            geometry::Rect {
                x: 0,
                y: 0,
                width: WIDTH,
                height: HEIGHT,
            },
        );

        let level_width_scaled = (LEVEL_WIDTH * 2) / 5;
        let level_height_scaled = (LEVEL_HEIGHT * 2) / 5;
        let preview_x = WIDTH.saturating_sub(level_width_scaled) / 2;
        let preview_y = BACKGROUND_START.saturating_sub(level_height_scaled) / 2;
        compositor.add_layer(
            TEXTURE_ID_LEVEL,
            LEVEL_WIDTH,
            LEVEL_HEIGHT,
            geometry::Rect {
                x: preview_x,
                y: preview_y,
                width: level_width_scaled,
                height: level_height_scaled,
            },
        );
    }

    fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<bool> {
        let static_draw = self.draw_static(canvas)?;
        let level_draw = self.draw_level(canvas)?;

        Ok(static_draw || level_draw)
    }

    fn will_redraw(&self) -> bool {
        false
    }

    fn finish(self: Box<Self>) -> state::GameState {
        match self.state.choice {
            Some(ChoiceBriefing::Launch) if self.fade_controller.fadeout_done() => GameState {
                global: self.global_state,
                scene: state::Scene::Level(None),
            },
            Some(ChoiceBriefing::NextLevel) => GameState {
                global: GameStateGlobal {
                    current_level: (self.global_state.current_level + 1) % NUM_LEVELS,
                    ..self.global_state
                },
                scene: state::Scene::Briefing(Some((self.fade_controller.finish(), None))),
            },
            Some(ChoiceBriefing::PreviousLevel) => GameState {
                global: GameStateGlobal {
                    current_level: (self.global_state.current_level + NUM_LEVELS - 1) % NUM_LEVELS,
                    ..self.global_state
                },
                scene: state::Scene::Briefing(Some((self.fade_controller.finish(), None))),
            },
            _ => GameState {
                global: self.global_state,
                scene: state::Scene::Briefing(Some((
                    self.fade_controller.finish(),
                    Some(self.state),
                ))),
            },
        }
    }
}
