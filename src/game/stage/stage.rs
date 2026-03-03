use std::mem::transmute;
use std::rc::Rc;
use std::time::{Duration, Instant};

use anyhow::Result;
use rustlings::game_data::GameData;
use rustlings::sdl_rendering::with_texture_canvas;
use rustlings::sdl3_aux::{current_refresh_rate, is_main_thread};
use sdl3::mouse::MouseState;
use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::{ScaleMode, Texture};
use sdl3::sys::video::SDL_WindowFlags;
use sdl3::{
    Sdl,
    event::{Event, EventWatchCallback, WindowEvent},
    rect::Rect as SdlRect,
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
};

use crate::scene::{CursorType, Scene};
use crate::stage::event_collector::{DecodedEvent, EventCollector};
use crate::stage::render_state::{Layer, PrescalingMode, RenderState, StaticTexture};

const MAX_TIMESLICE_MSEC: u64 = 100;
const FALLBACK_MIN_EVENT_AGGREGATION_TIME_MULLIS: u64 = 5;
const REFRESH_RATE_SAFETY_FACTOR: f32 = 0.8;

pub enum RunResult {
    Quit,
    RenderReset,
    NextScene,
}

pub struct Stage<'sdl> {
    sdl_context: &'sdl Sdl,
    canvas: &'sdl mut Canvas<Window>,
    texture_creator: &'sdl TextureCreator<WindowContext>,
    game_data: Rc<GameData>,
}

impl<'sdl> Stage<'sdl> {
    pub fn new(
        sdl_context: &'sdl Sdl,
        canvas: &'sdl mut Canvas<Window>,
        texture_creator: &'sdl TextureCreator<WindowContext>,
        game_data: Rc<GameData>,
    ) -> Self {
        Self {
            sdl_context,
            canvas,
            texture_creator,
            game_data,
        }
    }

    pub fn run(&mut self, scene: &mut dyn Scene<'sdl>) -> Result<RunResult> {
        let mut render_state = RenderState::new(scene, &self.game_data, self.texture_creator)?;

        scene.register_layers(&mut render_state);

        let expose_watch = ExposeWatch::new(self, &mut render_state, scene);
        let mut event_watch = self.sdl_context.event()?.add_event_watch(expose_watch);
        event_watch.deactivate();

        let mut redraw = true;
        let mut ts_reference = Instant::now();
        let mut time_old = 0;

        let mouse_state = MouseState::new(&self.sdl_context.event_pump()?);
        render_state.mouse_x = mouse_state.x();
        render_state.mouse_y = mouse_state.y();

        let mut event_collector = EventCollector::new();

        loop {
            let ts = Instant::now();

            let mut time = ts.duration_since(ts_reference).as_millis() as u64;
            if time - time_old > MAX_TIMESLICE_MSEC {
                ts_reference += Duration::from_millis(time - time_old - MAX_TIMESLICE_MSEC);
                time = ts.duration_since(ts_reference).as_millis() as u64;
            }

            time_old = time;
            scene.tick(time);

            if scene.is_complete() {
                return Ok(RunResult::NextScene);
            }

            redraw = scene.draw(&mut self.canvas)? || redraw;
            if redraw {
                self.render(&mut render_state, scene)?;
                redraw = false;
            }

            event_watch.activate();
            event_collector.collect_events(
                self.min_event_aggregation_time_millis(),
                scene.next_tick_at_msec().saturating_sub(time),
                &mut self.sdl_context.event_pump()?,
            );
            event_watch.deactivate();

            let mut toggle_fullscreen = false;
            for event in event_collector.decoded_events().as_ref() {
                match *event {
                    DecodedEvent::Quit => return Ok(RunResult::Quit),
                    DecodedEvent::RenderReset => return Ok(RunResult::RenderReset),
                    DecodedEvent::Redraw => redraw = true,
                    DecodedEvent::ToggleFullscreen => toggle_fullscreen = !toggle_fullscreen,
                    DecodedEvent::DispatchSceneEvent(event) => scene.dispatch_event(event),
                    DecodedEvent::MouseMove { x, y, .. } => {
                        render_state.mouse_x = x;
                        render_state.mouse_y = y;
                        redraw = true;
                    }
                }
            }

            if toggle_fullscreen {
                let is_fullscreen =
                    self.canvas.window().window_flags().0 & SDL_WindowFlags::FULLSCREEN.0 != 0;

                self.canvas
                    .window()
                    .clone()
                    .set_fullscreen(!is_fullscreen)?;
            }
        }
    }

    fn min_event_aggregation_time_millis(&self) -> u64 {
        let refresh_rate = current_refresh_rate(&self.canvas.window());

        if refresh_rate > 0. {
            (1000. / refresh_rate * REFRESH_RATE_SAFETY_FACTOR).round() as u64
        } else {
            FALLBACK_MIN_EVENT_AGGREGATION_TIME_MULLIS
        }
    }

    fn render(
        &mut self,
        render_state: &mut RenderState<'sdl>,
        scene: &mut dyn Scene<'sdl>,
    ) -> Result<()> {
        let (canvas_width, canvas_height) = self.canvas.output_size()?;
        render_state.update_layout(canvas_width as usize, canvas_height as usize);

        self.canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        self.canvas.clear();

        for i in 0..render_state.layers.len() {
            let layer = render_state
                .layers
                .get_mut(i)
                .ok_or(anyhow::format_err!("invalid layer {}", i))?;

            let dest = render_state
                .layout
                .layers
                .get(i)
                .ok_or(anyhow::format_err!("no layout for layer {}", i))?;

            let texture = self.prescale_layer(scene, layer)?;
            texture.set_blend_mode(sdl3::render::BlendMode::Blend);
            let _ = self.canvas.copy(texture, None, Some(dest.into()))?;
        }

        if scene.cursor_type() != CursorType::None {
            let layout_cursor = render_state.layout.cursor.clone();
            let pixel_scale = self.canvas.window().display_scale();

            let mouse_x = render_state.mouse_x * pixel_scale;
            let mouse_y = render_state.mouse_y * pixel_scale;

            let static_texture = render_state.get_cursor(scene.cursor_type()).unwrap();

            let texture =
                self.prescale_static_texture(static_texture, layout_cursor.prescaling_mode)?;
            texture.set_blend_mode(sdl3::render::BlendMode::Blend);

            let _ = self.canvas.copy(
                texture,
                None,
                SdlRect::new(
                    mouse_x as i32 - layout_cursor.center_x as i32,
                    mouse_y as i32 - layout_cursor.center_y as i32,
                    layout_cursor.width as u32,
                    layout_cursor.height as u32,
                ),
            )?;
        }

        self.canvas.present();
        Ok(())
    }

    fn prescale_layer<'scene: 'layer, 'layer>(
        &mut self,
        scene: &'scene mut dyn Scene<'sdl>,
        layer: &'layer mut Layer<'sdl>,
    ) -> Result<&'layer mut Texture<'sdl>> {
        self.prescale_texture(
            scene.texture(layer.texture_id)?,
            &mut layer.intermediate_texture,
            layer.prescaling_mode,
            None,
        )
    }

    fn prescale_static_texture<'a>(
        &mut self,
        static_texture: &'a mut StaticTexture<'sdl>,
        prescaling_mode: PrescalingMode,
    ) -> Result<&'a mut Texture<'sdl>> {
        let prescaled_texture = self.prescale_texture(
            &mut static_texture.texture,
            &mut static_texture.intermediate_texture,
            prescaling_mode,
            Some(static_texture.prescaling_mode),
        )?;

        static_texture.prescaling_mode = prescaling_mode;

        Ok(prescaled_texture)
    }

    fn prescale_texture<'a>(
        &mut self,
        source_texture: &'a mut Texture<'sdl>,
        maybe_intermediate_texture: &'a mut Option<Texture<'sdl>>,
        prescaling_mode: PrescalingMode,
        previous_prescaling_mode: Option<PrescalingMode>,
    ) -> Result<&'a mut Texture<'sdl>> {
        let (integer_scaled_width, integer_scaled_height) = match prescaling_mode {
            PrescalingMode::None(scaling_mode) => {
                source_texture.set_scale_mode(scaling_mode);
                return Ok(source_texture);
            }
            PrescalingMode::Quis(width, height) => (width as u32, height as u32),
        };

        let needs_recreate = match maybe_intermediate_texture.as_ref() {
            Some(texture)
                if texture.width() == integer_scaled_width
                    && texture.height() == integer_scaled_height =>
            {
                false
            }
            _ => true,
        };

        if needs_recreate {
            *maybe_intermediate_texture = Some(self.texture_creator.create_texture_target(
                PixelFormat::RGBA8888,
                integer_scaled_width,
                integer_scaled_height,
            )?);
        }

        let intermediate_texture = maybe_intermediate_texture.as_mut().unwrap();

        let already_prescaled = match previous_prescaling_mode {
            Some(mode) => mode == prescaling_mode,
            None => false,
        };

        if needs_recreate || !already_prescaled {
            source_texture.set_scale_mode(ScaleMode::Nearest);
            intermediate_texture.set_scale_mode(ScaleMode::Linear);

            source_texture.set_blend_mode(sdl3::render::BlendMode::None);
            with_texture_canvas(self.canvas, intermediate_texture, |canvas| -> Result<()> {
                canvas
                    .copy(
                        source_texture,
                        None,
                        SdlRect::new(0, 0, integer_scaled_width, integer_scaled_height),
                    )
                    .map_err(anyhow::Error::from)
            })?;
        }

        Ok(intermediate_texture)
    }
}

struct ExposeWatch {
    stage: *mut Stage<'static>,
    render_state: *mut RenderState<'static>,
    scene: *mut dyn Scene<'static>,
}
unsafe impl Send for ExposeWatch {}

impl EventWatchCallback for ExposeWatch {
    fn callback(&mut self, event: Event) {
        // https://wiki.libsdl.org/SDL3/SDL_SetEventFilter states EXPOSE is guranteed
        // to be dispatched on the main thread, but making this explicit doesn't hurt
        if !is_main_thread() {
            return;
        }

        match event {
            Event::Window {
                win_event: WindowEvent::Exposed,
                ..
            } => (),
            _ => return,
        }

        unsafe {
            let _ = (*self.stage).render(&mut *self.render_state, &mut *self.scene);
        }
    }
}

impl ExposeWatch {
    pub fn new(stage: &mut Stage, render_state: &mut RenderState, scene: &mut dyn Scene) -> Self {
        // SAFETY: EventWatchCallbacks are declared as Send + 'static, which makes sense for the
        // general case: they are sent to another thread and live until the watch is removed.
        // However, we don't need this as we 1. we check whether we are on the main thread before
        // doing anything and 2. drop and destroy the watch before the dependencies are dropped
        // or moved.
        ExposeWatch {
            stage: (stage as *mut _) as *mut Stage<'static>,
            render_state: (render_state as *mut _) as *mut RenderState<'static>,
            scene: unsafe {
                transmute::<*mut dyn Scene<'_>, *mut dyn Scene<'static>>(scene as *mut _)
            },
        }
    }
}
