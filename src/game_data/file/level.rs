use std::{fs, path::Path};

use anyhow::{bail, Context, Result};

use crate::game_data::{Level, Object, TerrainTile, NUM_SKILLS};

use super::encoding::datfile;

pub fn read_level_file(path: &Path, index: usize) -> Result<Vec<Level>> {
    let filename = format!("level00{}.dat", index);
    println!("reading {}", &filename);

    let compressed_level_data = fs::read(path.join(filename).as_os_str())?;

    let decompressed_level_sections = datfile::parse(&compressed_level_data)?;
    let mut levels: Vec<Level> = Vec::new();

    for section in decompressed_level_sections.sections.iter() {
        let level = decode_level(&section.data)?;
        levels.push(level);
    }

    Ok(levels)
}

fn decode_level(data: &[u8]) -> Result<Level> {
    if data.len() != 2048 {
        bail!("not a level: invalid length");
    }

    let mut skills = [0 as u32; NUM_SKILLS];
    for i in 0..NUM_SKILLS {
        skills[i] = read16(data, 0x08 + 2 * i)? as u32;
    }

    let mut terrain_tiles: Vec<TerrainTile> = Vec::new();
    for i in 0..400 {
        if let Some(tile) = read_terrain_tile(data, i)? {
            terrain_tiles.push(tile);
        }
    }

    let mut objects: Vec<Object> = Vec::new();
    for i in 0..32 {
        if let Some(object) = read_object(data, i)? {
            objects.push(object);
        }
    }

    Ok(Level {
        release_rate: read16(data, 0)? as u32,
        released: read16(data, 0x02)? as u32,
        required: read16(data, 0x04)? as u32,
        time_limit: read16(data, 0x06)? as u32,
        start_x: read16(data, 0x18)? as u32,
        graphics_set: read16(data, 0x1a)? as u32,
        skills,
        extended_graphics_set: read16(data, 0x1c)? as u32,
        name: read_name(data)?,
        terrain_tiles,
        objects,
    })
}

fn read8(data: &[u8], offset: usize) -> Result<u8> {
    Ok(*data.get(offset).context("invalid level data")? as u8)
}

fn read16(data: &[u8], offset: usize) -> Result<u16> {
    Ok(((read8(data, offset)? as u16) << 8) | read8(data, offset + 1)? as u16)
}

fn read_name(data: &[u8]) -> Result<String> {
    let mut name = String::new();

    for i in 0..32 {
        let charcode = read8(data, 0x07e0 + i)?;
        name.push(char::from_u32(charcode as u32).context("invalid level name")?);
    }

    Ok(String::from(name.trim()))
}

fn read_terrain_tile(data: &[u8], index: usize) -> Result<Option<TerrainTile>> {
    if index >= 400 {
        bail!("invalid terrain index");
    }

    let x_and_flags = read16(data, 0x120 + 4 * index)?;
    if x_and_flags == 0xffff {
        return Ok(Option::None);
    }
    let flags = x_and_flags >> 12;

    let y = read16(data, 0x122 + 4 * index)? as i32;

    Ok(Option::Some(TerrainTile {
        x: (x_and_flags & 0x0fff) as i32 - 16,
        y: ((y << 16) >> 23) - 4,
        id: (y & 0x3f) as u32,
        do_not_overwrite: (flags & 0x08) != 0,
        flip_y: (flags & 0x04) != 0,
        remove_terrain: (flags & 0x02) != 0,
    }))
}

fn read_object(data: &[u8], index: usize) -> Result<Option<Object>> {
    if index >= 32 {
        bail!("invalid object index");
    }

    let x = read16(data, 0x20 + 8 * index)? as i16;
    let y = read16(data, 0x22 + 8 * index)? as i16;
    let id = read16(data, 0x24 + 8 * index)? as u32;
    let flags = read8(data, 0x26 + 8 * index)?;
    let flip = read8(data, 0x27 + 8 * index)?;

    if x == 0 && y == 0 && id == 0 && flags == 0 && flip == 0 {
        return Ok(Option::None);
    }

    Ok(Option::Some(Object {
        x: (x as i32) - 16,
        y: y as i32,
        id,
        do_not_overwrite: (flags & 0x80) != 0,
        draw_only_over_terrain: (flags & 0x40) != 0,
        flip_y: (flip & 0x80) != 0,
    }))
}
