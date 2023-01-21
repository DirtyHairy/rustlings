use super::util::{create_pixel_format, create_window, read_ground};
use crate::{
    file::{self, sprite::Bitmap},
    level::{Level, TerrainTile},
};

use anyhow::{anyhow, Context, Result};
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
};
use std::{
    cmp::{max, min},
    fs,
    path::Path,
    thread::sleep,
    time::Duration,
};

const LEVEL_WIDTH: u32 = 1600;
const LEVEL_HEIGHT: u32 = 160;

struct GameData {
    levels: Vec<Level>,
    ground_data: Vec<file::ground::Content>,
    tileset: Vec<file::tileset::Content>,
}

fn read_levels(file_name: &str) -> Result<Vec<Level>> {
    let path = Path::new(file_name);

    let compressed_level_data = fs::read(path.as_os_str())
        .with_context(|| format!("failed to load read '{}'", file_name))?;

    let decompressed_level_sections = file::encoding::datfile::parse(&compressed_level_data)?;
    let mut levels: Vec<Level> = Vec::new();

    for section in decompressed_level_sections.sections.iter() {
        let level = Level::decode(&section.data)?;
        levels.push(level);
    }

    Ok(levels)
}

fn dump_level(level: &Level) -> () {
    println!("{}", level);
    println!();
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

            if tile.remove_terrain && !tile.do_not_overwrite_exiting {
                if !bitmap.transparency[src_index] {
                    background_data[dest_index] = 255
                }
            } else if tile.do_not_overwrite_exiting {
                if background_data[dest_index] == 255 && !bitmap.transparency[src_index] {
                    background_data[dest_index] = bitmap.data[src_index];
                }
            } else {
                if !bitmap.transparency[src_index] {
                    background_data[dest_index] = bitmap.data[src_index];
                }
            }
        }
    }
}

fn compose_level(
    data: &GameData,
    index: usize,
    background_texture: &mut sdl2::render::Texture,
) -> Result<()> {
    let level = data.levels.get(index).context("invalid level ID")?;
    let tiles = data
        .tileset
        .get(level.graphics_set as usize)
        .context("invalid tile set")?;

    let mut background_data: Vec<u8> = vec![255; LEVEL_WIDTH as usize * LEVEL_HEIGHT as usize];

    for tile in &level.terrain_tiles {
        let bitmap_optional = tiles.tiles.get(tile.id as usize).and_then(|x| x.as_ref());

        match bitmap_optional {
            None => continue,
            Some(bitmap) => compose_tile_onto_background(tile, bitmap, &mut background_data),
        }
    }

    let pixel_format = create_pixel_format()?;

    let palette = data
        .ground_data
        .get(level.graphics_set as usize)
        .context("invalid tileset index")?
        .palettes
        .custom
        .map(|(r, g, b)| Color::RGBA(r as u8, g as u8, b as u8, 0xff).to_u32(&pixel_format));

    let mut texture_data = vec![0u32; LEVEL_WIDTH as usize * LEVEL_HEIGHT as usize];

    for x in 0..LEVEL_WIDTH {
        for y in 0..LEVEL_HEIGHT {
            let index = y * LEVEL_WIDTH + x;
            texture_data[index as usize] = if background_data[index as usize] == 255 {
                0
            } else {
                palette[background_data[index as usize] as usize]
            }
        }
    }

    let texture_data_8: &[u8];
    unsafe {
        let (_, x, _) = texture_data.align_to();
        assert_eq!(x.len(), 4 * texture_data.len());

        texture_data_8 = x;
    }

    background_texture.update(
        Rect::new(0, 0, LEVEL_WIDTH, LEVEL_HEIGHT),
        texture_data_8,
        4 * LEVEL_WIDTH as usize,
    )?;

    Ok(())
}

fn render(
    x: u32,
    zoom: u32,
    background: &sdl2::render::Texture,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
) -> Result<()> {
    canvas.clear();

    canvas
        .copy(
            &background,
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

fn cap_x(x: i32, zoom: u32) -> u32 {
    max(
        min(x as i32 + 10, LEVEL_WIDTH as i32 - 320 * 4 / zoom as i32) as i32 - 10,
        0,
    ) as u32
}

fn display_levels<'a>(data: &GameData) -> Result<()> {
    let sdl_context = sdl2::init().map_err(|s| anyhow!(s))?;
    let sdl_video = sdl_context.video().map_err(|s| anyhow!(s))?;
    let mut event_pump = sdl_context.event_pump().map_err(|s| anyhow!(s))?;
    let window = create_window(&sdl_video)?;
    let mut canvas = window.into_canvas().accelerated().present_vsync().build()?;
    let texture_creator = canvas.texture_creator();

    let mut background = texture_creator.create_texture_target(
        PixelFormatEnum::RGBA8888,
        LEVEL_WIDTH,
        LEVEL_HEIGHT,
    )?;

    let mut running = true;
    let mut i_level = 0;
    let mut zoom = 4;
    let mut x: u32 = cap_x(data.levels[i_level].start_x as i32, zoom);

    let mut left = false;
    let mut right = false;

    compose_level(data, i_level, &mut background)?;
    render(x, zoom, &background, &mut canvas)?;
    dump_level(&data.levels[i_level]);

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
                        x = cap_x(data.levels[i_level].start_x as i32, zoom);

                        dump_level(&data.levels[i_level]);
                        compose_level(data, i_level, &mut background)?;
                        render(x, zoom, &background, &mut canvas)?;
                    }
                    Keycode::Down => {
                        i_level = ((i_level + data.levels.len()) - 1) % data.levels.len();
                        x = cap_x(data.levels[i_level].start_x as i32, zoom);

                        dump_level(&data.levels[i_level]);
                        compose_level(data, i_level, &mut background)?;
                        render(x, zoom, &background, &mut canvas)?;
                    }
                    Keycode::Plus => {
                        let old_zoom = zoom;

                        zoom = match zoom {
                            1 => 2,
                            2 => 4,
                            _ => zoom,
                        };

                        x = cap_x(
                            (x + 320 * 2 / old_zoom) as i32 - (320 * 2 / zoom) as i32,
                            zoom,
                        );

                        render(x, zoom, &background, &mut canvas)?;
                    }

                    Keycode::Minus => {
                        let old_zoom = zoom;

                        zoom = match zoom {
                            4 => 2,
                            2 => 1,
                            _ => zoom,
                        };

                        x = cap_x(
                            (x + 320 * 2 / old_zoom) as i32 - (320 * 2 / zoom) as i32,
                            zoom,
                        );

                        render(x, zoom, &background, &mut canvas)?;
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
            x = cap_x(x as i32 - 10, zoom);
            render(x, zoom, &background, &mut canvas)?;
        } else if right {
            x = cap_x(x as i32 + 10, zoom);
            render(x, zoom, &background, &mut canvas)?;
        }

        sleep(Duration::from_millis(20));
    }

    Ok(())
}

pub fn main(data_path: &Path) -> Result<()> {
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

    display_levels(&GameData {
        levels,
        ground_data,
        tileset,
    })?;

    Ok(())
}
