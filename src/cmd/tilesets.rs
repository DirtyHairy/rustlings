use std::{cmp::max, path::Path, thread::sleep, time::Duration};

use super::util::{create_pixel_format, create_window, timestamp};
use crate::{file, sdl_display::SDLSprite};
use anyhow::{anyhow, Context, Result};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color};

fn display_tileset(
    ground_dats: &Vec<file::ground::Content>,
    tilesets: &Vec<file::tileset::Content>,
) -> Result<()> {
    let sdl_context = sdl2::init().map_err(|s| anyhow!(s))?;
    let sdl_video = sdl_context.video().map_err(|s| anyhow!(s))?;
    let mut event_pump = sdl_context.event_pump().map_err(|s| anyhow!(s))?;

    let window = create_window(&sdl_video)?;

    let mut canvas = window.into_canvas().accelerated().present_vsync().build()?;
    canvas.clear();

    let texture_creator = canvas.texture_creator();
    let pixel_format = create_pixel_format()?;

    let mut spritesets: Vec<Vec<SDLSprite>> = Vec::new();

    for i in 0..ground_dats.len() {
        let ground_dat = &ground_dats[i];
        let tileset = &tilesets[i];
        let mut sprites: Vec<SDLSprite> = Vec::new();

        let palette = ground_dat
            .palettes
            .custom
            .map(|(r, g, b)| Color::RGBA(r as u8, g as u8, b as u8, 0xff).to_u32(&pixel_format));

        for object_sprite in &tileset.object_sprites {
            object_sprite
                .as_ref()
                .and_then(|sprite| SDLSprite::from_sprite(&sprite, &palette, &texture_creator).ok())
                .map(|sdl_sprite| sprites.push(sdl_sprite));
        }

        for tile in &tileset.tiles {
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

        if now - last_draw > 1000 / 10 {
            let mut x: i32 = 0;
            let mut y: i32 = 1;
            let mut height: i32 = 0;

            canvas.clear();

            for sprite in &spritesets[ispriteset] {
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
    let mut ground: Vec<file::ground::Content> = Vec::new();
    let mut tileset: Vec<file::tileset::Content> = Vec::new();

    for i in 0..5 {
        let ground_dat =
            file::ground::read(path, i).context(format!("failed to read ground data set {}", i))?;

        tileset.push(
            file::tileset::read(path, i, &ground_dat)
                .context(format!("failed to read ground data set {}", i))?,
        );

        ground.push(ground_dat);
    }

    display_tileset(&ground, &tileset)?;

    Ok(())
}
