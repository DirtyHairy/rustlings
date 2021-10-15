mod bitstream;
mod datfile;
mod sprites;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fs;

fn display_lemming(lemming: &sprites::Sprite) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let sdl_video = sdl_context.video()?;
    let mut event_pump = sdl_context.event_pump()?;

    let window = sdl_video
        .window("Rustlings", 640, 480)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();

    let pixel_format =
        sdl2::pixels::PixelFormat::try_from(sdl2::pixels::PixelFormatEnum::RGBA8888)?;

    let mut texture = texture_creator
        .create_texture(
            sdl2::pixels::PixelFormatEnum::RGBA8888,
            sdl2::render::TextureAccess::Static,
            lemming.width as u32,
            lemming.height as u32,
        )
        .map_err(|e| e.to_string())?;

    let palette = ([
        (0, 0, 0),
        (64, 64, 224),
        (0, 176, 0),
        (240, 208, 208),
        (176, 176, 0),
        (240, 32, 32),
        (128, 128, 128),
    ] as [(u8, u8, u8); 7])
        .map(|(r, g, b)| Color::RGBA(r, g, b, 0xff).to_u32(&pixel_format));

    let mut bitmap_data = vec![0u32; lemming.width * lemming.height];
    for x in 0..lemming.width {
        for y in 0..lemming.height {
            bitmap_data[(y * lemming.width) + x] =
                palette[lemming.frames[0].data[(y * lemming.width) + x] as usize];
        }
    }

    let data8: &[u8];
    unsafe {
        let (_, x, _) = bitmap_data.align_to();
        assert_eq!(x.len(), 4 * bitmap_data.len());

        data8 = x;
    }
    texture
        .update(None, data8, 4 * lemming.width)
        .map_err(|e| e.to_string())?;

    canvas.clear();
    canvas.copy(
        &texture,
        None,
        Some(Rect::new(
            0,
            0,
            (lemming.width * 4) as u32,
            (lemming.height * 4) as u32,
        )),
    )?;
    canvas.present();

    let mut running = true;
    while running {
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

    let mut maybe_walking_lemming: Option<sprites::Sprite> = None;
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
            maybe_walking_lemming =
                sprites::Sprite::read_planar(8, 16, 10, 2, &decompressed_section).ok();
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

    let walking_lemming = maybe_walking_lemming.expect("sprite not loaded");

    if let Err(msg) = display_lemming(&walking_lemming) {
        println!("SDL failed: {}", msg);
    }
}
