use super::util::{create_pixel_format, create_window, read_ground};
use crate::{
    file::{self, tileset},
    level::Level,
};

use anyhow::{anyhow, Context, Result};
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
};
use std::{fs, path::Path, thread::sleep, time::Duration};

use crate::sdl_display::SDLSprite;

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

fn dump_levels(levels: &Vec<Level>, verbose: bool) -> () {
    for (index, level) in levels.iter().enumerate() {
        println!("Level {}:", index);
        println!("{}", level);

        if verbose {
            println!("");

            for (index, tile) in level.terrain_tiles.iter().enumerate() {
                println!("tile {}:\n{}\n", index, tile);
            }
        }
    }
}

fn render_level(
    data: &GameData,
    sprites: [Vec<Option<SDLSprite>>; 5],
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    index: usize,
) -> Result<()> {
    let level = data.levels.get(index).context("invalid level ID")?;

    for tile in &level.terrain_tiles {
        let sprite_optional = sprites[level.graphics_set as usize]
            .get(tile.id as usize)
            .and_then(|x| x.as_ref());

        match sprite_optional {
            None => continue,
            Some(sprite) => sprite.blit(canvas, tile.x, tile.y, 0, 1, tile.flip_y)?,
        }
    }

    Ok(())
}

fn display_levels<'a>(data: &GameData) -> Result<()> {
    let sdl_context = sdl2::init().map_err(|s| anyhow!(s))?;
    let sdl_video = sdl_context.video().map_err(|s| anyhow!(s))?;
    let mut event_pump = sdl_context.event_pump().map_err(|s| anyhow!(s))?;
    let window = create_window(&sdl_video)?;

    let mut canvas = window.into_canvas().accelerated().present_vsync().build()?;
    let texture_creator = canvas.texture_creator();
    let pixel_format = create_pixel_format()?;

    let mut background_texture =
        texture_creator.create_texture_target(PixelFormatEnum::RGBA8888, 1200, 160)?;

    let mut sprites: [Vec<Option<SDLSprite>>; 5] = [(); 5].map(|()| Vec::new());
    for (index, tileset) in data.tileset.iter().enumerate() {
        let palette = data
            .ground_data
            .get(index)
            .context("invalid tileset index")?
            .palettes
            .custom
            .map(|(r, g, b)| Color::RGBA(r as u8, g as u8, b as u8, 0xff).to_u32(&pixel_format));

        for bitmap_option in &tileset.tiles {
            sprites[index].push(
                bitmap_option.as_ref().and_then(|bitmap| {
                    SDLSprite::from_bitmap(bitmap, &palette, &texture_creator).ok()
                }),
            )
        }
    }

    canvas.with_texture_canvas(&mut background_texture, |texture_canvas| {
        texture_canvas.clear();

        if let Err(e) = render_level(data, sprites, texture_canvas, 0) {
            println!("failed to render background: {}", e);
        }
    })?;

    canvas.clear();

    canvas
        .copy(
            &background_texture,
            Rect::new(0, 0, 1200, 160),
            Rect::new(0, 80, 1200, 160),
        )
        .map_err(|s| anyhow!(s))?;

    canvas.present();

    let mut running = true;

    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => running = false,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => match code {
                    Keycode::Escape => running = false,
                    _ => (),
                },
                _ => (),
            }
        }

        sleep(Duration::from_millis(20));
    }

    Ok(())
}

pub fn main(level_file_name: &str, data_path: &Path, verbose: bool) -> Result<()> {
    let levels = read_levels(level_file_name)?;
    let (ground_data, tileset) = read_ground(data_path)?;

    dump_levels(&levels, verbose);
    display_levels(&GameData {
        levels,
        ground_data,
        tileset,
    })?;

    Ok(())
}
