use std::marker::PhantomData;
use std::mem::transmute;
use std::rc::Rc;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use rustlings::game_data::GameData;
use rustlings::sdl::{is_main_thread, with_texture_canvas};
use sdl3::event::EventWatch;
use sdl3::mouse::MouseState;
use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::{ScaleMode, Texture};
use sdl3::sys::video::{SDL_WINDOW_FULLSCREEN, SDL_WINDOW_MOUSE_FOCUS, SDL_WindowFlags};
use sdl3::{
    Sdl,
    event::{Event, EventWatchCallback, WindowEvent},
    rect::Rect as SdlRect,
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
};

use crate::scene::{self, CursorType, MouseCoordinates, Scene, SceneEvent};
use crate::stage::event_collector::{EventCollector, GameEvent, MouseButton, MouseOwnerChange};
use crate::stage::render_state::{Layer, PrescalingMode, RenderState, StaticTexture};
use crate::timing::RenderConfiguration;

const MAX_TIMESLICE_EXCESS_MUSEC: u64 = 100000;
const FRAME_SAFE_BOUNDARY_FRACTION: u64 = 10;
const USE_FRAME_SYNC_THRESHOLD_MUSEC: u64 = 100000;

pub enum StopReason {
    Quit,
    RenderReset,
    RenderConfigChange,
    NextScene,
}

pub struct Stage<'sdl> {
    sdl_context: &'sdl Sdl,
    canvas: &'sdl mut Canvas<Window>,
    texture_creator: &'sdl TextureCreator<WindowContext>,
    game_data: Rc<GameData>,

    last_opacity: Option<u8>,
    rerender: bool,
    suspended: bool,
    speed: u32,

    ts_reference: Instant,
    time_offset_musec: u64,

    render_configuration: RenderConfiguration,
}

impl From<MouseButton> for scene::MouseButton {
    fn from(btn: MouseButton) -> Self {
        match btn {
            MouseButton::Left => Self::Left,
            MouseButton::Right => Self::Right,
        }
    }
}

impl<'sdl> Stage<'sdl> {
    pub fn new(
        sdl_context: &'sdl Sdl,
        canvas: &'sdl mut Canvas<Window>,
        texture_creator: &'sdl TextureCreator<WindowContext>,
        render_configuration: RenderConfiguration,
        game_data: Rc<GameData>,
    ) -> Self {
        Self {
            sdl_context,
            canvas,
            texture_creator,
            game_data,
            last_opacity: None,
            rerender: false,
            suspended: false,
            ts_reference: Instant::now(),
            time_offset_musec: 0,
            speed: 1,
            render_configuration,
        }
    }

    pub fn run(&mut self, scene: &mut dyn Scene<'sdl>) -> Result<StopReason> {
        let mut render_state = RenderState::new(scene, &self.game_data, self.texture_creator)?;

        scene.register_layers(&mut render_state);

        let mut event_collector = EventCollector::new(scene.emulate_right_mouse_buttom());

        self.rerender = true;
        self.suspended = false;
        self.ts_reference = Instant::now();
        self.time_offset_musec = 0;

        self.transfer_mouse_state(&mut render_state)?;

        scene.set_is_fullscreen(self.is_fullscreen());
        scene.set_mouse_enabled(render_state.mouse_enabled);

        self.update_layout(&mut render_state)?;
        scene.dispatch_event(SceneEvent::MouseMove(self.mouse_coordinates_from(
            &render_state,
            scene,
            render_state.mouse_x,
            render_state.mouse_y,
        )));

        let mut last_tick_musec: u64 = 0;
        let mut aggregation_timeout_musec: Option<u64> = None;

        loop {
            if !self.suspended {
                self.render_scene(scene, &mut render_state)?;
            }

            // start of the next frame
            let ts_frame_start = Instant::now();

            // elapsed game time
            let mut time_musec: u64 =
                self.time_since_reference_musec(ts_frame_start) + self.time_offset_musec;

            // limit jumps in game time by adjusting the offset if necessary (can happen if the
            // game blocks, e.g. while a window is being resize-dragged)
            self.limit_time_step(
                &mut time_musec,
                last_tick_musec,
                aggregation_timeout_musec,
                ts_frame_start,
            );

            if self.render_configuration.use_frame_sync && !self.suspended {
                // All timing critical update intervals are chosen to be multiples of a single frame.
                // If the tick coincides with a frame boundary we enter an unstable state where
                // the timing loop alternates between two ticks per interval and zero ticks per interval.
                // We compensate by adjusting the phase of the game clock such that the beginning of a new
                // frame (in game time) lies in the middle of the frame window (in real time).
                self.adjust_tick_phase(&mut time_musec, ts_frame_start);
            }

            if !self.suspended {
                last_tick_musec = time_musec;
                scene.tick(self.canvas, time_musec)?;

                if scene.is_complete() {
                    return Ok(StopReason::NextScene);
                }
            }

            if let Some(stop_reason) =
                self.consume_events(&mut render_state, scene, &mut event_collector)?
            {
                return Ok(stop_reason);
            }

            if self.render_configuration.is_stale(self.canvas) {
                return Ok(StopReason::RenderConfigChange);
            }

            aggregation_timeout_musec = self.calculate_aggregation_timeout(scene, time_musec);

            let mut event_pump = self.sdl_context.event_pump()?;
            let time_budget_per_frame_musec = self.render_configuration.time_budget_per_frame_musec;
            let expose_watch = ExposeWatch::new(self, &mut render_state, scene)?;

            event_collector.collect_events(
                ts_frame_start + Duration::from_micros(time_budget_per_frame_musec),
                aggregation_timeout_musec,
                &mut event_pump,
            );

            drop(expose_watch);

            let mouse_was_enabled = render_state.mouse_enabled;
            self.update_mouse_state(&event_collector, &mut render_state);

            if mouse_was_enabled != render_state.mouse_enabled {
                scene.set_mouse_enabled(render_state.mouse_enabled);
            }
        }
    }

    fn limit_time_step(
        &mut self,
        time_musec: &mut u64,
        time_old_musec: u64,
        aggregation_timeout_musec: Option<u64>,
        now: Instant,
    ) {
        if let Some(aggregate_timeout_musec) = aggregation_timeout_musec
            && *time_musec - time_old_musec
                > (aggregate_timeout_musec + MAX_TIMESLICE_EXCESS_MUSEC) * self.speed as u64
        {
            self.ts_reference = now;
            self.time_offset_musec = time_old_musec
                + (aggregate_timeout_musec + MAX_TIMESLICE_EXCESS_MUSEC) * self.speed as u64;

            *time_musec = self.time_since_reference_musec(now) + self.time_offset_musec;
        }
    }

    fn adjust_tick_phase(&mut self, time_musec: &mut u64, now: Instant) {
        let frame_duration_musec = self.render_configuration.frame_duration_musec;

        if time_musec
            .next_multiple_of(frame_duration_musec)
            .saturating_sub(*time_musec)
            < frame_duration_musec / FRAME_SAFE_BOUNDARY_FRACTION
        {
            self.time_offset_musec += time_musec.next_multiple_of(frame_duration_musec)
                + frame_duration_musec / 2
                - *time_musec;

            *time_musec = self.time_since_reference_musec(now) + self.time_offset_musec;
        } else if time_musec
            .saturating_sub((*time_musec / frame_duration_musec) * frame_duration_musec)
            < frame_duration_musec / FRAME_SAFE_BOUNDARY_FRACTION
        {
            self.time_offset_musec += (*time_musec / frame_duration_musec) * frame_duration_musec
                + frame_duration_musec / 2
                - *time_musec;

            *time_musec = self.time_since_reference_musec(now) + self.time_offset_musec;
        }
    }

    fn calculate_aggregation_timeout(
        &self,
        scene: &mut dyn Scene<'sdl>,
        time_musec: u64,
    ) -> Option<u64> {
        if scene.will_redraw() || self.last_opacity != Some(scene.opacity()) {
            Some(0)
        } else {
            scene.next_tick_at_musec().map(|x| {
                let mut timeout = x.saturating_sub(time_musec) / self.speed as u64;
                if self.render_configuration.use_frame_sync
                    && timeout < USE_FRAME_SYNC_THRESHOLD_MUSEC
                {
                    timeout = 0;
                }

                timeout
            })
        }
    }

    fn time_since_reference_musec(&self, now: Instant) -> u64 {
        now.duration_since(self.ts_reference).as_micros() as u64 * self.speed as u64
    }

    fn transfer_mouse_state(&mut self, render_state: &mut RenderState) -> Result<()> {
        let mouse_state = MouseState::new(&self.sdl_context.event_pump()?);
        render_state.mouse_x = mouse_state.x();
        render_state.mouse_y = mouse_state.y();

        let window_flags = self.canvas.window().window_flags();
        render_state.mouse_enabled = window_flags.0 & SDL_WINDOW_MOUSE_FOCUS.0 != 0;

        Ok(())
    }

    fn is_fullscreen(&self) -> bool {
        self.canvas.window().window_flags().0 & SDL_WINDOW_FULLSCREEN.0 != 0
    }

    fn render_scene(
        &mut self,
        scene: &mut dyn Scene<'sdl>,
        render_state: &mut RenderState<'sdl>,
    ) -> Result<()> {
        let scene_changed = scene.draw(self.canvas)?;
        let rerender = self.render_configuration.use_frame_sync
            || (self.rerender || scene_changed || Some(scene.opacity()) != self.last_opacity);

        if rerender {
            self.render(render_state, scene, scene_changed)?;
            self.rerender = false;
        }

        Ok(())
    }

    fn consume_events(
        &mut self,
        render_state: &mut RenderState,
        scene: &mut dyn Scene,
        event_collector: &mut EventCollector,
    ) -> Result<Option<StopReason>> {
        let mut toggle_fullscreen = false;
        for event in event_collector.decoded_events() {
            match *event {
                GameEvent::Quit => return Ok(Some(StopReason::Quit)),
                GameEvent::RenderReset => return Ok(Some(StopReason::RenderReset)),
                GameEvent::Redraw => self.rerender = true,
                GameEvent::ToggleFullscreen => {
                    toggle_fullscreen = !toggle_fullscreen;
                }
                GameEvent::DispatchSceneEvent(event) => scene.dispatch_event(event),
                GameEvent::MouseMove { .. } | GameEvent::MouseEnter | GameEvent::MouseLeave => (),
                GameEvent::MouseDown { x, y, button } => {
                    scene.dispatch_event(SceneEvent::MouseDown(
                        button.into(),
                        self.mouse_coordinates_from(render_state, scene, x, y),
                    ))
                }
                GameEvent::MouseUp { x, y, button } => scene.dispatch_event(SceneEvent::MouseUp(
                    button.into(),
                    self.mouse_coordinates_from(render_state, scene, x, y),
                )),
                GameEvent::EnterFullscreen => {
                    scene.set_is_fullscreen(true);
                    self.suspended = false;
                }
                GameEvent::LeaveFullscreen => {
                    scene.set_is_fullscreen(false);
                    self.suspended = false;
                }
                GameEvent::SetSpeed(speed) => {
                    let now = Instant::now();

                    self.time_offset_musec += self.time_since_reference_musec(now);
                    self.ts_reference = now;
                    self.speed = speed;
                }
            }
        }

        if let Some((x, y)) = event_collector.aggregated_events().mouse_coordinates {
            scene.dispatch_event(SceneEvent::MouseMove(self.mouse_coordinates_from(
                render_state,
                scene,
                x,
                y,
            )));
        }

        if toggle_fullscreen {
            let is_fullscreen =
                self.canvas.window().window_flags().0 & SDL_WindowFlags::FULLSCREEN.0 != 0;

            self.suspended = true;

            self.canvas
                .window()
                .clone()
                .set_fullscreen(!is_fullscreen)?;
        }

        Ok(None)
    }

    fn mouse_coordinates_from(
        &self,
        render_state: &RenderState,
        scene: &dyn Scene,
        x_win: f32,
        y_win: f32,
    ) -> MouseCoordinates {
        let window = self.canvas.window();
        let pixel_scale = window.pixel_density();

        let x_frac = (x_win * pixel_scale - render_state.layout.scene.x as f32)
            / render_state.layout.scene.width as f32
            * scene.width() as f32;

        let y_frac = (y_win * pixel_scale - render_state.layout.scene.y as f32)
            / render_state.layout.scene.height as f32
            * scene.height() as f32;

        let x = (x_frac.floor() as i32).clamp(0, scene.width() as i32 - 1) as u32;
        let y = (y_frac.floor() as i32).clamp(0, scene.height() as i32 - 1) as u32;

        MouseCoordinates {
            x,
            y,
            x_frac,
            y_frac,
        }
    }

    fn update_mouse_state(
        &mut self,
        event_collector: &EventCollector,
        render_state: &mut RenderState,
    ) {
        let aggregated_events = event_collector.aggregated_events();

        if let Some((x, y)) = aggregated_events.mouse_coordinates {
            render_state.mouse_x = x;
            render_state.mouse_y = y;

            let (width, height) = self.canvas.window().size();

            render_state.mouse_enabled =
                x >= 0. && x < width as f32 && y >= 0. && y < height as f32;

            self.rerender = true;
        }

        match aggregated_events.mouse_owner_change {
            Some(MouseOwnerChange::Enter) => {
                render_state.mouse_enabled = true;
                self.rerender = true;
            }
            Some(MouseOwnerChange::Leave) => {
                render_state.mouse_enabled = false;
                self.rerender = true;
            }
            None => (),
        }
    }

    fn update_layout(&mut self, render_state: &mut RenderState<'sdl>) -> Result<()> {
        let (canvas_width, canvas_height) = self.canvas.output_size()?;
        render_state.update_layout(canvas_width, canvas_height);

        Ok(())
    }

    fn render(
        &mut self,
        render_state: &mut RenderState<'sdl>,
        scene: &mut dyn Scene<'sdl>,
        needs_redraw: bool,
    ) -> Result<()> {
        self.update_layout(render_state)?;

        self.canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        self.canvas.clear();

        for i in 0..render_state.layers.len() {
            let layer = render_state
                .layers
                .get_mut(i)
                .ok_or_else(|| anyhow!("invalid layer {}", i))?;

            let dest = render_state
                .layout
                .layers
                .get(i)
                .ok_or_else(|| anyhow!("no layout for layer {}", i))?;

            let texture = self.prescale_layer(scene, layer, needs_redraw)?;
            texture.set_blend_mode(sdl3::render::BlendMode::Blend);
            self.canvas.copy(texture, None, Some(dest.into()))?;
        }

        let scene_opacity = scene.opacity();
        if scene_opacity != 255 {
            render_state.overlay.set_alpha_mod(255 - scene_opacity);
            self.canvas.copy(&render_state.overlay, None, None)?;
        }

        self.last_opacity = Some(scene_opacity);

        if render_state.mouse_enabled && scene.cursor_type() != CursorType::None {
            let layout_cursor = render_state.layout.cursor.clone();
            let pixel_scale = self.canvas.window().pixel_density();

            let mouse_x = render_state.mouse_x * pixel_scale;
            let mouse_y = render_state.mouse_y * pixel_scale;

            let static_texture = render_state.get_cursor(scene.cursor_type()).unwrap();

            let texture =
                self.prescale_static_texture(static_texture, layout_cursor.prescaling_mode)?;
            texture.set_blend_mode(sdl3::render::BlendMode::Blend);

            self.canvas.copy(
                texture,
                None,
                SdlRect::new(
                    mouse_x as i32 - layout_cursor.center_x as i32,
                    mouse_y as i32 - layout_cursor.center_y as i32,
                    layout_cursor.width,
                    layout_cursor.height,
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
        needs_redraw: bool,
    ) -> Result<&'layer mut Texture<'sdl>> {
        let redraw_on_difference_from = if needs_redraw {
            None
        } else {
            Some(layer.current_prescaling_mode)
        };

        let prescaled_texture = self.prescale_texture(
            scene.texture(layer.texture_id)?,
            &mut layer.intermediate_texture,
            layer.prescaling_mode,
            redraw_on_difference_from,
        )?;

        layer.current_prescaling_mode = layer.prescaling_mode;

        Ok(prescaled_texture)
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
        redraw_on_difference_from: Option<PrescalingMode>,
    ) -> Result<&'a mut Texture<'sdl>> {
        let (integer_scaled_width, integer_scaled_height) = match prescaling_mode {
            PrescalingMode::None(scaling_mode) => {
                source_texture.set_scale_mode(scaling_mode);
                return Ok(source_texture);
            }
            PrescalingMode::Quis(width, height) => (width, height),
        };

        let needs_recreate = !matches!(maybe_intermediate_texture.as_ref(), Some(texture)
                if texture.width() == integer_scaled_width
                    && texture.height() == integer_scaled_height);

        if needs_recreate {
            *maybe_intermediate_texture = Some(self.texture_creator.create_texture_target(
                PixelFormat::RGBA8888,
                integer_scaled_width,
                integer_scaled_height,
            )?);
        }

        let intermediate_texture = maybe_intermediate_texture.as_mut().unwrap();

        let already_prescaled = redraw_on_difference_from == Some(prescaling_mode);

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

// In order to avoid undefined behavior caused by double borrows (which for example might
// manifest in associated data consistency issues caused by unspilled changes to the
// dependencies) ExposeWatch extends the borrow of its dependencies over its lifetime.
struct ExposeWatch<'a> {
    _watch: EventWatch<ExposeWatchCb>,
    _parents: PhantomData<&'a mut ()>,
}

impl<'a> ExposeWatch<'a> {
    pub fn new(
        stage: &'a mut Stage,
        render_state: &'a mut RenderState,
        scene: &'a mut dyn Scene,
    ) -> Result<Self> {
        let sdl = stage.sdl_context.clone();

        let expose_watch = ExposeWatchCb::new(stage, render_state, scene);
        // Important: no borrows of any of stage, render_state or scene anymore
        // below this point while ExposeWatch is alive!

        let watch = sdl.event()?.add_event_watch(expose_watch);

        Ok(Self {
            _watch: watch,
            _parents: PhantomData,
        })
    }
}

// We need an explicit drop to make sure that 'a stays alive until ExposeWatch
// is actually dropped and to guarantee that _watch cannot be moved out.
impl Drop for ExposeWatch<'_> {
    fn drop(&mut self) {}
}

struct ExposeWatchCb {
    stage: *mut Stage<'static>,
    render_state: *mut RenderState<'static>,
    scene: *mut dyn Scene<'static>,
}
unsafe impl Send for ExposeWatchCb {}

impl EventWatchCallback for ExposeWatchCb {
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
            let _ = (*self.stage).render(&mut *self.render_state, &mut *self.scene, true);
        }
    }
}

impl ExposeWatchCb {
    pub fn new(stage: &mut Stage, render_state: &mut RenderState, scene: &mut dyn Scene) -> Self {
        // SAFETY: EventWatchCallbacks are declared as Send + 'static, which makes sense for the
        // general case: they are sent to another thread and live until the watch is removed.
        // However, we don't need this as we 1. check whether we are on the main thread before
        // doing anything and 2. drop and destroy the watch before the dependencies are dropped
        // or moved. Note that we still need to avoid a double borrow of the dependencies during
        // the lifetime of the watch in order to avoid undefined behavior (which e.g. might manifest
        // in data consistency issues); this is why ExposeWatch keeps those references alive.
        //
        // The whole construction qualifies as abomination, but MacOS stalls the event pump during
        // window resize, so this is the only way to get redraws and avoid an ugly stretched texture
        // while the window handles are being dragged.
        #[allow(clippy::unnecessary_cast)]
        ExposeWatchCb {
            stage: (stage as *mut _) as *mut Stage<'static>,
            render_state: (render_state as *mut _) as *mut RenderState<'static>,
            scene: unsafe {
                transmute::<*mut dyn Scene<'_>, *mut dyn Scene<'static>>(scene as *mut _)
            },
        }
    }
}
