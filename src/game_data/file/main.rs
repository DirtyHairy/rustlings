use anyhow::{bail, Result};
use std::{convert::TryInto, fs, path::Path};

use super::encoding::datfile;
use super::sprite_helper::sprite_read_planar;

use crate::game_data::{Sprite, TransparencyEncoding, NUM_LEMMING_SPRITES};

const LEMMING_SPRITES: [(usize, usize, usize, usize); NUM_LEMMING_SPRITES] = [
    (8, 16, 10, 2),
    (1, 16, 10, 2),
    (8, 16, 10, 2),
    (1, 16, 10, 2),
    (16, 16, 14, 3),
    (8, 16, 12, 2),
    (8, 16, 12, 2),
    (16, 16, 10, 2),
    (8, 16, 12, 2),
    (8, 16, 12, 2),
    (16, 16, 13, 3),
    (16, 16, 13, 3),
    (32, 16, 10, 3),
    (32, 16, 10, 3),
    (24, 16, 13, 3),
    (24, 16, 13, 3),
    (4, 16, 10, 2),
    (4, 16, 10, 2),
    (4, 16, 16, 3),
    (4, 16, 16, 3),
    (4, 16, 16, 3),
    (4, 16, 16, 3),
    (16, 16, 10, 2),
    (8, 16, 13, 2),
    (14, 16, 14, 4),
    (16, 16, 10, 2),
    (8, 16, 10, 2),
    (8, 16, 10, 2),
    (16, 16, 10, 2),
    (1, 32, 32, 3),
];

pub struct Content {
    pub lemming_sprites: [Sprite; NUM_LEMMING_SPRITES],
}

pub fn read_main(path: &Path) -> Result<Content> {
    println!("reading main.dat\n");
    let maindata = fs::read(path.join("main.dat").as_os_str())?;

    let datfile::Content { sections } = datfile::parse(&maindata)?;
    if sections.len() < 1 {
        bail!("invalid main.dat");
    }

    let mut lemming_sprites: Vec<Sprite> = Vec::new();
    let mut offset = 0;

    for (frame_count, width, height, bpp) in LEMMING_SPRITES {
        lemming_sprites.push(sprite_read_planar(
            frame_count,
            width,
            height,
            bpp,
            &sections[0].data,
            &mut offset,
            (width * height * bpp) / 8,
            TransparencyEncoding::Black,
        )?);
    }

    Ok(Content {
        lemming_sprites: lemming_sprites
            .try_into()
            .map_err(|_| ())
            .expect("internal error"),
    })
}
