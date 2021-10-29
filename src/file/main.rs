use anyhow::*;
use std::{convert::TryInto, fs, path::Path};

use super::sprite::Sprite;
use crate::definitions::LEMMING_SPRITES;
use crate::file::encoding::datfile;

pub struct Content {
    pub lemming_sprites: [Sprite; 30],
}

pub fn parse(path: &Path) -> Result<Content> {
    let maindata = fs::read(path.join("main.dat").as_os_str())?;

    println!("reading main.dat\n");

    let datfile::Content { sections } = datfile::parse(&maindata)?;
    if sections.len() < 1 {
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
            sections[0]
                .data
                .get(offset..)
                .ok_or(anyhow!("out of bounds reading lemming sprites"))?,
            &mut offset,
        )?);
    }

    return Ok(Content {
        lemming_sprites: lemming_sprites
            .try_into()
            .map_err(|_| ())
            .expect("internal error"),
    });
}
