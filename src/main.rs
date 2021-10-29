mod bitstream;
mod datfile;
mod definitions;
mod files;
mod sdl_display;
mod sprites;

use anyhow::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::convert::TryFrom;

use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::sdl_display::SDLSprite;
use crate::sprites::Sprite;

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

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("usage: rustlings <path/to/datfiles>");
        return;
    }

    let path = Path::new(&args[1]);

    let maindata = files::main::parse(path).expect("failed to parse main.dat");

    if let Err(msg) = display_sprites(maindata.lemming_sprites.to_vec()) {
        println!("SDL failed: {}", msg);
    }
}
