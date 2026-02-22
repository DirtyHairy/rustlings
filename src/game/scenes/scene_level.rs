use crate::geometry::Rect;
use anyhow::Result;
use rustlings::{game_data::GameData, sdl_rendering::texture_from_bitmap};
use sdl3::{
    pixels::PixelFormat,
    render::{Canvas, Texture, TextureCreator},
    video::Window,
};
use std::{cell::RefCell, rc::Rc};

use crate::scene::Scene;

pub struct SceneLevel<'sdl> {
    game_data: Rc<GameData>,

    texture_screen: RefCell<Texture<'sdl>>,
    texture_skill_panel: Texture<'sdl>,
}

impl<'sdl> SceneLevel<'sdl> {
    pub fn new<T>(
        game_data: Rc<GameData>,
        texture_creator: &'sdl TextureCreator<T>,
    ) -> Result<Self> {
        let texture_screen =
            RefCell::new(texture_creator.create_texture_target(PixelFormat::RGBA8888, 320, 200)?);

        let texture_skill_panel = texture_from_bitmap(
            &game_data.skill_panel,
            &game_data.resolve_skill_panel_palette(0),
            texture_creator,
        )?;

        Ok(SceneLevel {
            game_data,
            texture_screen,
            texture_skill_panel,
        })
    }
}

impl<'texture_creator> Scene<'texture_creator> for SceneLevel<'texture_creator> {
    fn get_width(&self) -> usize {
        320
    }

    fn get_height(&self) -> usize {
        200
    }

    fn get_aspect(&self) -> f32 {
        1.2
    }

    fn register_layers<'scene>(
        &'scene self,
        compositor: &mut dyn crate::scene::Compositor<'scene, 'texture_creator>,
    ) {
        compositor.add_layer(&self.texture_screen, Rect::new(0, 0, 320, 200));
    }

    fn draw(&self, canvas: &mut Canvas<Window>) -> Result<()> {
        canvas
            .with_texture_canvas(&mut *self.texture_screen.borrow_mut(), |canvas| {
                let _ = canvas.copy(
                    &self.texture_skill_panel,
                    None,
                    sdl3::rect::Rect::new(0, 160, 320, 40),
                );
            })
            .map_err(anyhow::Error::from)
    }
}
