use anyhow::Result;
use sdl3::{
    keyboard::{Keycode, Mod, Scancode},
    render::{Canvas, Texture},
    video::Window,
};

use crate::{
    geometry::Rect,
    state::{GameState, SceneState},
};

#[derive(Clone, Copy)]
#[allow(unused)]
pub struct MouseCoordinates {
    pub x: usize,
    pub y: usize,
    pub x_frac: f32,
    pub y_frac: f32,
}

#[derive(Clone, Copy)]
#[allow(unused)]
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
    MouseMove(MouseCoordinates),
    MouseDown(MouseCoordinates),
    MouseUp(MouseCoordinates),
}

#[derive(Clone, Copy, PartialEq)]
pub enum CursorType {
    None,
    Crosshair,
    Box,
}

pub trait Compositor {
    fn add_layer(&mut self, texture_id: usize, width: usize, height: usize, destination: Rect);
}

pub trait Scene<'texture_creator> {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn aspect(&self) -> f32;
    fn opacity(&self) -> u8;

    fn set_is_fullscreen(&mut self, is_fullscreen: bool);
    fn set_mouse_enabled(&mut self, mouse_enabled: bool);

    fn cursor_type(&self) -> CursorType;

    fn dispatch_event(&mut self, event: SceneEvent);
    fn tick(&mut self, clock_msec: u64);
    fn next_tick_at_msec(&self) -> u64;

    fn is_complete(&self) -> bool;

    fn texture(&mut self, id: usize) -> Result<&mut Texture<'texture_creator>>;
    fn register_layers(&self, compositor: &mut dyn Compositor);

    fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<bool>;
    fn will_redraw(&self) -> bool;

    fn finish(self: Box<Self>) -> (GameState, SceneState);
}
