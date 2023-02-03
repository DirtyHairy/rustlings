use std::{fs, path::Path};

use anyhow::*;

use crate::game_data::{
    ObjectInfo, Palettes, TerrainInfo, OBJECTS_PER_TILESET, PALETTE_SIZE, TILES_PER_TILESET,
};

use super::palette::PALETTE_FIXED;

pub struct Content {
    pub object_info: [ObjectInfo; OBJECTS_PER_TILESET],
    pub terrain_info: [TerrainInfo; TILES_PER_TILESET],
    pub palettes: Palettes,
}

pub fn read_ground(path: &Path, index: usize) -> Result<Content> {
    let filename = format!("ground{}o.dat", index);
    println!("reading {}", &filename);

    let data = fs::read(path.join(&filename).as_os_str())?;

    let mut offset = 0;
    let mut object_info: [ObjectInfo; OBJECTS_PER_TILESET] =
        [(); OBJECTS_PER_TILESET].map(|_| ObjectInfo::default());

    for i in 0..16 {
        let (value, new_offset) = read_object_info(&data, offset)?;
        offset = new_offset;

        object_info[i] = value;
    }

    let mut terrain_info: [TerrainInfo; TILES_PER_TILESET] =
        [(); TILES_PER_TILESET].map(|_| TerrainInfo::default());

    for i in 0..64 {
        let (value, new_offset) = read_terrain_info(&data, offset)?;
        offset = new_offset;

        terrain_info[i] = value;
    }

    let (palettes, offset) = read_palettes(&data, offset)?;

    if offset != data.len() {
        bail!(
            "extra data left in {}: read {} bytes, but got {} bytes",
            &filename,
            offset,
            data.len()
        );
    }

    Ok(Content {
        object_info,
        terrain_info,
        palettes,
    })
}

fn read_object_info(buffer: &Vec<u8>, offset: usize) -> Result<(ObjectInfo, usize)> {
    let (animation_flags, offset) = read_word(buffer, offset)?;
    let (animation_start, offset) = read_byte(buffer, offset)?;
    let (animation_end, offset) = read_byte(buffer, offset)?;
    let (width, offset) = read_byte(buffer, offset)?;
    let (height, offset) = read_byte(buffer, offset)?;
    let (animation_frame_size, offset) = read_word(buffer, offset)?;
    let (mask_offset, offset) = read_word(buffer, offset)?;
    let offset = offset + 4;
    let (trigger_left, offset) = read_word(buffer, offset)?;
    let (trigger_top, offset) = read_word(buffer, offset)?;
    let (trigger_width, offset) = read_byte(buffer, offset)?;
    let (trigger_height, offset) = read_byte(buffer, offset)?;
    let (trigger_effect, offset) = read_byte(buffer, offset)?;
    let (frames_offset, offset) = read_word(buffer, offset)?;
    let (preview_frame_offset, offset) = read_word(buffer, offset)?;
    let offset = offset + 2;
    let (trap_sound_effect, offset) = read_byte(buffer, offset)?;

    Ok((
        ObjectInfo {
            animation_loops: (animation_flags & 0x01) > 0,
            animation_start,
            animation_end,
            width,
            height,
            animation_frame_size,
            mask_offset,
            trigger_left,
            trigger_top,
            trigger_width,
            trigger_height,
            trigger_effect,
            frames_offset,
            preview_frame_offset,
            trap_sound_effect,
        },
        offset,
    ))
}

fn read_terrain_info(buffer: &Vec<u8>, offset: usize) -> Result<(TerrainInfo, usize)> {
    let (width, offset) = read_byte(buffer, offset)?;
    let (height, offset) = read_byte(buffer, offset)?;
    let (image_offset, offset) = read_word(buffer, offset)?;
    let (mask_offset, offset) = read_word(buffer, offset)?;
    let offset = offset + 2;

    Ok((
        TerrainInfo {
            width,
            height,
            image_offset,
            mask_offset,
        },
        offset,
    ))
}

fn read_palette_entry(buffer: &Vec<u8>, offset: usize) -> Result<((usize, usize, usize), usize)> {
    let (r, offset) = read_byte(buffer, offset)?;
    let (g, offset) = read_byte(buffer, offset)?;
    let (b, offset) = read_byte(buffer, offset)?;

    Ok(((r << 2, g << 2, b << 2), offset))
}
fn read_palette(buffer: &Vec<u8>, offset: usize) -> Result<([(usize, usize, usize); 16], usize)> {
    let mut palette: [(usize, usize, usize); PALETTE_SIZE] = [(0, 0, 0); PALETTE_SIZE];
    let mut offset = offset;

    for i in 0..7 {
        palette[i] = PALETTE_FIXED[i];
    }

    for i in 8..16 {
        let (entry, new_offset) = read_palette_entry(buffer, offset)?;

        palette[i] = entry;
        offset = new_offset;
    }

    palette[7] = palette[8];

    Ok((palette, offset))
}

fn read_palettes(buffer: &Vec<u8>, offset: usize) -> Result<(Palettes, usize)> {
    let offset = offset + 24;
    let (custom, offset) = read_palette(buffer, offset)?;
    let (standard, offset) = read_palette(buffer, offset)?;
    let (preview, offset) = read_palette(buffer, offset)?;

    Ok((
        Palettes {
            custom,
            standard,
            preview,
        },
        offset,
    ))
}

fn read_byte(buffer: &Vec<u8>, offset: usize) -> Result<(usize, usize)> {
    Ok((
        *buffer
            .get(offset)
            .ok_or(anyhow!("offset {} out of bounds", offset))? as usize,
        offset + 1,
    ))
}

fn read_word(buffer: &Vec<u8>, offset: usize) -> Result<(usize, usize)> {
    Ok((
        ((read_byte(buffer, offset)?.0 as u16) | (read_byte(buffer, offset + 1)?.0 as u16) << 8)
            as usize,
        offset + 2,
    ))
}
