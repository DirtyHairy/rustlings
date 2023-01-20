use std::{convert::TryInto, fs, path::Path};

use anyhow::{anyhow, bail, Result};

use crate::file::{encoding::datfile, sprite::TransparencyEncoding};

use super::{
    ground,
    sprite::{Bitmap, Sprite},
};

pub struct Content {
    pub object_sprites: [Option<Sprite>; 16],
    pub tiles: [Option<Bitmap>; 64],
}

pub fn read(path: &Path, index: usize, ground_data: &ground::Content) -> Result<Content> {
    let filename = format!("vgagr{}.dat", index);
    let data = fs::read(path.join(&filename).as_os_str())?;

    println!("reading {}", &filename);

    let datfile::Content { sections } = datfile::parse(&data)?;
    if sections.len() != 2 {
        bail!("bad tileset {}", filename);
    }

    let object_data = &sections[1].data;
    let mut object_sprites: Vec<Option<Sprite>> = Vec::new();

    for object_info in &ground_data.object_info {
        if object_info.width * object_info.height == 0 {
            object_sprites.push(None);
            continue;
        }

        let mut offset = object_info.frames_offset;
        object_sprites.push(Some(Sprite::read_planar(
            object_info.animation_end,
            object_info.width,
            object_info.height,
            4,
            &object_data,
            &mut offset,
            object_info.animation_frame_size,
            TransparencyEncoding::PlanarOffset(object_info.mask_offset),
        )?));
    }

    let tileset_data = &sections[0].data;
    let mut tiles: Vec<Option<Bitmap>> = Vec::new();

    for terrain_info in &ground_data.terrain_info {
        if terrain_info.width * terrain_info.height == 0 {
            tiles.push(None);
            continue;
        }

        tiles.push(Some(Bitmap::read_planar(
            terrain_info.width,
            terrain_info.height,
            4,
            &tileset_data
                .get(terrain_info.image_offset..)
                .ok_or(anyhow!("tile data out of bounds"))?,
            TransparencyEncoding::PlanarAt(
                &tileset_data
                    .get(terrain_info.mask_offset..)
                    .ok_or(anyhow!("tile data out of bounds"))?,
            ),
        )?));
    }

    return Ok(Content {
        object_sprites: object_sprites
            .try_into()
            .map_err(|_| ())
            .expect("internal error"),
        tiles: tiles.try_into().map_err(|_| ()).expect("internal error"),
    });
}
