use std::rc::Rc;

use anyhow::{Result, format_err};
use rustlings::{
    game_data::{
        GameData, LEVEL_HEIGHT, LEVEL_WIDTH, Level, MINIMAP_AREA_Y, MINIMAP_FRAME_HEIGHT,
        MINIMAP_FRAME_WIDTH, MINIMAP_VIEW_HEIGHT, MINIMAP_VIEW_WIDTH, MINIMAP_VIEW_X,
        MINIMAP_VIEW_Y, PALETTE_SIZE, PaletteEntry, SCREEN_HEIGHT, SCREEN_WIDTH,
        SKILL_PANEL_HEIGHT, file::level,
    },
    sdl_rendering::{SDLSprite, texture_from_bitmap, with_texture_canvas},
    sdl3_aux::apply_blend_mode,
};
use sdl3::{
    pixels::{Color, PixelFormat},
    rect::Rect as SdlRect,
    render::{BlendMode::Blend, Canvas, ScaleMode, Texture, TextureCreator},
    sys::blendmode::{
        SDL_BLENDFACTOR_DST_ALPHA, SDL_BLENDFACTOR_ONE, SDL_BLENDFACTOR_ONE_MINUS_DST_ALPHA,
        SDL_BLENDFACTOR_ONE_MINUS_SRC_ALPHA, SDL_BLENDFACTOR_ZERO, SDL_BLENDOPERATION_ADD,
        SDL_BlendMode, SDL_ComposeCustomBlendMode,
    },
    video::Window,
};

use crate::{geometry::Rect, state::SceneStateLevel};

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    pub struct Redraw: u32 {
        const LEVEL = 0x01;
        const SCREEN = 0x02;
        const ALL = !0;
    }
}

const SKILL_PANEL_Y: usize = SCREEN_HEIGHT - SKILL_PANEL_HEIGHT;

const TEXTURE_ID_MAIN_SCREEN: usize = 0;
const TEXTURE_ID_MINIMAP: usize = 1;

struct Object<'texture_creator> {
    id: usize,

    x: usize,
    y: usize,
    flip: bool,

    sprite: SDLSprite<'texture_creator>,
}

pub struct Renderer<'texture_creator> {
    redraw: Redraw,

    texture_terrain: Texture<'texture_creator>,
    texture_skill_panel: Texture<'texture_creator>,
    texture_minimap_frame: Texture<'texture_creator>,
    texture_level: Texture<'texture_creator>,
    texture_screen: Texture<'texture_creator>,

    objects_background: Vec<Object<'texture_creator>>,
    objects_foreground: Vec<Object<'texture_creator>>,
    objects_merge: Vec<Object<'texture_creator>>,

    blend_mode_merge: SDL_BlendMode,
    blend_mode_background: SDL_BlendMode,
}

impl<'texture_creator> Renderer<'texture_creator> {
    pub fn new<T>(
        level: &Level,
        scene_state: &SceneStateLevel,
        game_data: Rc<GameData>,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        let palette = game_data.resolve_palette(&level)?;
        let palette_skill_panel = game_data.resolve_skill_panel_palette(0);

        let texture_terrain = texture_from_bitmap(&scene_state.terrain, &palette, texture_creator)?;

        let texture_skill_panel = texture_from_bitmap(
            &game_data.skill_panel.panel,
            &palette_skill_panel,
            texture_creator,
        )?;

        let texture_minimap_frame = texture_from_bitmap(
            &game_data.skill_panel.minimap_frame,
            &palette_skill_panel,
            texture_creator,
        )?;

        let texture_level = texture_creator.create_texture_target(
            PixelFormat::RGBA8888,
            LEVEL_WIDTH as u32,
            LEVEL_HEIGHT as u32,
        )?;

        let texture_screen = texture_creator.create_texture_target(
            PixelFormat::RGBA8888,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        )?;

        let objects_merge = create_objects(&game_data, &palette, level, texture_creator, |o| {
            o.draw_only_over_terrain
        })?;

        let objects_foreground =
            create_objects(&game_data, &palette, level, texture_creator, |o| {
                !o.draw_only_over_terrain && !o.do_not_overwrite
            })?;

        let objects_background =
            create_objects(&game_data, &palette, level, texture_creator, |o| {
                !o.draw_only_over_terrain && o.do_not_overwrite
            })?;

        let blend_mode_merge = SDL_ComposeCustomBlendMode(
            SDL_BLENDFACTOR_DST_ALPHA,
            SDL_BLENDFACTOR_ONE_MINUS_SRC_ALPHA,
            SDL_BLENDOPERATION_ADD,
            SDL_BLENDFACTOR_ZERO,
            SDL_BLENDFACTOR_ONE,
            SDL_BLENDOPERATION_ADD,
        );

        let blend_mode_background = SDL_ComposeCustomBlendMode(
            SDL_BLENDFACTOR_ONE_MINUS_DST_ALPHA,
            SDL_BLENDFACTOR_ONE,
            SDL_BLENDOPERATION_ADD,
            SDL_BLENDFACTOR_ONE_MINUS_DST_ALPHA,
            SDL_BLENDFACTOR_ONE,
            SDL_BLENDOPERATION_ADD,
        );

        Ok(Renderer {
            redraw: Redraw::ALL,

            texture_terrain,
            texture_skill_panel,
            texture_minimap_frame,
            texture_level,
            texture_screen,

            objects_merge,
            objects_foreground,
            objects_background,

            blend_mode_merge,
            blend_mode_background,
        })
    }

    pub fn mark_for_redraw(&mut self, redraw: Redraw) {
        self.redraw.insert(redraw);
    }

    pub fn register_layers(&self, compositor: &mut dyn crate::scene::Compositor) {
        compositor.add_layer(
            TEXTURE_ID_MINIMAP,
            LEVEL_WIDTH,
            LEVEL_HEIGHT,
            Rect::new(
                MINIMAP_VIEW_X,
                SKILL_PANEL_Y + MINIMAP_VIEW_Y,
                MINIMAP_VIEW_WIDTH,
                MINIMAP_VIEW_HEIGHT,
            ),
        );

        compositor.add_layer(
            TEXTURE_ID_MAIN_SCREEN,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            Rect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT),
        );
    }

    fn minimap_frame_position(&self, state: &SceneStateLevel) -> (usize, usize) {
        (
            MINIMAP_VIEW_X + (state.level_x * MINIMAP_VIEW_WIDTH) / LEVEL_WIDTH - 1,
            SKILL_PANEL_Y + MINIMAP_AREA_Y,
        )
    }

    pub fn texture(&mut self, id: usize) -> Result<&mut Texture<'texture_creator>> {
        match id {
            TEXTURE_ID_MAIN_SCREEN => Ok(&mut self.texture_screen),
            TEXTURE_ID_MINIMAP => Ok(&mut self.texture_level),
            _ => Err(anyhow::format_err!("invalid texture id {}", id)),
        }
    }

    pub fn draw(&mut self, state: &SceneStateLevel, canvas: &mut Canvas<Window>) -> Result<bool> {
        let redraw = self.redraw;
        self.redraw = Redraw::empty();

        if redraw.is_empty() {
            return Ok(false);
        }

        if redraw.contains(Redraw::LEVEL) {
            with_texture_canvas(canvas, &mut self.texture_level, |canvas| -> Result<()> {
                canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
                canvas.clear();

                self.texture_terrain
                    .set_blend_mode(sdl3::render::BlendMode::Blend);
                self.texture_terrain.set_scale_mode(ScaleMode::Nearest);
                canvas
                    .copy(&self.texture_terrain, None, None)
                    .map_err(anyhow::Error::from)?;

                for object in &mut self.objects_merge {
                    apply_blend_mode(&mut object.sprite.texture(), self.blend_mode_merge);
                    object.sprite.texture().set_scale_mode(ScaleMode::Nearest);

                    object.sprite.blit(
                        canvas,
                        object.x as i32,
                        object.y as i32,
                        state.object_state[object.id].frame,
                        1,
                        object.flip,
                    )?;
                }

                for object in &mut self.objects_background {
                    apply_blend_mode(&mut object.sprite.texture(), self.blend_mode_background);
                    object.sprite.texture().set_scale_mode(ScaleMode::Nearest);

                    object.sprite.blit(
                        canvas,
                        object.x as i32,
                        object.y as i32,
                        state.object_state[object.id].frame,
                        1,
                        object.flip,
                    )?;
                }

                for object in &mut self.objects_foreground {
                    object.sprite.texture().set_blend_mode(Blend);
                    object.sprite.texture().set_scale_mode(ScaleMode::Nearest);

                    object.sprite.blit(
                        canvas,
                        object.x as i32,
                        object.y as i32,
                        state.object_state[object.id].frame,
                        1,
                        object.flip,
                    )?;
                }

                Ok(())
            })?;
        }

        let (frame_x, frame_y) = self.minimap_frame_position(state);

        with_texture_canvas(canvas, &mut self.texture_screen, |canvas| -> Result<()> {
            canvas.set_draw_color(Color::RGBA(0, 0, 0, 0xff));
            canvas.clear();

            self.texture_skill_panel
                .set_blend_mode(sdl3::render::BlendMode::None);
            self.texture_skill_panel.set_scale_mode(ScaleMode::Nearest);
            canvas.copy(
                &self.texture_skill_panel,
                None,
                SdlRect::new(
                    0,
                    LEVEL_HEIGHT as i32,
                    SCREEN_WIDTH as u32,
                    SKILL_PANEL_HEIGHT as u32,
                ),
            )?;

            self.texture_level
                .set_blend_mode(sdl3::render::BlendMode::None);
            self.texture_level.set_scale_mode(ScaleMode::Nearest);
            canvas.copy(
                &self.texture_level,
                SdlRect::new(
                    state.level_x as i32,
                    0,
                    SCREEN_WIDTH as u32,
                    LEVEL_HEIGHT as u32,
                ),
                SdlRect::new(0, 0, SCREEN_WIDTH as u32, LEVEL_HEIGHT as u32),
            )?;

            self.texture_minimap_frame.set_blend_mode(Blend);
            self.texture_minimap_frame
                .set_scale_mode(ScaleMode::Nearest);
            canvas.copy(
                &self.texture_minimap_frame,
                None,
                SdlRect::new(
                    frame_x as i32,
                    frame_y as i32,
                    MINIMAP_FRAME_WIDTH as u32,
                    MINIMAP_FRAME_HEIGHT as u32,
                ),
            )?;

            Ok(())
        })?;

        Ok(true)
    }
}

fn create_objects<'texture_creator, P: Fn(&&level::Object) -> bool, T>(
    game_data: &GameData,
    palette: &[PaletteEntry; PALETTE_SIZE],
    level: &Level,
    texture_creator: &'texture_creator TextureCreator<T>,
    predicate: P,
) -> Result<Vec<Object<'texture_creator>>> {
    level
        .objects
        .iter()
        .enumerate()
        .filter(|(_, x)| predicate(x))
        .map(|(id, o)| -> Result<Object> {
            let tileset = game_data
                .tilesets
                .get(level.graphics_set as usize)
                .ok_or(format_err!("invalid tileset {}", o.id))?;

            let sprite = tileset
                .object_sprites
                .get(o.id as usize)
                .ok_or(format_err!("invlid object {}", o.id))?
                .as_ref()
                .ok_or(format_err!("object {} not defined", o.id))?;

            let sdl_sprite = SDLSprite::from_sprite(&sprite, palette, texture_creator)?;

            Ok(Object {
                id,
                x: o.x as usize,
                y: o.y as usize,
                flip: o.flip_y,
                sprite: sdl_sprite,
            })
        })
        .collect()
}
