mod bitstream;
mod datfile;
mod sdl_display;
mod sprites;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fs;
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

fn display_sprites(sprites: Vec<&Sprite>) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let sdl_video = sdl_context.video()?;
    let mut event_pump = sdl_context.event_pump()?;

    let window = sdl_video
        .window("Rustlings", 1024, 768)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    canvas.clear();

    let texture_creator = canvas.texture_creator();

    let pixel_format =
        sdl2::pixels::PixelFormat::try_from(sdl2::pixels::PixelFormatEnum::RGBA8888)?;

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
                    (isprite % 8 * 32 * 4) as i32,
                    (isprite / 8 * 32 * 4) as i32,
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
        println!("usage: rustlings <main.dat>");
        return;
    }

    let maindata = fs::read(&args[1]).expect("give me main.dat");

    println!("read {} bytes\n", maindata.len());
    let mut offset = 0;

    let mut sprites: Vec<Option<Sprite>> = Vec::new();
    let mut i = 0;

    loop {
        let (header, o) = datfile::Header::read(&maindata, offset).expect("bad file");

        println!("found header:\n{}", header);

        let checksum = datfile::calculate_checksum(&header, &maindata, o).expect("bad file");
        if checksum == header.checksum {
            println!("checksum OK!")
        } else {
            println!(
                "checksum mismatch, expected {}, got {}",
                header.checksum, checksum
            )
        }

        let mut decompressed_section: Vec<u8> = Vec::with_capacity(header.decompressed_data_size);

        datfile::decompress_section(
            &mut bitstream::Bitstream::create(
                maindata[o..o + header.compressed_data_size - 10].to_vec(),
                header.num_bits_in_first_byte,
            ),
            &mut decompressed_section,
        );

        assert_eq!(decompressed_section.len(), header.decompressed_data_size);

        if i == 0 {
            let mut offset = 0;

            sprites.push(
                Sprite::read_planar(8, 16, 10, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(1, 16, 10, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(8, 16, 10, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(1, 16, 10, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(16, 16, 14, 3, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(8, 16, 12, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(8, 16, 12, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(16, 16, 10, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(8, 16, 12, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(8, 16, 12, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(16, 16, 13, 3, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(16, 16, 13, 3, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(32, 16, 10, 3, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(32, 16, 10, 3, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(24, 16, 13, 3, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(24, 16, 13, 3, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(4, 16, 10, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(4, 16, 10, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(4, 16, 16, 3, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(4, 16, 16, 3, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(4, 16, 16, 3, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(4, 16, 16, 3, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(16, 16, 10, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(8, 16, 13, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(14, 16, 14, 4, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(16, 16, 10, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(8, 16, 10, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(8, 16, 10, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(16, 16, 10, 2, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
            sprites.push(
                Sprite::read_planar(1, 32, 32, 3, &decompressed_section[offset..], &mut offset)
                    .ok(),
            );
        }

        offset = o + header.compressed_data_size - 10;

        println!();
        i += 1;

        match offset.cmp(&maindata.len()) {
            Ordering::Equal => break,
            Ordering::Greater => panic!("bad file"),
            Ordering::Less => continue,
        };
    }

    let sprites: Vec<&Sprite> = sprites
        .iter()
        .filter(|x| x.is_some())
        .map(|x| x.as_ref().expect(""))
        .collect();

    if let Err(msg) = display_sprites(sprites) {
        println!("SDL failed: {}", msg);
    }
}
