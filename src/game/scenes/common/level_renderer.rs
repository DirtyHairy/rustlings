use std::{array, rc::Rc};

use anyhow::{Error, Result, anyhow, bail};
use rustlings::{
    game_data::{
        Bitmap, GameData, LEVEL_HEIGHT, LEVEL_WIDTH, Level, OBJECTS_PER_TILESET, PALETTE_SIZE,
        PaletteEntry, ParticleSets, file::level,
    },
    sdl::{
        SdlAtlas, SdlAtlasBuilder, apply_blend_mode, texture_from_bitmap,
        texture_from_bitmap_mapped, with_texture_canvas,
    },
};
use sdl3::{
    pixels::{Color, PixelFormat},
    rect::Point,
    render::{
        BlendMode, Canvas, FPoint, RenderTarget, ScaleMode, Texture, TextureAccess, TextureCreator,
    },
    sys::blendmode::{
        SDL_BLENDFACTOR_DST_ALPHA, SDL_BLENDFACTOR_ONE, SDL_BLENDFACTOR_ONE_MINUS_DST_ALPHA,
        SDL_BLENDFACTOR_ONE_MINUS_SRC_ALPHA, SDL_BLENDFACTOR_SRC_ALPHA, SDL_BLENDFACTOR_ZERO,
        SDL_BLENDMODE_BLEND, SDL_BLENDMODE_MOD, SDL_BLENDMODE_NONE, SDL_BLENDOPERATION_ADD,
        SDL_BlendMode, SDL_ComposeCustomBlendMode,
    },
    video::Window,
};
use strum::{EnumCount, VariantArray};

use crate::{
    scenes::common::{BRICK_COLOR, BRICK_LINE_WIDTH, DIG_LINE_WIDTH, TerrainDiff, TerrainDiffKind},
    state::{Activity, Direction, LemmingAnimation, LemmingsState, ObjectsState},
};

const MAX_PARTICLES_PER_COLOR: usize = 99 * 80 / 8;

struct Object {
    index: usize,
    atlas_index: usize,

    x: u32,
    y: u32,
    flip: bool,
}

struct StencilTextures<'texture_creator> {
    stencil_terrain: Texture<'texture_creator>,
    intermediate_terrain: Texture<'texture_creator>,
}

enum RenderStrategy<'texture_creator> {
    Blend,
    Stencil(StencilTextures<'texture_creator>),
}

pub struct LevelRenderer<'texture_creator> {
    texture_terrain: Texture<'texture_creator>,
    texture_level: Texture<'texture_creator>,

    atlas: SdlAtlas<'texture_creator>,
    sprite_index_countdown: usize,
    sprite_index_mask: usize,

    mask_points: Vec<FPoint>,

    objects_background: Vec<Object>,
    objects_foreground: Vec<Object>,
    objects_merge: Vec<Object>,

    blend_mode_merge: SDL_BlendMode,
    blend_mode_background: SDL_BlendMode,
    blend_mode_apply_mask: SDL_BlendMode,

    render_strategy: RenderStrategy<'texture_creator>,

    palette: [PaletteEntry; PALETTE_SIZE],
    particle_buffer: [Vec<FPoint>; PALETTE_SIZE],

    game_data: Rc<GameData>,
}

impl<'texture_creator> LevelRenderer<'texture_creator> {
    pub fn new<T>(
        level: &Level,
        terrain: &Bitmap,
        game_data: Rc<GameData>,
        texture_creator: &'texture_creator TextureCreator<T>,
    ) -> Result<Self> {
        let palette = game_data.resolve_palette(level)?;

        let texture_terrain =
            texture_from_bitmap(terrain, &palette, texture_creator, TextureAccess::Target)?;

        let texture_level = texture_creator.create_texture_target(
            PixelFormat::RGBA8888,
            LEVEL_WIDTH,
            LEVEL_HEIGHT,
        )?;

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

        let blend_mode_apply_mask = SDL_ComposeCustomBlendMode(
            SDL_BLENDFACTOR_ZERO,
            SDL_BLENDFACTOR_SRC_ALPHA,
            SDL_BLENDOPERATION_ADD,
            SDL_BLENDFACTOR_ZERO,
            SDL_BLENDFACTOR_SRC_ALPHA,
            SDL_BLENDOPERATION_ADD,
        );

        let mut texture_probe =
            texture_creator.create_texture(PixelFormat::RGBA8888, TextureAccess::Static, 1, 1)?;

        let render_strategy = if apply_blend_mode(&mut texture_probe, blend_mode_merge)
            && apply_blend_mode(&mut texture_probe, blend_mode_background)
            && apply_blend_mode(&mut texture_probe, blend_mode_apply_mask)
        {
            RenderStrategy::Blend
        } else {
            println!("custom blend modes not supported; falling back to stencil based rendering");

            let stencil_terrain = texture_from_bitmap_mapped(
                terrain,
                &palette,
                texture_creator,
                TextureAccess::Target,
                |Color { a, .. }| {
                    if a == 0 {
                        Color::RGBA(0, 0, 0, 0)
                    } else {
                        Color::RGBA(0xff, 0xff, 0xff, 0xff)
                    }
                },
            )?;

            let intermediate_terrain = texture_creator.create_texture_target(
                PixelFormat::RGBA8888,
                LEVEL_WIDTH,
                LEVEL_HEIGHT,
            )?;

            RenderStrategy::Stencil(StencilTextures {
                stencil_terrain,
                intermediate_terrain,
            })
        };

        let mut atlas_builder =
            SdlAtlasBuilder::with_capacity(LemmingAnimation::COUNT + OBJECTS_PER_TILESET + 1);

        [Direction::Right, Direction::Left]
            .iter()
            .cloned()
            .for_each(|direction| {
                LemmingAnimation::VARIANTS
                    .iter()
                    .copied()
                    .map(|animation| {
                        &game_data.lemming_sprites[animation.sprite(direction) as usize]
                    })
                    .for_each(|sprite| {
                        atlas_builder.add_sprite(sprite);
                    })
            });

        let object_atlas_index: Vec<Option<usize>> = game_data
            .tilesets
            .get(level.graphics_set as usize)
            .ok_or_else(|| anyhow!("invalid tileset {}", level.graphics_set))?
            .object_sprites
            .iter()
            .map(|sprite| sprite.as_ref().map(|s| atlas_builder.add_sprite(s)))
            .collect();

        let sprite_index_countdown = atlas_builder.add_sprite(&game_data.font_countdown);

        let (sprite_index_mask, mask_points): (usize, Vec<FPoint>) =
            if matches!(render_strategy, RenderStrategy::Blend) {
                let mut first_index: Option<usize> = None;

                for sprite in &game_data.mask_sprites {
                    first_index.get_or_insert(atlas_builder.add_sprite(sprite));
                }

                (first_index.unwrap(), Vec::with_capacity(16 * 22))
            } else {
                (0, Vec::new())
            };

        let atlas = atlas_builder.build(&palette, texture_creator)?;
        println!("built atlas, size is {}x{}", atlas.width(), atlas.height());

        let objects_merge =
            create_objects(&object_atlas_index, level, |o| o.draw_only_over_terrain)?;

        let objects_foreground = create_objects(&object_atlas_index, level, |o| {
            !o.draw_only_over_terrain && !o.do_not_overwrite
        })?;

        let objects_background = create_objects(&object_atlas_index, level, |o| {
            !o.draw_only_over_terrain && o.do_not_overwrite
        })?;

        Ok(Self {
            texture_terrain,
            texture_level,

            atlas,
            sprite_index_countdown,
            sprite_index_mask,

            mask_points,

            objects_merge,
            objects_foreground,
            objects_background,

            blend_mode_merge,
            blend_mode_background,
            blend_mode_apply_mask,

            render_strategy,

            palette,
            particle_buffer: array::from_fn(|_| Vec::with_capacity(MAX_PARTICLES_PER_COLOR)),

            game_data: Rc::clone(&game_data),
        })
    }

    pub fn texture(&mut self) -> &mut Texture<'texture_creator> {
        &mut self.texture_level
    }

    pub fn apply_diff(&mut self, canvas: &mut Canvas<Window>, diff: &[TerrainDiff]) -> Result<()> {
        match self.render_strategy {
            RenderStrategy::Blend => self.apply_diff_gpu(canvas, diff),
            RenderStrategy::Stencil(_) => self.apply_diff_fallback(canvas, diff),
        }?;

        Ok(())
    }

    fn apply_diff_gpu(&mut self, canvas: &mut Canvas<Window>, diff: &[TerrainDiff]) -> Result<()> {
        with_texture_canvas(canvas, &mut self.texture_terrain, |canvas| -> Result<()> {
            for &entry in diff {
                match entry.kind {
                    TerrainDiffKind::Dig => {
                        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
                        canvas.set_blend_mode(BlendMode::None);

                        canvas.draw_line(
                            Point::new(entry.x, entry.y),
                            Point::new(entry.x + DIG_LINE_WIDTH as i32 - 1, entry.y),
                        )?;
                    }
                    TerrainDiffKind::Mask(sprite, frame) => {
                        self.atlas.apply_blend_mode(self.blend_mode_apply_mask);

                        self.atlas.blit(
                            canvas,
                            self.sprite_index_mask + sprite as usize,
                            entry.x,
                            entry.y,
                            frame,
                            false,
                            false,
                        )?;
                    }
                    TerrainDiffKind::Brick => {
                        let (r, g, b) = self.palette[BRICK_COLOR];

                        canvas.set_draw_color(Color::RGBA(r, g, b, 0xff));
                        canvas.set_blend_mode(BlendMode::None);

                        canvas.draw_line(
                            Point::new(entry.x, entry.y),
                            Point::new(entry.x + BRICK_LINE_WIDTH as i32 - 1, entry.y),
                        )?;
                    }
                }
            }

            Ok(())
        })
    }

    fn apply_diff_fallback(
        &mut self,
        canvas: &mut Canvas<Window>,
        diff: &[TerrainDiff],
    ) -> Result<()> {
        let RenderStrategy::Stencil(StencilTextures {
            stencil_terrain, ..
        }) = &mut self.render_strategy
        else {
            unreachable!();
        };

        for entry in diff {
            if let TerrainDiffKind::Mask(sprite, frame) = entry.kind {
                let sprite = &self.game_data.mask_sprites[sprite as usize];
                let frame = &sprite.frames[frame];

                self.mask_points.clear();

                let mut i = 0;
                for y in 0..sprite.height {
                    for x in 0..sprite.width {
                        if frame.transparency[i] {
                            self.mask_points.push(FPoint::new(
                                (entry.x + x as i32) as f32,
                                (entry.y + y as i32) as f32,
                            ));
                        }

                        i += 1;
                    }
                }
            }

            let (brick_color_terrain, brick_color_stencil) = {
                let (r, g, b) = self.palette[BRICK_COLOR];

                (
                    Color::RGBA(r, g, b, 0xff),
                    Color::RGBA(0xff, 0xff, 0xff, 0xff),
                )
            };

            let targets: &mut [(&mut Texture<'texture_creator>, Color)] =
                if self.objects_merge.is_empty() {
                    &mut [(&mut self.texture_terrain, brick_color_terrain)]
                } else {
                    &mut [
                        (&mut self.texture_terrain, brick_color_terrain),
                        (stencil_terrain, brick_color_stencil),
                    ]
                };

            for (texture, brick_color) in targets.iter_mut() {
                with_texture_canvas(canvas, texture, |canvas| -> Result<()> {
                    match entry.kind {
                        TerrainDiffKind::Dig => {
                            canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
                            canvas.set_blend_mode(BlendMode::None);

                            canvas.draw_line(
                                Point::new(entry.x, entry.y),
                                Point::new(entry.x + DIG_LINE_WIDTH as i32 - 1, entry.y),
                            )?;
                        }
                        TerrainDiffKind::Mask(_, _) => {
                            canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
                            canvas.set_blend_mode(BlendMode::None);

                            canvas.draw_points(&self.mask_points as &[FPoint])?;
                        }
                        TerrainDiffKind::Brick => {
                            canvas.set_draw_color(*brick_color);
                            canvas.set_blend_mode(BlendMode::None);

                            canvas.draw_line(
                                Point::new(entry.x, entry.y),
                                Point::new(entry.x + BRICK_LINE_WIDTH as i32 - 1, entry.y),
                            )?;
                        }
                    }

                    Ok(())
                })?;
            }
        }

        Ok(())
    }

    pub fn draw(
        &mut self,
        objects_state: &ObjectsState,
        lemmings_state: &LemmingsState,
        canvas: &mut Canvas<Window>,
    ) -> Result<()> {
        match self.render_strategy {
            RenderStrategy::Blend => {
                self.draw_strategy_blend(objects_state, lemmings_state, canvas)?
            }
            RenderStrategy::Stencil(..) if self.objects_merge.is_empty() => {
                self.draw_strategy_stencil_no_mergeables(objects_state, lemmings_state, canvas)?
            }
            _ => self.draw_strategy_stencil(objects_state, lemmings_state, canvas)?,
        };

        Ok(())
    }

    fn draw_strategy_blend(
        &mut self,
        objects_state: &ObjectsState,
        lemmings_state: &LemmingsState,
        canvas: &mut Canvas<Window>,
    ) -> Result<()> {
        with_texture_canvas(canvas, &mut self.texture_level, |canvas| -> Result<()> {
            copy_texture(canvas, &mut self.texture_terrain, SDL_BLENDMODE_NONE)?;
            blit_objects(
                canvas,
                objects_state,
                &mut self.objects_merge,
                self.blend_mode_merge,
                &mut self.atlas,
            )?;
            blit_objects(
                canvas,
                objects_state,
                &mut self.objects_background,
                self.blend_mode_background,
                &mut self.atlas,
            )?;
            blit_objects(
                canvas,
                objects_state,
                &mut self.objects_foreground,
                SDL_BLENDMODE_BLEND,
                &mut self.atlas,
            )?;

            draw_lemmings(
                canvas,
                lemmings_state,
                &mut self.atlas,
                self.sprite_index_countdown,
                &mut self.particle_buffer,
                &self.game_data.particle_sets,
                &self.palette,
            )?;

            Ok(())
        })?;

        Ok(())
    }

    fn draw_strategy_stencil_no_mergeables(
        &mut self,
        objects_state: &ObjectsState,
        lemmings_state: &LemmingsState,
        canvas: &mut Canvas<Window>,
    ) -> Result<()> {
        let RenderStrategy::Stencil(StencilTextures { .. }) = &mut self.render_strategy else {
            unreachable!()
        };

        with_texture_canvas(canvas, &mut self.texture_level, |canvas| -> Result<()> {
            canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
            canvas.clear();

            blit_objects(
                canvas,
                objects_state,
                &mut self.objects_background,
                SDL_BLENDMODE_BLEND,
                &mut self.atlas,
            )?;
            copy_texture(canvas, &mut self.texture_terrain, SDL_BLENDMODE_BLEND)?;
            blit_objects(
                canvas,
                objects_state,
                &mut self.objects_foreground,
                SDL_BLENDMODE_BLEND,
                &mut self.atlas,
            )?;

            draw_lemmings(
                canvas,
                lemmings_state,
                &mut self.atlas,
                self.sprite_index_countdown,
                &mut self.particle_buffer,
                &self.game_data.particle_sets,
                &self.palette,
            )?;

            Ok(())
        })
    }

    fn draw_strategy_stencil(
        &mut self,
        objects_state: &ObjectsState,
        lemmings_state: &LemmingsState,
        canvas: &mut Canvas<Window>,
    ) -> Result<()> {
        let RenderStrategy::Stencil(StencilTextures {
            stencil_terrain,
            intermediate_terrain,
        }) = &mut self.render_strategy
        else {
            unreachable!()
        };

        with_texture_canvas(canvas, &mut self.texture_level, |canvas| -> Result<()> {
            copy_texture(canvas, &mut self.texture_terrain, SDL_BLENDMODE_NONE)?;
            blit_objects(
                canvas,
                objects_state,
                &mut self.objects_merge,
                SDL_BLENDMODE_BLEND,
                &mut self.atlas,
            )?;

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
                objects_state,
                &mut self.objects_background,
                SDL_BLENDMODE_BLEND,
                &mut self.atlas,
            )?;
            copy_texture(canvas, intermediate_terrain, SDL_BLENDMODE_BLEND)?;
            blit_objects(
                canvas,
                objects_state,
                &mut self.objects_foreground,
                SDL_BLENDMODE_BLEND,
                &mut self.atlas,
            )?;

            draw_lemmings(
                canvas,
                lemmings_state,
                &mut self.atlas,
                self.sprite_index_countdown,
                &mut self.particle_buffer,
                &self.game_data.particle_sets,
                &self.palette,
            )?;

            Ok(())
        })
    }
}

fn create_objects<P: Fn(&&level::Object) -> bool>(
    atlas_index: &[Option<usize>],
    level: &Level,
    predicate: P,
) -> Result<Vec<Object>> {
    level
        .objects
        .iter()
        .enumerate()
        .filter(|(_, x)| predicate(x))
        .map(|(index, o)| -> Result<Object> {
            Ok(Object {
                index,
                atlas_index: atlas_index[o.id as usize]
                    .ok_or_else(|| anyhow!("no sprite in atlas for object {}", o.id))?,
                x: o.x as u32,
                y: o.y as u32,
                flip: o.flip_y,
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
    canvas.copy(texture, None, None).map_err(Error::from)
}

fn blit_objects<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    state: &ObjectsState,
    objects: &mut [Object],
    blend_mode: SDL_BlendMode,
    atlas: &mut SdlAtlas,
) -> Result<()> {
    if !atlas.apply_blend_mode(blend_mode) {
        bail!("failed to apply blend mode");
    }

    for object in objects {
        atlas.blit(
            canvas,
            object.atlas_index,
            object.x as i32,
            object.y as i32,
            state[object.index].frame,
            false,
            object.flip,
        )?;
    }

    Ok(())
}

fn draw_lemmings<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    state: &LemmingsState,
    atlas: &mut SdlAtlas,
    sprint_index_countdown: usize,
    particle_buffer: &mut [Vec<FPoint>; PALETTE_SIZE],
    particle_sets: &ParticleSets,
    palette: &[PaletteEntry],
) -> Result<()> {
    if !atlas.apply_blend_mode(SDL_BLENDMODE_BLEND) {
        bail!("failed to apply blend mode");
    }

    let mut has_particles = false;

    for lemming in state {
        if let Some(iset) = lemming.particle_frame {
            if !has_particles {
                has_particles = true;
                particle_buffer.iter_mut().for_each(|b| b.clear());
            }

            let set = &particle_sets.particle_sets[iset];

            for i in 0..set.count {
                let particle = &particle_sets.particles[set.offset + i];

                particle_buffer[particle.color as usize].push(FPoint::new(
                    (lemming.x + particle.x as i32) as f32,
                    (lemming.y + particle.y as i32) as f32,
                ));
            }
        }

        let Some(animation) = lemming.animation else {
            continue;
        };

        let (foot_x, foot_y) = animation.foot();

        let sprite_index = match lemming.direction {
            Direction::Right => animation as usize,
            Direction::Left => animation as usize + LemmingAnimation::COUNT,
        };

        atlas.blit(
            canvas,
            sprite_index,
            lemming.x - foot_x as i32,
            lemming.y - foot_y as i32,
            lemming.frame,
            false,
            false,
        )?;

        if let Some(countdown) = lemming.countdown
            && !matches!(lemming.activity, Activity::Frying | Activity::Splatting)
        {
            atlas.blit(
                canvas,
                sprint_index_countdown,
                lemming.x - 1,
                lemming.y - foot_y as i32 - 9,
                (countdown / 16) as usize + 1,
                false,
                false,
            )?;
        }
    }

    if has_particles {
        canvas.set_blend_mode(BlendMode::None);

        for (color, particles) in particle_buffer.iter().enumerate() {
            if particles.is_empty() {
                continue;
            }

            let (r, g, b) = palette[color];
            canvas.set_draw_color(Color::RGBA(r, g, b, 0xff));

            canvas.draw_points(particles as &[FPoint])?;
        }
    }

    Ok(())
}
