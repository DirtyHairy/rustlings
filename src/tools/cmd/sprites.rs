use std::{path::Path, thread::sleep, time::Duration};

use rustlings::game_data::{GameData, read_game_data};
use rustlings::sdl_rendering::SDLSprite;

use super::util;
use anyhow::{Result, anyhow};
use sdl3::{event::Event, keyboard::Keycode};

fn display_sprites(game_data: &GameData) -> Result<()> {
    let sdl_context = sdl3::init().map_err(|s| anyhow!(s))?;
    sdl3::hint::set("SDL_RENDER_VSYNC", "1");
    sdl3::hint::set("SDL_FRAMEBUFFER_ACCELERATION", "1");

    let sdl_video = sdl_context.video().map_err(|s| anyhow!(s))?;
    let mut event_pump = sdl_context.event_pump().map_err(|s| anyhow!(s))?;

    let window = util::create_window(&sdl_video, false)?;

    let mut canvas = window.into_canvas();
    canvas.clear();

    let texture_creator = canvas.texture_creator();

    let mut sdl_sprites: Vec<SDLSprite> = game_data
        .lemming_sprites
        .iter()
        .map(|s| SDLSprite::from_sprite(s, &game_data.static_palette, &texture_creator))
        .filter(|x| x.is_ok())
        .map(|x| x.expect(""))
        .collect();

    let mut running = true;
    let mut iframe = 0;
    let mut last_draw: u32 = 0;

    while running {
        let now = util::timestamp();

        canvas.clear();

        if now - last_draw > 1000 / 15 {
            for (isprite, sprite) in sdl_sprites.iter_mut().enumerate() {
                sprite.blit(
                    &mut canvas,
                    (isprite % 10 * 32 * 4) as i32,
                    (isprite / 10 * 32 * 4) as i32,
                    iframe,
                    4,
                    false,
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

pub fn main(path: &Path) -> Result<()> {
    let game_data = read_game_data(path)?;

    display_sprites(&game_data)?;

    Ok(())
}
