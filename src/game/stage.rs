use crate::geometry;
use crate::scene::{Compositor, Scene};
use anyhow::Result;
use rustlings::sdl3_aux::{SDL_EVENT_RENDER_DEVICE_LOST, is_main_thread};
use sdl3::sys::video::SDL_WindowFlags;
use sdl3::{
    Sdl,
    event::{Event, EventWatchCallback, WindowEvent},
    keyboard::{Keycode, Mod},
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
};
use std::mem::transmute;

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

    fn render(&mut self, render_state: &mut RenderState, scene: &mut dyn Scene) -> Result<()> {
        let (canvas_width, canvas_height) = self.canvas.output_size()?;
        render_state.update_layout(canvas_width as usize, canvas_height as usize);

        self.canvas.clear();

        for i in 0..render_state.layers.len() {
            let layer = &render_state.layers[i];
            let dest = &render_state.layout.layers[i];

            let texture = scene.texture(layer.texture_id)?;
            texture.set_scale_mode(sdl3::render::ScaleMode::Nearest);

            let _ = self.canvas.copy(texture, None, Some(dest.into()))?;
        }

        self.canvas.present();
        Ok(())
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

struct Layer {
    texture_id: usize,
    destination: geometry::Rect,
}

#[derive(Default)]
struct Layout {
    width: usize,
    height: usize,

    pub scene: geometry::Rect,
    pub layers: Vec<geometry::Rect>,
}

#[derive(Default)]
struct RenderState {
    layers: Vec<Layer>,
    layout: Layout,

    scene_width: usize,
    scene_height: usize,
    scene_aspect: f32,
}

impl RenderState {
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
            dest_scene.width = width as usize;
            dest_scene.y = 0;
            dest_scene.x = ((w - width) / 2.) as usize;
        } else {
            let height = h_scene * w / w_scene;

            dest_scene.width = width;
            dest_scene.height = height as usize;
            dest_scene.x = 0;
            dest_scene.y = ((h - height) / 2.) as usize;
        }

        let dest_layers = self
            .layers
            .iter()
            .map(|layer| geometry::Rect {
                x: dest_scene.x + (layer.destination.x * dest_scene.width) / self.scene_width,
                y: dest_scene.y + (layer.destination.y * dest_scene.height) / self.scene_height,
                width: (layer.destination.width * dest_scene.width) / self.scene_width,
                height: (layer.destination.height * dest_scene.height) / self.scene_height,
            })
            .collect();

        self.layout = Layout {
            width,
            height,
            scene: dest_scene,
            layers: dest_layers,
        };
    }
}

impl Compositor for RenderState {
    fn add_layer(&mut self, texture_id: usize, destination: geometry::Rect) {
        self.layers.push(Layer {
            texture_id,
            destination,
        });
    }
}

struct ExposeWatch {
    stage: *mut Stage<'static>,
    render_state: *mut RenderState,
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
            render_state: render_state as *mut _,
            scene: unsafe {
                transmute::<*mut dyn Scene<'_>, *mut dyn Scene<'static>>(scene as *mut _)
            },
        }
    }
}
