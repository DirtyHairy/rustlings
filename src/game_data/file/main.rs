use crate::game_data::Bitmap;

use crate::game_data::file::encoding::datfile;
use crate::game_data::file::sprite::{Sprite, TransparencyEncoding};
use anyhow::{Result, bail};
use std::{convert::TryInto, fs, path::Path};

pub const NUM_LEMMING_SPRITES: usize = 30;

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
    pub skill_panel: Bitmap,
}

pub fn read_main(path: &Path) -> Result<Content> {
    println!("reading main.dat\n");
    let maindata = fs::read(path.join("main.dat").as_os_str())?;

    let datfile::Content { sections } = datfile::parse(&maindata)?;
    if sections.len() < 3 {
        bail!("invalid main.dat");
    }

    let mut lemming_sprites: Vec<Sprite> = Vec::new();
    let mut offset = 0;

    for (frame_count, width, height, bpp) in LEMMING_SPRITES {
        lemming_sprites.push(Sprite::read_planar(
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
        skill_panel: Bitmap::read_planar(
            320,
            40,
            4,
            &sections[2].data,
            TransparencyEncoding::Black,
        )?,
    })
}
