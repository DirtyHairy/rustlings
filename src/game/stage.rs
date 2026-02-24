use std::mem::transmute;

use anyhow::Result;
use rustlings::sdl_rendering::with_texture_canvas;
use rustlings::sdl3_aux::{SDL_EVENT_RENDER_DEVICE_LOST, is_main_thread};
use sdl3::pixels::PixelFormat;
use sdl3::render::{ScaleMode, Texture};
use sdl3::sys::video::SDL_WindowFlags;
use sdl3::{
    Sdl,
    event::{Event, EventWatchCallback, WindowEvent},
    keyboard::{Keycode, Mod},
    rect::Rect as SdlRect,
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
};

use crate::geometry;
use crate::scene::{Compositor, Scene};

pub enum RunResult {
    Quit,
    RenderReset,
}

pub struct Stage<'sdl> {
    sdl_context: &'sdl Sdl,
    canvas: &'sdl mut Canvas<Window>,
    texture_creator: &'sdl TextureCreator<WindowContext>,
}

impl<'sdl> Stage<'sdl> {
    pub fn new(
        sdl_context: &'sdl Sdl,
        canvas: &'sdl mut Canvas<Window>,
        texture_creator: &'sdl TextureCreator<WindowContext>,
    ) -> Result<Self> {
        Ok(Stage {
            sdl_context,
            canvas,
            texture_creator,
        })
    }

    pub fn run(&mut self, scene: &mut dyn Scene<'sdl>) -> Result<RunResult> {
        let mut render_state = RenderState::new(scene);

        scene.register_layers(&mut render_state);
        scene.draw(&mut self.canvas)?;

        let mut redraw = true;
        loop {
            if redraw {
                self.render(&mut render_state, scene)?;
                redraw = false;
            }

            let expose_watch = ExposeWatch::new(self, &mut render_state, scene);
            let _event_watch = self.sdl_context.event()?.add_event_watch(expose_watch);

            let handle_events_result = self.handle_events()?;

            match handle_events_result {
                HandleEventsResult::Quit => return Ok(RunResult::Quit),
                HandleEventsResult::RenderReset => return Ok(RunResult::RenderReset),
                HandleEventsResult::Redraw => redraw = true,
                HandleEventsResult::ToggleFullscreen => {
                    let is_fullscreen =
                        self.canvas.window().window_flags().0 & SDL_WindowFlags::FULLSCREEN.0 != 0;

                    self.canvas
                        .window()
                        .clone()
                        .set_fullscreen(!is_fullscreen)?;
                }
            }
        }
    }

    fn render(
        &mut self,
        render_state: &mut RenderState<'sdl>,
        scene: &mut dyn Scene<'sdl>,
    ) -> Result<()> {
        let (canvas_width, canvas_height) = self.canvas.output_size()?;
        render_state.update_layout(canvas_width as usize, canvas_height as usize);

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
            let _ = self.canvas.copy(texture, None, Some(dest.into()))?;
        }

        self.canvas.present();
        Ok(())
    }

    fn prescale_layer<'scene: 'layer, 'layer>(
        &mut self,
        scene: &'scene mut dyn Scene<'sdl>,
        layer: &'layer mut Layer<'sdl>,
    ) -> Result<&'layer mut Texture<'sdl>> {
        let source_texture = scene.texture(layer.texture_id)?;

        let (integer_scaled_width, integer_scaled_height) = match layer.prescaling_mode {
            PrescalingMode::None(scaling_mode) => {
                source_texture.set_scale_mode(scaling_mode);
                return Ok(source_texture);
            }
            PrescalingMode::Quis(width, height) => (width as u32, height as u32),
        };

        let intermediate_texture = match layer.intermediate_texture.as_mut() {
            Some(texture)
                if texture.width() == integer_scaled_width
                    && texture.height() == integer_scaled_height =>
            {
                texture
            }
            _ => {
                layer.intermediate_texture = Some(self.texture_creator.create_texture_target(
                    PixelFormat::RGBA8888,
                    integer_scaled_width,
                    integer_scaled_height,
                )?);
                layer.intermediate_texture.as_mut().unwrap()
            }
        };

        source_texture.set_scale_mode(ScaleMode::Nearest);
        intermediate_texture.set_scale_mode(ScaleMode::Linear);

        with_texture_canvas(self.canvas, intermediate_texture, |canvas| -> Result<()> {
            canvas
                .copy(
                    source_texture,
                    None,
                    SdlRect::new(0, 0, integer_scaled_width, integer_scaled_height),
                )
                .map_err(anyhow::Error::from)
        })?;

        Ok(layer.intermediate_texture.as_mut().unwrap())
    }

    fn handle_events(&mut self) -> Result<HandleEventsResult> {
        loop {
            if let Some(handle_event_result) = self
                .sdl_context
                .event_pump()?
                .wait_event_timeout(50)
                .and_then(handle_event)
            {
                return Ok(handle_event_result);
            }
        }
    }
}

fn handle_event(event: Event) -> Option<HandleEventsResult> {
    match event {
        Event::Quit { .. } => Some(HandleEventsResult::Quit),
        Event::Window { win_event, .. } => match win_event {
            WindowEvent::PixelSizeChanged(_, _) => Some(HandleEventsResult::Redraw),
            _ => None,
        },
        Event::RenderDeviceReset { .. } => Some(HandleEventsResult::RenderReset),
        Event::RenderTargetsReset { .. } => Some(HandleEventsResult::RenderReset),
        Event::KeyDown {
            keycode: Some(code),
            keymod,
            repeat: false,
            ..
        } => match (code, keymod) {
            (Keycode::R, Mod::LCTRLMOD | Mod::RCTRLMOD) => Some(HandleEventsResult::RenderReset),
            (Keycode::Q, Mod::LALTMOD | Mod::RALTMOD | Mod::LGUIMOD | Mod::RGUIMOD) => {
                Some(HandleEventsResult::Quit)
            }
            (Keycode::Return, Mod::LALTMOD | Mod::RALTMOD | Mod::LGUIMOD | Mod::RGUIMOD) => {
                Some(HandleEventsResult::ToggleFullscreen)
            }
            _ => None,
        },
        Event::Unknown {
            type_: SDL_EVENT_RENDER_DEVICE_LOST,
            ..
        } => Some(HandleEventsResult::RenderReset),
        _ => None,
    }
}

enum HandleEventsResult {
    Quit,
    Redraw,
    RenderReset,
    ToggleFullscreen,
}

#[derive(Debug, PartialEq)]
enum PrescalingMode {
    None(ScaleMode),
    Quis(usize, usize),
}

impl Default for PrescalingMode {
    fn default() -> Self {
        PrescalingMode::None(ScaleMode::Nearest)
    }
}

struct Layer<'texture_creator> {
    texture_id: usize,

    texture_width: usize,
    texture_height: usize,
    destination: geometry::Rect,

    prescaling_mode: PrescalingMode,
    intermediate_texture: Option<Texture<'texture_creator>>,
}

#[derive(Default)]
struct Layout {
    width: usize,
    height: usize,

    pub scene: geometry::Rect,
    pub layers: Vec<geometry::Rect>,
}

#[derive(Default)]
struct RenderState<'texture_creator> {
    layers: Vec<Layer<'texture_creator>>,
    layout: Layout,

    scene_width: usize,
    scene_height: usize,
    scene_aspect: f32,
}

impl RenderState<'_> {
    pub fn new(scene: &dyn Scene) -> Self {
        RenderState {
            scene_width: scene.width(),
            scene_height: scene.height(),
            scene_aspect: scene.aspect(),
            ..Default::default()
        }
    }

    pub fn update_layout(&mut self, width: usize, height: usize) {
        if self.layout.width == width && self.layout.height == height {
            return;
        }

        let w = width as f32;
        let h = height as f32;
        let w_scene = self.scene_width as f32;
        let h_scene = self.scene_height as f32 * self.scene_aspect;

        let mut dest_scene: geometry::Rect = Default::default();

        if w_scene * h / h_scene <= w {
            let width = w_scene * h / h_scene;

            dest_scene.height = height;
            dest_scene.width = width.round() as usize;
            dest_scene.y = 0;
            dest_scene.x = ((w - width) / 2.).round() as usize;
        } else {
            let height = h_scene * w / w_scene;

            dest_scene.width = width;
            dest_scene.height = height.round() as usize;
            dest_scene.x = 0;
            dest_scene.y = ((h - height) / 2.).round() as usize;
        }

        let mut dest_layers: Vec<geometry::Rect> = Vec::with_capacity(self.layers.len());
        let scale_x = dest_scene.width as f32 / self.scene_width as f32;
        let scale_y = dest_scene.height as f32 / self.scene_height as f32;

        for layer in &mut self.layers {
            let dest = geometry::Rect {
                x: dest_scene.x + (layer.destination.x as f32 * scale_x).round() as usize,
                y: dest_scene.y + (layer.destination.y as f32 * scale_y).round() as usize,
                width: (layer.destination.width as f32 * scale_x).round() as usize,
                height: (layer.destination.height as f32 * scale_y).round() as usize,
            };

            dest_layers.push(dest);
            layer.prescaling_mode =
                calculate_prescaling_mode(layer.texture_width, layer.texture_height, &dest);
        }

        self.layout = Layout {
            width,
            height,
            scene: dest_scene,
            layers: dest_layers,
        };
    }
}

fn calculate_prescaling_mode(width: usize, height: usize, dest: &geometry::Rect) -> PrescalingMode {
    if width == 0 || height == 0 {
        return Default::default();
    }
    let mut integer_scale_x = (dest.width as f32 / width as f32).round() as usize;
    let mut integer_scale_y = (dest.height as f32 / height as f32).round() as usize;

    if (integer_scale_x == 0 && integer_scale_y <= 1)
        || (integer_scale_y == 0 && integer_scale_x <= 1)
    {
        // At least one axis is downscaled, and the other is not nontrivially
        // integer scaled -> use the original texture and use linear scaling
        return PrescalingMode::None(ScaleMode::Linear);
    }

    // We are integer scaling along at least one axis, so make sure we keep
    // the other finite.
    integer_scale_x = std::cmp::max(1, integer_scale_x);
    integer_scale_y = std::cmp::max(1, integer_scale_y);

    let integer_scaled_width = width * integer_scale_x;
    let integer_scaled_height = height * integer_scale_y;

    if integer_scaled_width == dest.width && integer_scaled_height == dest.height {
        // Integer scaling step is sufficient -> use the original texture and use
        // nearest-neighbour scaling.
        return PrescalingMode::None(ScaleMode::Nearest);
    }

    if integer_scale_x == 1 && integer_scale_y == 1 {
        // The integer scaling step is trivial -> use the original texture
        // and use linear scaling
        return PrescalingMode::None(ScaleMode::Linear);
    }

    PrescalingMode::Quis(integer_scaled_width, integer_scaled_height)
}

impl Compositor for RenderState<'_> {
    fn add_layer(
        &mut self,
        texture_id: usize,
        width: usize,
        height: usize,
        destination: geometry::Rect,
    ) {
        self.layers.push(Layer {
            texture_id,
            texture_width: width,
            texture_height: height,
            destination,
            intermediate_texture: None,
            prescaling_mode: Default::default(),
        });
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
        ExposeWatch {
            stage: (stage as *mut _) as *mut Stage<'static>,
            render_state: (render_state as *mut _) as *mut RenderState<'static>,
            scene: unsafe {
                transmute::<*mut dyn Scene<'_>, *mut dyn Scene<'static>>(scene as *mut _)
            },
        }
    }
}

#[cfg(test)]
mod test {
    use sdl3::render::ScaleMode;

    use crate::{
        geometry,
        stage::{PrescalingMode, calculate_prescaling_mode},
    };

    #[test]
    fn calculate_prescaling_mode_degenerate_width() {
        let prescaling_mode =
            calculate_prescaling_mode(0, 100, &geometry::Rect::new(0, 0, 100, 100));

        assert_eq!(prescaling_mode, Default::default());
    }

    #[test]
    fn calculate_prescaling_mode_degenerate_height() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 0, &geometry::Rect::new(0, 0, 100, 100));

        assert_eq!(prescaling_mode, Default::default());
    }

    #[test]
    fn calculate_prescaling_mode_downscale_both_1() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 100, &geometry::Rect::new(0, 0, 20, 20));

        assert_eq!(prescaling_mode, PrescalingMode::None(ScaleMode::Linear));
    }

    #[test]
    fn calculate_prescaling_mode_downscale_both_2() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 100, &geometry::Rect::new(0, 0, 90, 90));

        assert_eq!(prescaling_mode, PrescalingMode::None(ScaleMode::Linear));
    }

    #[test]
    fn calculate_prescaling_mode_downscale_one_1() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 100, &geometry::Rect::new(0, 0, 90, 190));

        assert_eq!(prescaling_mode, PrescalingMode::Quis(100, 200));
    }

    #[test]
    fn calculate_prescaling_mode_downscale_one_2() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 100, &geometry::Rect::new(0, 0, 90, 110));

        assert_eq!(prescaling_mode, PrescalingMode::None(ScaleMode::Linear));
    }

    #[test]
    fn calculate_prescaling_mode_upscale_exact() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 100, &geometry::Rect::new(0, 0, 200, 300));

        assert_eq!(prescaling_mode, PrescalingMode::None(ScaleMode::Nearest));
    }

    #[test]
    fn calculate_prescaling_mode_upscale_slighty() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 100, &geometry::Rect::new(0, 0, 110, 120));

        assert_eq!(prescaling_mode, PrescalingMode::None(ScaleMode::Linear))
    }

    #[test]
    fn calculate_prescaling_mode_upscale_almost() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 100, &geometry::Rect::new(0, 0, 110, 190));

        assert_eq!(prescaling_mode, PrescalingMode::Quis(100, 200));
    }

    #[test]
    fn calculate_prescaling_mode_upscale() {
        let prescaling_mode =
            calculate_prescaling_mode(100, 100, &geometry::Rect::new(0, 0, 610, 890));

        assert_eq!(prescaling_mode, PrescalingMode::Quis(600, 900));
    }
}
