use std::rc::Rc;

use anyhow::{Result, bail, format_err};
use rustlings::{
    game_data::{
        GameData, LEVEL_HEIGHT, LEVEL_WIDTH, Level, MINIMAP_AREA_Y, MINIMAP_FRAME_HEIGHT,
        MINIMAP_FRAME_WIDTH, MINIMAP_VIEW_HEIGHT, MINIMAP_VIEW_WIDTH, MINIMAP_VIEW_X,
        MINIMAP_VIEW_Y, PALETTE_SIZE, PaletteEntry, SCREEN_HEIGHT, SCREEN_WIDTH,
        SKILL_PANEL_HEIGHT, file::level,
    },
    sdl_rendering::{
        SDLSprite, texture_from_bitmap, texture_from_bitmap_mapped, with_texture_canvas,
    },
    sdl3_aux::apply_blend_mode,
};
use sdl3::{
    pixels::{Color, PixelFormat},
    rect::Rect as SdlRect,
    render::{BlendMode::Blend, Canvas, ScaleMode, Texture, TextureAccess, TextureCreator},
    sys::blendmode::{
        SDL_BLENDFACTOR_DST_ALPHA, SDL_BLENDFACTOR_ONE, SDL_BLENDFACTOR_ONE_MINUS_DST_ALPHA,
        SDL_BLENDFACTOR_ONE_MINUS_SRC_ALPHA, SDL_BLENDFACTOR_ZERO, SDL_BLENDMODE_BLEND,
        SDL_BLENDMODE_MOD, SDL_BLENDOPERATION_ADD, SDL_BlendMode, SDL_ComposeCustomBlendMode,
    },
    video::Window,
};
use sdl3::{
    render::{BlendMode, RenderTarget},
    sys::blendmode::SDL_BLENDMODE_NONE,
};

use crate::{
    geometry::Rect, scenes::scene_level::skill_panel_renderer::SkillPanelRenderer,
    state::SceneStateLevel,
};

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    pub struct Redraw: u32 {
        const LEVEL = 0x01;
        const SCREEN = 0x02;
        const SKILL_PANEL = 0x04;
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

struct StencilTextures<'texture_creator> {
    stencil_terrain: Texture<'texture_creator>,
    intermediate_terrain: Texture<'texture_creator>,
}

enum RenderStrategy<'texture_creator> {
    Blend,
    Stencil(StencilTextures<'texture_creator>),
}

pub struct Renderer<'texture_creator> {
    redraw: Redraw,

    texture_terrain: Texture<'texture_creator>,
    texture_minimap_frame: Texture<'texture_creator>,
    texture_level: Texture<'texture_creator>,
    texture_screen: Texture<'texture_creator>,

    objects_background: Vec<Object<'texture_creator>>,
    objects_foreground: Vec<Object<'texture_creator>>,
    objects_merge: Vec<Object<'texture_creator>>,

    blend_mode_merge: SDL_BlendMode,
    blend_mode_background: SDL_BlendMode,

    skill_panel_renderer: SkillPanelRenderer<'texture_creator>,

    render_strategy: RenderStrategy<'texture_creator>,
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

        let skill_panel_renderer =
            SkillPanelRenderer::new(level, Rc::clone(&game_data), texture_creator)?;

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

        let mut texture_probe =
            texture_creator.create_texture(PixelFormat::RGBA8888, TextureAccess::Static, 1, 1)?;

        let render_mode = if apply_blend_mode(&mut texture_probe, blend_mode_background)
            && apply_blend_mode(&mut texture_probe, blend_mode_merge)
        {
            RenderStrategy::Blend
        } else {
            println!("custom blend modes not supported; falling back to stencil based rendering");

            let stencil_terrain = texture_from_bitmap_mapped(
                &scene_state.terrain,
                &palette,
                texture_creator,
                |Color { a, .. }| {
                    if a == 0 {
                        Color::RGBA(0, 0, 0, 0)
                    } else {
                        Color::RGBA(255, 255, 255, 255)
                    }
                },
            )?;

            let intermediate_terrain = texture_creator.create_texture_target(
                PixelFormat::RGBA8888,
                LEVEL_WIDTH as u32,
                LEVEL_HEIGHT as u32,
            )?;

            RenderStrategy::Stencil(StencilTextures {
                stencil_terrain,
                intermediate_terrain,
            })
        };

        Ok(Renderer {
            redraw: Redraw::ALL,

            texture_terrain,
            texture_minimap_frame,
            texture_level,
            texture_screen,

            objects_merge,
            objects_foreground,
            objects_background,

            blend_mode_merge,
            blend_mode_background,

            skill_panel_renderer,

            render_strategy: render_mode,
        })
    }

    pub fn mark_for_redraw(&mut self, redraw: Redraw) {
        self.redraw.insert(redraw);
    }

    pub fn will_redraw(&self) -> bool {
        !self.redraw.is_empty()
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
        if self.redraw.is_empty() {
            return Ok(false);
        }

        let mut redraw = self.redraw;
        self.redraw = Redraw::empty();

        if redraw.contains(Redraw::LEVEL) {
            self.draw_level(state, canvas)?;

            redraw.insert(Redraw::SCREEN);
        }

        if redraw.contains(Redraw::SKILL_PANEL) && self.skill_panel_renderer.draw(state, canvas)? {
            redraw.insert(Redraw::SCREEN);
        }

        if redraw.contains(Redraw::SCREEN) {
            self.draw_screen(state, canvas)?;
        }

        Ok(true)
    }

    fn draw_level(&mut self, state: &SceneStateLevel, canvas: &mut Canvas<Window>) -> Result<()> {
        match self.render_strategy {
            RenderStrategy::Blend => self.draw_level_strategy_blend(state, canvas)?,
            RenderStrategy::Stencil(..) if self.objects_merge.len() == 0 => {
                self.draw_level_strategy_stencil_no_mergeables(state, canvas)?
            }
            _ => self.draw_level_strategy_stencil(state, canvas)?,
        };

        Ok(())
    }

    fn draw_level_strategy_blend(
        &mut self,
        state: &SceneStateLevel,
        canvas: &mut Canvas<Window>,
    ) -> Result<()> {
        with_texture_canvas(canvas, &mut self.texture_level, |canvas| -> Result<()> {
            copy_texture(canvas, &mut self.texture_terrain, SDL_BLENDMODE_NONE)?;
            blit_objects(
                canvas,
                state,
                &mut self.objects_merge,
                self.blend_mode_merge,
            )?;
            blit_objects(
                canvas,
                state,
                &mut self.objects_background,
                self.blend_mode_background,
            )?;
            blit_objects(
                canvas,
                state,
                &mut self.objects_foreground,
                SDL_BLENDMODE_BLEND,
            )?;

            Ok(())
        })?;

        Ok(())
    }

    fn draw_level_strategy_stencil_no_mergeables(
        &mut self,
        state: &SceneStateLevel,
        canvas: &mut Canvas<Window>,
    ) -> Result<()> {
        if let RenderStrategy::Stencil(StencilTextures { .. }) = &mut self.render_strategy
            && self.objects_merge.len() == 0
        {
            with_texture_canvas(canvas, &mut self.texture_level, |canvas| -> Result<()> {
                canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
                canvas.clear();

                blit_objects(
                    canvas,
                    state,
                    &mut self.objects_background,
                    SDL_BLENDMODE_BLEND,
                )?;
                copy_texture(canvas, &mut self.texture_terrain, SDL_BLENDMODE_BLEND)?;
                blit_objects(
                    canvas,
                    state,
                    &mut self.objects_foreground,
                    SDL_BLENDMODE_BLEND,
                )?;

                Ok(())
            })?;

            Ok(())
        } else {
            bail!("unreachable")
        }
    }

    fn draw_level_strategy_stencil(
        &mut self,
        state: &SceneStateLevel,
        canvas: &mut Canvas<Window>,
    ) -> Result<()> {
        if let RenderStrategy::Stencil(StencilTextures {
            stencil_terrain,
            intermediate_terrain,
        }) = &mut self.render_strategy
            && self.objects_merge.len() > 0
        {
            with_texture_canvas(canvas, &mut self.texture_level, |canvas| -> Result<()> {
                copy_texture(canvas, &mut self.texture_terrain, SDL_BLENDMODE_NONE)?;
                blit_objects(canvas, state, &mut self.objects_merge, SDL_BLENDMODE_BLEND)?;

                Ok(())
            })?;

            with_texture_canvas(canvas, intermediate_terrain, |canvas| -> Result<()> {
                copy_texture(canvas, stencil_terrain, SDL_BLENDMODE_NONE)?;
                copy_texture(canvas, &mut self.texture_level, SDL_BLENDMODE_MOD)?;

                Ok(())
            })?;

            with_texture_canvas(canvas, &mut self.texture_level, |canvas| -> Result<()> {
                canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
                canvas.clear();

                blit_objects(
                    canvas,
                    state,
                    &mut self.objects_background,
                    SDL_BLENDMODE_BLEND,
                )?;
                copy_texture(canvas, intermediate_terrain, SDL_BLENDMODE_BLEND)?;
                blit_objects(
                    canvas,
                    state,
                    &mut self.objects_foreground,
                    SDL_BLENDMODE_BLEND,
                )?;

                Ok(())
            })?;

            Ok(())
        } else {
            bail!("unreachable")
        }
    }

    fn draw_screen(&mut self, state: &SceneStateLevel, canvas: &mut Canvas<Window>) -> Result<()> {
        let (frame_x, frame_y) = self.minimap_frame_position(state);

        with_texture_canvas(canvas, &mut self.texture_screen, |canvas| -> Result<()> {
            canvas.set_draw_color(Color::RGBA(0, 0, 0, 0xff));
            canvas.clear();

            let skill_panel_texture = self.skill_panel_renderer.texture();
            skill_panel_texture.set_blend_mode(BlendMode::None);
            skill_panel_texture.set_scale_mode(ScaleMode::Nearest);
            canvas.copy(
                skill_panel_texture,
                None,
                SdlRect::new(
                    0,
                    LEVEL_HEIGHT as i32,
                    SCREEN_WIDTH as u32,
                    SKILL_PANEL_HEIGHT as u32,
                ),
            )?;

            self.texture_level.set_blend_mode(BlendMode::None);
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

        Ok(())
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

fn copy_texture<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    texture: &mut Texture,
    blend_mode: SDL_BlendMode,
) -> Result<()> {
    if !apply_blend_mode(texture, blend_mode) {
        bail!("failed to apply blend mode");
    }

    texture.set_scale_mode(ScaleMode::Nearest);
    canvas
        .copy(texture, None, None)
        .map_err(anyhow::Error::from)
}

fn blit_objects<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    state: &SceneStateLevel,
    objects: &mut [Object],
    blend_mode: SDL_BlendMode,
) -> Result<()> {
    for object in objects {
        if !apply_blend_mode(&mut object.sprite.texture(), blend_mode) {
            bail!("failed to apply blend mode");
        }

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
}
