mod definitions;
mod file;
mod sdl_display;

use anyhow::*;
use clap::{App, Arg, SubCommand};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::convert::TryFrom;

use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::sdl_display::SDLSprite;
use file::sprite::Sprite;

fn timestamp() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32
}

fn display_sprites(sprites: Vec<Sprite>) -> Result<()> {
    let sdl_context = sdl2::init().map_err(|s| anyhow!(s))?;
    let sdl_video = sdl_context.video().map_err(|s| anyhow!(s))?;
    let mut event_pump = sdl_context.event_pump().map_err(|s| anyhow!(s))?;

    let window = sdl_video
        .window("Rustlings", 1280, 800)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().accelerated().present_vsync().build()?;

    canvas.clear();

    let texture_creator = canvas.texture_creator();

    let pixel_format = sdl2::pixels::PixelFormat::try_from(sdl2::pixels::PixelFormatEnum::RGBA8888)
        .map_err(|s| anyhow!(s))?;

    let palette: Vec<u32> = (vec![
        (0u8, 0u8, 0u8),
        (64, 64, 224),
        (0, 176, 0),
        (240, 208, 208),
        (176, 176, 0),
        (240, 32, 32),
        (128, 128, 128),
        (0u8, 0u8, 0u8),
        (64, 64, 224),
        (0, 176, 0),
        (240, 208, 208),
        (176, 176, 0),
        (240, 32, 32),
        (128, 128, 128),
        (0u8, 0u8, 0u8),
        (64, 64, 224),
    ] as Vec<(u8, u8, u8)>) /* */
        .iter()
        .map(|(r, g, b)| Color::RGBA(*r, *g, *b, 0xff).to_u32(&pixel_format))
        .collect();

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

fn main_sprites(path: &Path) {
    let main_dat = file::main::parse(path).expect("failed to parse main.dat");

    if let Err(msg) = display_sprites(main_dat.lemming_sprites.to_vec()) {
        println!("SDL failed: {}", msg);
    }
}

fn main_tilesets(path: &Path) {
    let ground0 = file::ground::parse(path, 0).expect("failed to read ground0o.dat");

    println!("objects\n===\n");
    for object_info in ground0.object_info {
        println!("{}\n", object_info);
    }

    println!("terrain\n===\n");
    for terrain_info in ground0.terrain_info {
        println!("{}\n", terrain_info);
    }

    println!("palettes\n===\n{}", ground0.palettes);
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
