use rustlings::game_data::{
    Bitmap, DIFFICULTY_RATINGS, GameData, Level, OBJECTS_PER_TILESET, Object, PALETTE_SIZE,
    PaletteEntry, TerrainTile, read_game_data,
};
use rustlings::sdl_rendering::SDLSprite;
use rustlings::sdl3_aux::get_canvas_vsync;

use crate::cmd::util::{create_window, timestamp};
use anyhow::{Result, anyhow};
use sdl3::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormat},
    rect::Rect,
    render::{BlendMode, Canvas, RenderTarget, Texture, TextureCreator},
};
use std::{
    cmp::{max, min},
    path::Path,
    thread::sleep,
    time::Duration,
};

const LEVELS_TOTAL: usize = 120;
const LEVEL_WIDTH: u32 = 1600;
const LEVEL_HEIGHT: u32 = 160;
const TICK_TIME_MSEC: u32 = 1000 / 15;
const VGASPEC_POSITION: usize = 304;

type ObjectSprites<'a> = Vec<Vec<Option<SDLSprite<'a>>>>;

struct DrawState<'a> {
    background: Texture<'a>,
    mask: Texture<'a>,
    workbench: Texture<'a>,
    compose_target: Texture<'a>,
    object_sprites: ObjectSprites<'a>,
}

fn dump_level(level_index: usize, level: &Level) -> () {
    println!(
        "{} {}:",
        DIFFICULTY_RATINGS[level_index / 30],
        level_index % 30 + 1
    );

    println!("{}", level);
    println!();
}

fn get_palette(data: &GameData, level: &Level) -> Result<[PaletteEntry; PALETTE_SIZE]> {
    if level.extended_graphics_set > 0 {
        data.special_backgrounds
            .get(level.extended_graphics_set as usize - 1)
            .ok_or(anyhow!(
                "invalid extended graphics set {}",
                level.extended_graphics_set
            ))
            .map(|x| x.palette)
    } else {
        data.tilesets
            .get(level.graphics_set as usize)
            .ok_or(anyhow!("invlid graphics set {}", level.graphics_set))
            .map(|x| x.palettes.custom)
    }
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
    level: &Level,
    palette: &[PaletteEntry; PALETTE_SIZE],
    background_texture: &mut Texture,
    mask_texture: &mut Texture,
) -> Result<()> {
    let mut background_data: Vec<u8> = vec![255; LEVEL_WIDTH as usize * LEVEL_HEIGHT as usize];

    if level.extended_graphics_set > 0 {
        let special_background = data
            .special_backgrounds
            .get(level.extended_graphics_set as usize - 1)
            .ok_or(anyhow!("bad extended graphics set"))?;

        for y in 0..special_background.bitmap.height {
            for x in 0..special_background.bitmap.width {
                let i_src = y * special_background.bitmap.width + x;
                let i_dest = y * LEVEL_WIDTH as usize + VGASPEC_POSITION + x;

                background_data[i_dest] = if special_background.bitmap.transparency[i_src] {
                    255
                } else {
                    special_background.bitmap.data[i_src]
                };
            }
        }
    }

    for tile in &level.terrain_tiles {
        let bitmap_optional = data
            .tilesets
            .get(level.graphics_set as usize)
            .ok_or(anyhow!("bad graphics set"))?
            .tiles
            .get(tile.id as usize)
            .and_then(|x| x.as_ref());

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
                let (r, g, b) = palette[background_data[index as usize] as usize];
                Color::RGBA(r, g, b, 0xff).to_u32(&PixelFormat::RGBA8888)
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
    palette: &[PaletteEntry; PALETTE_SIZE],
    texture_creator: &'a TextureCreator<T>,
) -> Result<ObjectSprites<'a>> {
    let mut sprites: ObjectSprites = Vec::with_capacity(data.tilesets.len());

    for tileset in &data.tilesets {
        let mut object_sprites: Vec<Option<SDLSprite>> = Vec::with_capacity(OBJECTS_PER_TILESET);

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

    canvas.copy(
        texture,
        Rect::new(0, 0, LEVEL_WIDTH, LEVEL_HEIGHT),
        Rect::new(0, 0, LEVEL_WIDTH, LEVEL_HEIGHT),
    )?;

    Ok(())
}

fn render<'a>(
    x: u32,
    zoom: u32,
    frame: u64,
    level: &Level,
    draw_state: &mut DrawState<'a>,
    canvas: &mut Canvas<sdl3::video::Window>,
) -> Result<()> {
    let DrawState {
        compose_target,
        background,
        mask,
        object_sprites,
        workbench,
    } = draw_state;

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

    let (window_width, window_height) = canvas.output_size()?;

    compose_target.set_blend_mode(BlendMode::None);
    canvas.copy(
        &draw_state.compose_target,
        Rect::new(x as i32, 0, 320 * 4 / zoom, LEVEL_HEIGHT),
        Rect::new(
            0,
            (window_height - (window_height + (LEVEL_HEIGHT * zoom * window_height) / 800) / 2)
                as i32,
            window_width,
            LEVEL_HEIGHT * zoom * window_height / 800,
        ),
    )?;

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

fn create_canvas_texture<'a, T>(texture_creator: &'a TextureCreator<T>) -> Result<Texture<'a>> {
    let mut texture = texture_creator
        .create_texture_target(PixelFormat::RGBA8888, LEVEL_WIDTH, LEVEL_HEIGHT)
        .map_err(|e| anyhow!(e))?;

    texture.set_scale_mode(sdl3::render::ScaleMode::Nearest);

    Ok(texture)
}

fn switch_level<'a, T>(
    draw_state: &mut DrawState<'a>,
    data: &GameData,
    level: &Level,
    texture_creator: &'a TextureCreator<T>,
) -> Result<()> {
    let palette = get_palette(data, level)?;

    draw_state.object_sprites = build_object_sprites(data, &palette, texture_creator)?;
    compose_level(
        data,
        level,
        &palette,
        &mut draw_state.background,
        &mut draw_state.mask,
    )?;

    Ok(())
}

fn display_levels<'a>(data: &GameData, start_level: usize) -> Result<()> {
    let sdl_context = sdl3::init().map_err(|s| anyhow!(s))?;
    sdl3::hint::set("SDL_RENDER_VSYNC", "1");
    sdl3::hint::set("SDL_FRAMEBUFFER_ACCELERATION", "1");

    let sdl_video = sdl_context.video().map_err(|s| anyhow!(s))?;
    let mut event_pump = sdl_context.event_pump().map_err(|s| anyhow!(s))?;
    let window = create_window(&sdl_video, true)?;

    let mut canvas = window.into_canvas();
    println!(
        "canvas initialized; vsync: {}, driver: {}",
        get_canvas_vsync(&canvas),
        canvas.renderer_name
    );

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
    let mut level_index = start_level;
    let mut zoom = 4;
    let mut level = data.resolve_level(level_index).unwrap();
    let mut x: u32 = clamp_x(level.start_x as i32, zoom);

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
                        level_index = (level_index + 1) % LEVELS_TOTAL;
                        level = data.resolve_level(level_index).unwrap();

                        x = clamp_x(level.start_x as i32, zoom);

                        level_changed = true;
                    }
                    Keycode::Down => {
                        level_index = ((level_index + LEVELS_TOTAL) - 1) % LEVELS_TOTAL;
                        level = data.resolve_level(level_index).unwrap();

                        x = clamp_x(level.start_x as i32, zoom);

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
            switch_level(&mut draw_state, data, &level, &texture_creator)?;
            dump_level(level_index, &level);

            screen_dirty = true;
            level_changed = false;
            frame = 0;
        }

        if screen_dirty {
            render(x, zoom, frame, &level, &mut draw_state, &mut canvas)?;

            screen_dirty = false;
        }

        sleep(Duration::from_millis(TICK_TIME_MSEC as u64 / 5));
    }

    Ok(())
}

pub fn main(path: &Path, start_level: Option<&String>) -> Result<()> {
    let data = read_game_data(path)?;

    let mut i_start = 0;

    if let Some(pattern) = start_level {
        for (index, level) in data.levels.iter().enumerate() {
            if level
                .parameters
                .name
                .to_lowercase()
                .contains(pattern.to_lowercase().as_str())
            {
                i_start = index;
                break;
            }
        }
    }

    display_levels(&data, i_start)?;

    Ok(())
}
