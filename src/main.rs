mod definitions;
mod file;
mod sdl_display;

use anyhow::*;
use clap::{App, Arg, SubCommand};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::cmp::max;
use std::convert::{TryFrom, TryInto};

use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::definitions::STATIC_PALETTE;
use crate::file::sprite;
use crate::sdl_display::SDLSprite;
use file::sprite::Sprite;

fn timestamp() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32
}

fn create_window(sdl_video: &sdl2::VideoSubsystem) -> Result<sdl2::video::Window> {
    sdl_video
        .window("Rustlings", 1280, 800)
        .position_centered()
        .build()
        .map_err(|e| Error::from(e))
}

fn create_pixel_format() -> Result<sdl2::pixels::PixelFormat> {
    sdl2::pixels::PixelFormat::try_from(sdl2::pixels::PixelFormatEnum::RGBA8888)
        .map_err(|s| anyhow!(s))
}

fn display_sprites(sprites: Vec<Sprite>) -> Result<()> {
    let sdl_context = sdl2::init().map_err(|s| anyhow!(s))?;
    let sdl_video = sdl_context.video().map_err(|s| anyhow!(s))?;
    let mut event_pump = sdl_context.event_pump().map_err(|s| anyhow!(s))?;

    let window = create_window(&sdl_video)?;

    let mut canvas = window.into_canvas().accelerated().present_vsync().build()?;
    canvas.clear();

    let texture_creator = canvas.texture_creator();
    let pixel_format = create_pixel_format()?;

    let palette = STATIC_PALETTE
        .map(|(r, g, b)| Color::RGBA(r as u8, g as u8, b as u8, 0xff).to_u32(&pixel_format));

    let sdl_sprites: Vec<SDLSprite> = sprites
        .iter()
        .map(|s| SDLSprite::from_sprite(s, &palette, &texture_creator))
        .filter(|x| x.is_ok())
        .map(|x| x.expect(""))
        .collect();

    let mut running = true;
    let mut iframe = 0;
    let mut last_draw: u32 = 0;

    while running {
        let now = timestamp();

        if now - last_draw > 1000 / 10 {
            for (isprite, sprite) in sdl_sprites.iter().enumerate() {
                sprite.blit(
                    &mut canvas,
                    (isprite % 10 * 32 * 4) as i32,
                    (isprite / 10 * 32 * 4) as i32,
                    iframe,
                    4,
                )?;
            }

            canvas.present();

            iframe += 1;
            last_draw = now;
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => running = false,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => running = false,
                _ => (),
            }
        }

        sleep(Duration::from_millis(1));
    }

    Ok(())
}

fn display_tileset(
    ground_dat: &file::ground::Content,
    tileset: &file::tileset::Content,
) -> Result<()> {
    let sdl_context = sdl2::init().map_err(|s| anyhow!(s))?;
    let sdl_video = sdl_context.video().map_err(|s| anyhow!(s))?;
    let mut event_pump = sdl_context.event_pump().map_err(|s| anyhow!(s))?;

    let window = create_window(&sdl_video)?;

    let mut canvas = window.into_canvas().accelerated().present_vsync().build()?;
    canvas.clear();

    let texture_creator = canvas.texture_creator();
    let pixel_format = create_pixel_format()?;

    let palette = ground_dat
        .palettes
        .standard
        .map(|(r, g, b)| Color::RGBA(r as u8, g as u8, b as u8, 0xff).to_u32(&pixel_format));

    let mut sprites: Vec<SDLSprite> = Vec::new();

    for object_sprite in &tileset.object_sprites {
        object_sprite
            .as_ref()
            .and_then(|sprite| SDLSprite::from_sprite(&sprite, &palette, &texture_creator).ok())
            .map(|sdl_sprite| sprites.push(sdl_sprite));
    }

    let mut running = true;
    let mut iframe = 0;
    let mut last_draw: u32 = 0;

    while running {
        let now = timestamp();

        if now - last_draw > 1000 / 10 {
            let mut x: i32 = 0;
            let mut y: i32 = 0;
            let mut height: i32 = 0;

            for sprite in &sprites {
                if x + sprite.width as i32 > 1200 {
                    x = 0;
                    y = y + 4 * (height + 1);
                }

                sprite.blit(&mut canvas, x, y, iframe, 4)?;

                x = x + (sprite.width as i32 + 1) * 4;
                height = max(height as usize, sprite.height + 1) as i32;
            }

            canvas.present();

            iframe += 1;
            last_draw = now;
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => running = false,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => running = false,
                _ => (),
            }
        }

        sleep(Duration::from_millis(1));
    }

    Ok(())
}

fn main_sprites(path: &Path) {
    let main_dat = file::main::read(path).expect("failed to parse main.dat");

    if let Err(msg) = display_sprites(main_dat.lemming_sprites.to_vec()) {
        println!("SDL failed: {}", msg);
    }
}

fn main_tilesets(path: &Path) {
    let ground0 = file::ground::read(path, 0).expect("failed to read ground0o.dat");
    let tileset0 = file::tileset::read(path, 0, &ground0).expect("failed to read tileset bitmaps");

    println!("objects\n===\n");
    for object_info in &ground0.object_info {
        println!("{}\n", object_info);
    }

    println!("terrain\n===\n");
    for terrain_info in &ground0.terrain_info {
        println!("{}\n", terrain_info);
    }

    println!("palettes\n===\n{}", ground0.palettes);

    display_tileset(&ground0, &tileset0).expect("SDL failed");
}

fn main() -> Result<()> {
    let mut app = App::new("rustlings")
        .arg(
            Arg::with_name("PATH")
                .required(true)
                .help("path to .dat files")
                .index(1),
        )
        .subcommand(SubCommand::with_name("sprites").about("display lemming sprites"))
        .subcommand(SubCommand::with_name("tilesets").about("display tilesets"));

    let matches = app.clone().get_matches_safe()?;

    let path = Path::new(matches.value_of("PATH").expect("internal"));

    if matches.subcommand_matches("sprites").is_some() {
        main_sprites(path);
    } else if matches.subcommand_matches("tilesets").is_some() {
        main_tilesets(path);
    } else {
        app.print_help()?;

        println!();
    }

    return Ok(());
}
