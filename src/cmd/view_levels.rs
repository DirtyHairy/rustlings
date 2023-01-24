use super::util::{create_pixel_format, create_window, read_ground, read_levels, timestamp};
use crate::{
    file::{
        self,
        level::{Level, Object, TerrainTile},
        sprite::Bitmap,
    },
    sdl_display::SDLSprite,
};

use anyhow::{anyhow, Context, Result};
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    render::{BlendMode, Canvas, RenderTarget, Texture, TextureCreator},
};
use std::{
    cmp::{max, min},
    path::Path,
    thread::sleep,
    time::Duration,
};

const LEVEL_WIDTH: u32 = 1600;
const LEVEL_HEIGHT: u32 = 160;
const TICK_TIME_MSEC: u32 = 1000 / 15;
const VGASPEC_PALETTE_MAP: [u8; 8] = [0, 9, 10, 11, 12, 13, 14, 15];
const VGASPEC_POSITION: usize = 304;

type ObjectSprites<'a> = Vec<Vec<Option<SDLSprite<'a>>>>;

struct GameData {
    levels: Vec<Level>,
    ground_data: Vec<file::ground::Content>,
    tileset: Vec<file::tileset::Content>,
    vgaspec: Vec<file::vgaspec::Content>,
}

struct DrawState<'a> {
    background: Texture<'a>,
    mask: Texture<'a>,
    workbench: Texture<'a>,
    compose_target: Texture<'a>,
    object_sprites: ObjectSprites<'a>,
}

fn dump_level(level: &Level) -> () {
    println!("{}", level);
    println!();
}

fn prepare_palette_tiled(data: &GameData, graphics_set: usize) -> Result<[u32; 16]> {
    let pixel_format = create_pixel_format()?;

    Ok(data
        .ground_data
        .get(graphics_set)
        .context("invalid tileset index")?
        .palettes
        .custom
        .map(|(r, g, b)| Color::RGBA(r as u8, g as u8, b as u8, 0xff).to_u32(&pixel_format)))
}

fn prepare_palette_spec(data: &GameData, i_spec: usize) -> Result<[u32; 16]> {
    let pixel_format = create_pixel_format()?;

    Ok(data
        .vgaspec
        .get(i_spec)
        .context("invalid tileset index")?
        .palette
        .map(|(r, g, b)| Color::RGBA(r as u8, g as u8, b as u8, 0xff).to_u32(&pixel_format)))
}

fn prepare_palette(data: &GameData, level_index: usize) -> Result<[u32; 16]> {
    let level = data
        .levels
        .get(level_index)
        .ok_or(anyhow!("invalid level"))?;

    Ok(if level.extended_graphics_set > 0 {
        prepare_palette_spec(data, level.extended_graphics_set as usize - 1)?
    } else {
        prepare_palette_tiled(data, level.graphics_set as usize)?
    })
}

fn compose_tile_onto_background(
    tile: &TerrainTile,
    bitmap: &Bitmap,
    background_data: &mut Vec<u8>,
) -> () {
    for x in 0..bitmap.width {
        for y in 0..bitmap.height {
            let y_transformed = if tile.flip_y {
                bitmap.height - 1 - y
            } else {
                y
            };

            let x_dest = tile.x + x as i32;
            let y_dest = tile.y + y as i32;
            if x_dest < 0
                || x_dest >= LEVEL_WIDTH as i32
                || y_dest < 0
                || y_dest >= LEVEL_HEIGHT as i32
            {
                continue;
            }

            let src_index = (y_transformed * bitmap.width + x) as usize;
            let dest_index = (y_dest * LEVEL_WIDTH as i32 + x_dest) as usize;

            if tile.do_not_overwrite {
                if background_data[dest_index] == 255 && !bitmap.transparency[src_index] {
                    background_data[dest_index] = bitmap.data[src_index];
                }
            } else if tile.remove_terrain {
                if !bitmap.transparency[src_index] {
                    background_data[dest_index] = 255
                }
            } else {
                if !bitmap.transparency[src_index] {
                    background_data[dest_index] = bitmap.data[src_index];
                }
            }
        }
    }
}

fn update_texture(texture: &mut Texture, data: &[u32], dest: Rect, pitch: usize) -> Result<()> {
    let texture_data_8: &[u8];
    unsafe {
        let (_, x, _) = data.align_to();
        assert_eq!(x.len(), 4 * data.len());

        texture_data_8 = x;
    }

    texture.update(dest, texture_data_8, pitch)?;

    Ok(())
}

fn compose_level(
    data: &GameData,
    index: usize,
    palette: &[u32; 16],
    background_texture: &mut sdl2::render::Texture,
    mask_texture: &mut sdl2::render::Texture,
) -> Result<()> {
    let level = data.levels.get(index).context("invalid level ID")?;
    let tiles = data
        .tileset
        .get(level.graphics_set as usize)
        .context("invalid tile set")?;

    let mut background_data: Vec<u8> = vec![255; LEVEL_WIDTH as usize * LEVEL_HEIGHT as usize];

    if level.extended_graphics_set > 0 {
        let vgaspec = data
            .vgaspec
            .get(level.extended_graphics_set as usize - 1)
            .ok_or(anyhow!("bad extended graphics set"))?;

        for y in 0..vgaspec.bitmap.height {
            for x in 0..vgaspec.bitmap.width {
                let i_src = y * vgaspec.bitmap.width + x;
                let i_dest = y * LEVEL_WIDTH as usize + VGASPEC_POSITION + x;

                background_data[i_dest] = if vgaspec.bitmap.transparency[i_src] {
                    255
                } else {
                    VGASPEC_PALETTE_MAP[vgaspec.bitmap.data[i_src] as usize]
                };
            }
        }
    }

    for tile in &level.terrain_tiles {
        let bitmap_optional = tiles.tiles.get(tile.id as usize).and_then(|x| x.as_ref());

        match bitmap_optional {
            None => continue,
            Some(bitmap) => {
                compose_tile_onto_background(tile, bitmap, &mut background_data);
            }
        }
    }

    let mut texture_data = vec![0u32; LEVEL_WIDTH as usize * LEVEL_HEIGHT as usize];
    let mut mask_data = vec![0u32; LEVEL_WIDTH as usize * LEVEL_HEIGHT as usize];

    for x in 0..LEVEL_WIDTH {
        for y in 0..LEVEL_HEIGHT {
            let index = y * LEVEL_WIDTH + x;

            texture_data[index as usize] = if background_data[index as usize] == 255 {
                0
            } else {
                palette[background_data[index as usize] as usize]
            };

            mask_data[index as usize] = if background_data[index as usize] == 255 {
                0
            } else {
                0xffffffff
            };
        }
    }

    update_texture(
        background_texture,
        &texture_data,
        Rect::new(0, 0, LEVEL_WIDTH, LEVEL_HEIGHT),
        4 * LEVEL_WIDTH as usize,
    )?;

    update_texture(
        mask_texture,
        &mask_data,
        Rect::new(0, 0, LEVEL_WIDTH, LEVEL_HEIGHT),
        4 * LEVEL_WIDTH as usize,
    )?;

    Ok(())
}

fn build_object_sprites<'a, T>(
    data: &GameData,
    palette: &[u32; 16],
    texture_creator: &'a TextureCreator<T>,
) -> Result<ObjectSprites<'a>> {
    let mut sprites: ObjectSprites = Vec::with_capacity(data.tileset.len());

    for tileset in &data.tileset {
        let mut object_sprites: Vec<Option<SDLSprite>> = Vec::with_capacity(16);

        for object_sprite in &tileset.object_sprites {
            object_sprites.push(match object_sprite {
                None => None,
                Some(sprite) => Some(SDLSprite::from_sprite(sprite, palette, texture_creator)?),
            })
        }

        sprites.push(object_sprites);
    }

    Ok(sprites)
}

fn draw_objects<T: RenderTarget, F>(
    level: &Level,
    frame: u64,
    object_sprites: &mut ObjectSprites,
    canvas: &mut Canvas<T>,
    predicate: F,
) -> Result<()>
where
    F: Fn(&Object) -> bool,
{
    for object in &level.objects {
        if !predicate(object) {
            continue;
        };

        if let Some(sprite) = object_sprites
            .get_mut(level.graphics_set as usize)
            .and_then(|sprites| sprites.get_mut(object.id as usize))
            .and_then(|x| x.as_mut())
        {
            sprite.blit(
                canvas,
                object.x,
                object.y,
                (frame % sprite.frame_count as u64) as usize,
                1,
                object.flip_y,
            )?;
        }
    }

    Ok(())
}

fn draw<T: RenderTarget>(
    texture: &mut Texture,
    canvas: &mut Canvas<T>,
    blend_mode: BlendMode,
) -> Result<()> {
    texture.set_blend_mode(blend_mode);

    canvas
        .copy(
            texture,
            Rect::new(0, 0, LEVEL_WIDTH, LEVEL_HEIGHT),
            Rect::new(0, 0, LEVEL_WIDTH, LEVEL_HEIGHT),
        )
        .map_err(|s| anyhow!(s))
}

fn render<'a>(
    x: u32,
    zoom: u32,
    frame: u64,
    level: &Level,
    draw_state: &mut DrawState<'a>,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
) -> Result<()> {
    let compose_target = &mut draw_state.compose_target;
    let object_sprites = &mut draw_state.object_sprites;
    let background = &mut draw_state.background;
    let mask = &mut draw_state.mask;
    let workbench = &mut draw_state.workbench;

    canvas.with_texture_canvas(compose_target, |canvas| {
        let _ = draw(background, canvas, BlendMode::None);

        let _ = draw_objects(level, frame, object_sprites, canvas, |o| {
            o.draw_only_over_terrain
        });
    })?;

    canvas.with_texture_canvas(workbench, |canvas| {
        let _ = draw(mask, canvas, BlendMode::None);
        let _ = draw(compose_target, canvas, BlendMode::Mod);
    })?;

    canvas.with_texture_canvas(compose_target, |canvas| {
        canvas.clear();

        let _ = draw_objects(level, frame, object_sprites, canvas, |o| {
            o.do_not_overwrite && !o.draw_only_over_terrain
        });

        let _ = draw(workbench, canvas, BlendMode::Blend);

        let _ = draw_objects(level, frame, object_sprites, canvas, |o| {
            !o.do_not_overwrite && !o.draw_only_over_terrain
        });
    })?;

    canvas.clear();

    compose_target.set_blend_mode(BlendMode::None);
    canvas
        .copy(
            &draw_state.compose_target,
            Rect::new(x as i32, 0, 320 * 4 / zoom, LEVEL_HEIGHT),
            Rect::new(
                0,
                800 - (800 + LEVEL_HEIGHT as i32 * zoom as i32) / 2,
                1280,
                LEVEL_HEIGHT * zoom,
            ),
        )
        .map_err(|s| anyhow!(s))?;

    canvas.present();

    Ok(())
}

fn clamp_x(x: i32, zoom: u32) -> u32 {
    max(
        min(x as i32 + 10, LEVEL_WIDTH as i32 - 320 * 4 / zoom as i32) as i32 - 10,
        0,
    ) as u32
}

fn transform_x_for_zoom(x: u32, old_zoom: u32, zoom: u32) -> u32 {
    clamp_x(
        (x + 320 * 2 / old_zoom) as i32 - (320 * 2 / zoom) as i32,
        zoom,
    )
}

fn create_canvas_texture<T>(texture_creator: &TextureCreator<T>) -> Result<Texture> {
    texture_creator
        .create_texture_target(PixelFormatEnum::RGBA8888, LEVEL_WIDTH, LEVEL_HEIGHT)
        .map_err(|e| anyhow!(e))
}

fn switch_level<'a, T>(
    draw_state: &mut DrawState<'a>,
    data: &GameData,
    level_index: usize,
    texture_creator: &'a TextureCreator<T>,
) -> Result<()> {
    let palette = prepare_palette(data, level_index)?;

    draw_state.object_sprites = build_object_sprites(data, &palette, texture_creator)?;
    compose_level(
        data,
        level_index,
        &palette,
        &mut draw_state.background,
        &mut draw_state.mask,
    )?;

    Ok(())
}

fn display_levels<'a>(data: &GameData, start_level: usize) -> Result<()> {
    let sdl_context = sdl2::init().map_err(|s| anyhow!(s))?;
    let sdl_video = sdl_context.video().map_err(|s| anyhow!(s))?;
    let mut event_pump = sdl_context.event_pump().map_err(|s| anyhow!(s))?;
    let window = create_window(&sdl_video)?;
    let mut canvas = window.into_canvas().accelerated().present_vsync().build()?;
    let texture_creator = canvas.texture_creator();

    let mut draw_state = DrawState {
        background: create_canvas_texture(&texture_creator)?,
        mask: create_canvas_texture(&texture_creator)?,
        compose_target: create_canvas_texture(&texture_creator)?,
        workbench: create_canvas_texture(&texture_creator)?,
        object_sprites: Vec::new(),
    };

    let mut running = true;
    let mut frame: u64 = 0;
    let now = timestamp();
    let mut game_time = 0;
    let mut i_level = start_level;
    let mut zoom = 4;
    let mut x: u32 = clamp_x(data.levels[i_level].start_x as i32, zoom);

    let mut left = false;
    let mut right = false;
    let mut level_changed = true;
    let mut screen_dirty = true;

    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => running = false,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => match code {
                    Keycode::Escape => running = false,
                    Keycode::Left => left = true,
                    Keycode::Right => right = true,
                    Keycode::Up => {
                        i_level = (i_level + 1) % data.levels.len();
                        x = clamp_x(data.levels[i_level].start_x as i32, zoom);

                        level_changed = true;
                    }
                    Keycode::Down => {
                        i_level = ((i_level + data.levels.len()) - 1) % data.levels.len();
                        x = clamp_x(data.levels[i_level].start_x as i32, zoom);

                        level_changed = true;
                    }
                    Keycode::Plus => {
                        let old_zoom = zoom;

                        zoom = match zoom {
                            1 => 2,
                            2 => 4,
                            _ => zoom,
                        };

                        x = transform_x_for_zoom(x, old_zoom, zoom);

                        screen_dirty = true;
                    }

                    Keycode::Minus => {
                        let old_zoom = zoom;

                        zoom = match zoom {
                            4 => 2,
                            2 => 1,
                            _ => zoom,
                        };

                        x = transform_x_for_zoom(x, old_zoom, zoom);

                        screen_dirty = true;
                    }
                    _ => (),
                },
                Event::KeyUp {
                    keycode: Some(code),
                    ..
                } => match code {
                    Keycode::Left => left = false,
                    Keycode::Right => right = false,
                    _ => (),
                },
                _ => (),
            }
        }

        if left {
            x = clamp_x(x as i32 - 10, zoom);
            screen_dirty = true;
        } else if right {
            x = clamp_x(x as i32 + 10, zoom);
            screen_dirty = true;
        }

        let time = timestamp() - now;
        let ticks = (time - game_time) / TICK_TIME_MSEC;
        if ticks > 0 {
            game_time += ticks * TICK_TIME_MSEC;
            frame += ticks as u64;
            screen_dirty = true;
        }

        if level_changed {
            switch_level(&mut draw_state, data, i_level, &texture_creator)?;
            dump_level(&data.levels[i_level]);

            screen_dirty = true;
            level_changed = false;
            frame = 0;
        }

        if screen_dirty {
            render(
                x,
                zoom,
                frame,
                &data.levels[i_level],
                &mut draw_state,
                &mut canvas,
            )?;

            screen_dirty = false;
        }

        sleep(Duration::from_millis(TICK_TIME_MSEC as u64 / 5));
    }

    Ok(())
}

pub fn main(data_path: &Path, start_level: Option<&String>) -> Result<()> {
    let mut levels: Vec<Level> = Vec::new();

    for i in 0..10 {
        levels.append(&mut read_levels(
            data_path
                .join(format!("level00{}.dat", i))
                .to_str()
                .unwrap(),
        )?)
    }

    let (ground_data, tileset) = read_ground(data_path)?;

    let mut i_start = 0;

    if let Some(pattern) = start_level {
        for (index, level) in levels.iter().enumerate() {
            if level
                .name
                .to_lowercase()
                .contains(pattern.to_lowercase().as_str())
            {
                i_start = index;
                break;
            }
        }
    }

    let mut vgaspec: Vec<file::vgaspec::Content> = Vec::with_capacity(4);
    for i in 0..4 {
        vgaspec.push(file::vgaspec::Content::read(data_path, i)?);
    }

    display_levels(
        &GameData {
            levels,
            ground_data,
            tileset,
            vgaspec,
        },
        i_start,
    )?;

    Ok(())
}
