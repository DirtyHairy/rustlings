use std::{cmp::max, path::Path, thread::sleep, time::Duration};

use anyhow::{Result, anyhow};
use rustlings::game_data::{GameData, read_game_data};
use rustlings::sdl_rendering::SDLSprite;
use sdl3::{event::Event, keyboard::Keycode};

use crate::cmd::util::{create_window, timestamp};

fn display_tileset(game_data: &GameData) -> Result<()> {
    let sdl_context = sdl3::init().map_err(|s| anyhow!(s))?;
    sdl3::hint::set("SDL_RENDER_VSYNC", "1");
    sdl3::hint::set("SDL_FRAMEBUFFER_ACCELERATION", "1");

    let sdl_video = sdl_context.video().map_err(|s| anyhow!(s))?;
    let mut event_pump = sdl_context.event_pump().map_err(|s| anyhow!(s))?;

    let window = create_window(&sdl_video, false)?;

    let mut canvas = window.into_canvas();
    canvas.clear();

    let texture_creator = canvas.texture_creator();

    let mut spritesets: Vec<Vec<SDLSprite>> = Vec::new();

    for i in 0..game_data.tilesets.len() {
        let mut sprites: Vec<SDLSprite> = Vec::new();
        let palette = &game_data.tilesets[i].palettes.custom;

        for object_sprite in &game_data.tilesets[i].object_sprites {
            object_sprite
                .as_ref()
                .and_then(|sprite| SDLSprite::from_sprite(&sprite, palette, &texture_creator).ok())
                .map(|sdl_sprite| sprites.push(sdl_sprite));
        }

        for tile in &game_data.tilesets[i].tiles {
            tile.as_ref()
                .and_then(|bitmap| SDLSprite::from_bitmap(&bitmap, &palette, &texture_creator).ok())
                .map(|sdl_sprite| sprites.push(sdl_sprite));
        }

        spritesets.push(sprites);
    }

    let mut running = true;
    let mut iframe = 0;
    let mut last_draw: u32 = 0;
    let mut ispriteset: usize = 0;

    while running {
        let now = timestamp();

        if now - last_draw > 1000 / 15 {
            let mut x: i32 = 0;
            let mut y: i32 = 1;
            let mut height: i32 = 0;

            canvas.clear();

            for sprite in spritesets[ispriteset].iter_mut() {
                if x + sprite.width as i32 > 1200 {
                    x = 0;
                    y = y + 2 * (height + 1);
                }

                sprite.blit(&mut canvas, x, y, iframe, 2, false)?;

                x = x + (sprite.width as i32 + 1) * 2;
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
                    keycode: Some(code),
                    ..
                } => match code {
                    Keycode::Escape => running = false,
                    Keycode::Left => {
                        ispriteset = ((ispriteset + spritesets.len()) - 1) % spritesets.len()
                    }
                    Keycode::Right => ispriteset = (ispriteset + 1) % spritesets.len(),
                    _ => (),
                },
                _ => (),
            }
        }

        sleep(Duration::from_millis(1));
    }

    Ok(())
}

pub fn main(path: &Path) -> Result<()> {
    let game_data = read_game_data(path)?;

    display_tileset(&game_data)
}
