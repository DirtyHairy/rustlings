use crate::geometry::Rect;
use anyhow::Result;
use sdl3::{
    render::{Canvas, Texture},
    video::Window,
};
use std::cell::RefCell;

pub trait Compositor<'texture, 'creator> {
    fn add_layer(&mut self, texture: &'texture RefCell<Texture<'creator>>, destination: Rect);
}

pub trait Scene<'texture_creator> {
    fn get_dimensions(&self) -> (usize, usize);

    fn get_aspect(&self) -> f32;

    fn register_layers<'scene>(
        &'scene self,
        compositor: &mut dyn Compositor<'scene, 'texture_creator>,
    );

    fn draw(&self, canvas: &mut Canvas<Window>) -> Result<()>;
}
