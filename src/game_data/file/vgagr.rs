use crate::game_data::file::encoding::datfile;
use crate::game_data::file::ground::{OBJECTS_PER_TILESET, ObjectInfo, TILES_PER_TILESET, TerrainInfo};
use crate::game_data::file::sprite::{Bitmap, Sprite, TransparencyEncoding};
use anyhow::{Result, anyhow, bail};
use std::{fs, path::Path};

pub struct Content {
    pub object_sprites: [Option<Sprite>; OBJECTS_PER_TILESET],
    pub tiles: [Option<Bitmap>; TILES_PER_TILESET],
}

pub fn read_vgagr(
    path: &Path,
    index: usize,
    object_info: &[ObjectInfo],
    terrain_info: &[TerrainInfo],
) -> Result<Content> {
    let filename = format!("vgagr{}.dat", index);
    println!("reading {}", &filename);

    let data = fs::read(path.join(&filename).as_os_str())?;

    let datfile::Content { sections } = datfile::parse(&data)?;
    if sections.len() != 2 {
        bail!("bad tileset {}", filename);
    }

    let object_data = &sections[1].data;

    const SPRITE_NONE: Option<Sprite> = None;
    let mut object_sprites: [Option<Sprite>; OBJECTS_PER_TILESET] =
        [SPRITE_NONE; OBJECTS_PER_TILESET];

    for i in 0..OBJECTS_PER_TILESET {
        let info = &object_info[i];

        if info.width * info.height == 0 {
            continue;
        }

        let mut offset = info.frames_offset;
        object_sprites[i] = Some(Sprite::read_planar(
            info.animation_end,
            info.width,
            info.height,
            4,
            &object_data,
            &mut offset,
            info.animation_frame_size,
            TransparencyEncoding::PlanarOffset(info.mask_offset),
        )?);
    }

    let tileset_data = &sections[0].data;

    const BITMAP_NONE: Option<Bitmap> = None;
    let mut tiles: [Option<Bitmap>; TILES_PER_TILESET] = [BITMAP_NONE; TILES_PER_TILESET];

    for i in 0..TILES_PER_TILESET {
        let info = &terrain_info[i];

        if info.width * info.height == 0 {
            continue;
        }

        tiles[i] = Some(Bitmap::read_planar(
            info.width,
            info.height,
            4,
            &tileset_data
                .get(info.image_offset..)
                .ok_or(anyhow!("tile data out of bounds"))?,
            TransparencyEncoding::PlanarAt(
                &tileset_data
                    .get(info.mask_offset..)
                    .ok_or(anyhow!("tile data out of bounds"))?,
            ),
        )?);
    }

    Ok(Content {
        object_sprites,
        tiles,
    })
}
