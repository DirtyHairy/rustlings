use crate::geometry;
use anyhow::Result;
use rustlings::sdl3_aux::is_main_thread;
use sdl3::{
    Sdl,
    event::{Event, EventWatchCallback, WindowEvent},
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
};
use std::cell::RefCell;

use crate::scene::{Compositor, Scene};

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

    pub fn run<'scene>(&mut self, scene: &'scene dyn Scene<'sdl>) -> Result<()> {
        let mut render_state: RenderState<'scene, 'sdl> = RenderState::new(scene);

        scene.register_layers(&mut render_state);
        scene.draw(&mut self.canvas)?;

        loop {
            self.render(&mut render_state)?;

            let expose_watch = ExposeWatch::new(self, &mut render_state);
            let _event_watch = self.sdl_context.event()?.add_event_watch(expose_watch);

            let handle_events_result = self.handle_events()?;

            if let HandleEventsResult::Quit = handle_events_result {
                break;
            }
        }

        Ok(())
    }

    fn render(&mut self, render_state: &mut RenderState) -> Result<()> {
        let (canvas_width, canvas_height) = self.canvas.output_size()?;
        render_state.update_layout(canvas_width as usize, canvas_height as usize);

        self.canvas.clear();

        for i in 0..render_state.layers.len() {
            let layer = &render_state.layers[i];
            let dest = &render_state.layout.layers[i];

            layer
                .texture
                .borrow_mut()
                .set_scale_mode(sdl3::render::ScaleMode::Nearest);

            let _ = self
                .canvas
                .copy(&*layer.texture.borrow(), None, Some(dest.into()))?;
        }

        self.canvas.present();
        Ok(())
    }

    fn handle_events(&mut self) -> Result<HandleEventsResult> {
        loop {
            while let Some(event) = self.sdl_context.event_pump()?.wait_event_timeout(50) {
                if event_is_quit(&event) {
                    return Ok(HandleEventsResult::Quit);
                }

                if event_is_redraw(&event) {
                    return Ok(HandleEventsResult::Redraw);
                }
            }
        }
    }
}

fn event_is_quit(event: &Event) -> bool {
    match event {
        Event::Quit { .. } => true,
        Event::KeyDown {
            keycode: Some(code),
            ..
        } => *code == sdl3::keyboard::Keycode::Escape,
        _ => false,
    }
}

fn event_is_redraw(event: &Event) -> bool {
    match event {
        Event::Window { win_event, .. } => match win_event {
            WindowEvent::PixelSizeChanged(_, _) => true,
            _ => false,
        },
        _ => false,
    }
}

enum HandleEventsResult {
    Quit,
    Redraw,
}

struct Layer<'texture, 'creator> {
    texture: &'texture RefCell<Texture<'creator>>,
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
struct RenderState<'texture, 'creator> {
    layers: Vec<Layer<'texture, 'creator>>,
    layout: Layout,

    scene_width: usize,
    scene_height: usize,
    scene_aspect: f32,
}

impl<'texture, 'creator> RenderState<'texture, 'creator> {
    pub fn new(scene: &dyn Scene) -> Self {
        RenderState {
            scene_width: scene.get_width(),
            scene_height: scene.get_height(),
            scene_aspect: scene.get_aspect(),
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

impl<'texture, 'creator> Compositor<'texture, 'creator> for RenderState<'texture, 'creator> {
    fn add_layer(
        &mut self,
        texture: &'texture RefCell<Texture<'creator>>,
        destination: geometry::Rect,
    ) {
        self.layers.push(Layer {
            texture,
            destination,
        });
    }
}

struct ExposeWatch {
    stage: *mut Stage<'static>,
    render_state: *mut RenderState<'static, 'static>,
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
            let _ = (*self.stage).render(&mut *self.render_state);
        }
    }
}

impl ExposeWatch {
    pub fn new(stage: &mut Stage, render_state: &mut RenderState) -> Self {
        ExposeWatch {
            stage: (stage as *mut Stage<'_>) as *mut Stage<'static>,
            render_state: (render_state as *mut RenderState<'_, '_>)
                as *mut RenderState<'static, 'static>,
        }
    }
}
