use crate::geometry;
use anyhow::Result;
use sdl3::{
    Sdl,
    event::Event,
    rect::Rect as SdlRect,
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

struct Layer<'texture, 'creator> {
    texture: &'texture RefCell<Texture<'creator>>,
    destination: geometry::Rect,
}

#[derive(Default)]
struct Stack<'texture, 'creator> {
    layers: Vec<Layer<'texture, 'creator>>,
}

impl<'texture, 'creator> Compositor<'texture, 'creator> for Stack<'texture, 'creator> {
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
        let mut stack: Stack<'scene, 'sdl> = Default::default();

        scene.register_layers(&mut stack);
        scene.draw(&mut self.canvas)?;

        self.canvas.clear();
        let aspect = scene.get_aspect();

        for layer in &stack.layers {
            layer
                .texture
                .borrow_mut()
                .set_scale_mode(sdl3::render::ScaleMode::Nearest);

            let (canvas_width, canvas_height) = self.canvas.output_size()?;
            let scale_x = canvas_width / layer.texture.borrow().width();
            let scale_y = canvas_height / layer.texture.borrow().height();

            let _ = self.canvas.copy(
                &*layer.texture.borrow(),
                None,
                SdlRect::new(
                    scale_x as i32 * layer.destination.x as i32,
                    (scale_y as f32 * layer.destination.y as f32 * aspect) as i32,
                    scale_x * layer.destination.width as u32,
                    (scale_y as f32 * layer.destination.height as f32 as f32 * aspect) as u32,
                ),
            )?;
        }

        self.canvas.present();

        let mut event_pump = self.sdl_context.event_pump()?;
        loop {
            while let Some(event) = event_pump.wait_event_timeout(50) {
                if event_is_quit(&event) {
                    return Ok(());
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
