use crate::{
    geometry::Rect,
    state::{GameState, SceneState},
};
use anyhow::Result;
use sdl3::{render::Canvas, render::Texture, video::Window};

pub trait Compositor {
    fn add_layer(&mut self, texture_id: usize, destination: Rect);
}

pub trait Scene<'texture_creator> {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn aspect(&self) -> f32;

    fn texture(&mut self, id: usize) -> Result<&mut Texture<'texture_creator>>;

    fn register_layers(&self, compositor: &mut dyn Compositor);

    fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<()>;

    fn finish(&self) -> (GameState, SceneState);
}
