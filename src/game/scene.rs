use anyhow::Result;
use sdl3::{
    keyboard::{Keycode, Mod, Scancode},
    render::{Canvas, Texture},
    video::Window,
};

#[derive(Clone, Copy)]
pub enum SceneEvent {
    KeyDown {
        keycode: Keycode,
        keymod: Mod,
        scancode: Scancode,
    },
    KeyUp {
        keycode: Keycode,
        scancode: Scancode,
    },
}

use crate::{
    geometry::Rect,
    state::{GameState, SceneState},
};

pub trait Compositor {
    fn add_layer(&mut self, texture_id: usize, width: usize, height: usize, destination: Rect);
}

pub trait Scene<'texture_creator> {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn aspect(&self) -> f32;

    fn dispatch_event(&mut self, event: SceneEvent);
    fn tick(&mut self, clock_msec: u64);
    fn next_tick_at_msec(&self) -> u64;

    fn texture(&mut self, id: usize) -> Result<&mut Texture<'texture_creator>>;
    fn register_layers(&self, compositor: &mut dyn Compositor);

    fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<bool>;

    fn finish(self: Box<Self>) -> (GameState, SceneState);
}
