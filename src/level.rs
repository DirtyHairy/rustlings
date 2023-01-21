use std::{convert::TryFrom, fmt};

use anyhow::{bail, Context, Result};
use num_enum::TryFromPrimitive;

#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum Skill {
    Climber = 0,
    Floater,
    Bomber,
    Blocker,
    Builder,
    Basher,
    Miner,
    Digger,
}

pub struct TerrainTile {
    pub x: i32,
    pub y: i32,
    pub id: u32,
    pub do_not_overwrite_exiting: bool,
    pub flip_y: bool,
    pub remove_terrain: bool,
}

pub struct Level {
    pub release_rate: u32,
    pub released: u32,
    pub required: u32,
    pub time_limit: u32,
    pub skills: [u32; (Skill::Digger as usize) + 1],
    pub start_x: u32,
    pub graphics_set: u32,
    pub extended_graphics_set: u32,
    pub name: String,
    pub terrain_tiles: Vec<TerrainTile>,
}

fn read8(data: &Vec<u8>, offset: usize) -> Result<u32> {
    Ok(*data.get(offset).context("invalid level data")? as u32)
}

fn read16(data: &Vec<u8>, offset: usize) -> Result<u32> {
    Ok((read8(data, offset)? << 8) | read8(data, offset + 1)?)
}

fn read_name(data: &Vec<u8>) -> Result<String> {
    let mut name = String::new();

    for i in 0..32 {
        let charcode = read8(data, 0x07e0 + i)?;
        name.push(char::from_u32(charcode).context("invalid level name")?);
    }

    Ok(String::from(name.trim()))
}

fn read_terrain_tile(data: &Vec<u8>, index: usize) -> Result<Option<TerrainTile>> {
    if index >= 400 {
        bail!("invalid terrain index");
    }

    let x_and_flags = read16(data, 0x120 + 4 * index)?;
    if x_and_flags == 0xffff {
        return Ok(Option::None);
    }
    let flags = x_and_flags >> 12;

    let y0 = read8(data, 0x122 + 4 * index)?;
    let y1 = read8(data, 0x123 + 4 * index)?;

    Ok(Option::Some(TerrainTile {
        x: (x_and_flags & 0x0fff) as i32 - 16,
        y: (0x200 + ((y0 << 1) | (y1 >> 7)) as i32 - 0x1de) % 0x200 - 38,
        id: (y1) & 0x3f,
        do_not_overwrite_exiting: (flags & 0x08) != 0,
        flip_y: (flags & 0x04) != 0,
        remove_terrain: (flags & 0x02) != 0,
    }))
}

impl Level {
    pub fn decode(data: &Vec<u8>) -> Result<Level> {
        if data.len() != 2048 {
            bail!("not a level: invalid length");
        }

        let mut skills = [0 as u32; Skill::Digger as usize + 1];
        for i in 0..Skill::Digger as usize {
            skills[i] = read16(data, 0x08 + 2 * i)?;
        }

        let mut terrain_tiles = Vec::<TerrainTile>::new();
        for i in 0..400 as usize {
            let tile = read_terrain_tile(data, i)?;
            if tile.is_none() {
                continue;
            }

            terrain_tiles.push(tile.expect("unreachable"));
        }

        Ok(Level {
            release_rate: read16(data, 0)?,
            released: read16(data, 0x02)?,
            required: read16(data, 0x04)?,
            time_limit: read16(data, 0x06)?,
            start_x: read16(data, 0x18)?,
            graphics_set: read16(data, 0x1a)?,
            skills,
            extended_graphics_set: read16(data, 0x1c)?,
            name: read_name(data)?,
            terrain_tiles,
        })
    }
}

impl std::string::ToString for Skill {
    fn to_string(&self) -> String {
        let str = match self {
            Skill::Climber => "Climber",
            Skill::Floater => "Floater",
            Skill::Bomber => "Bomber",
            Skill::Blocker => "Blocker",
            Skill::Builder => "Builder",
            Skill::Basher => "Basher",
            Skill::Miner => "Miner",
            Skill::Digger => "Digger",
        };

        String::from(str)
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            r#"{}
*****
release_rate: {}
released: {}
required: {}
time_limit: {}
start_x: {}
graphics_set: {}
extended_graphics_set: {}
Skills"#,
            self.name,
            self.release_rate,
            self.released,
            self.required,
            self.time_limit,
            self.start_x,
            self.graphics_set,
            self.extended_graphics_set,
        )?;

        for i in 0..(Skill::Digger as usize) {
            writeln!(
                f,
                "  {}: {}",
                (Skill::try_from(i as u8).expect("unreachable")).to_string(),
                self.skills[i]
            )?;
        }

        Ok(())
    }
}

impl fmt::Display for TerrainTile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            r#"x: {}
y: {}
id: {}
overwrite_exiting: {}
flip_y: {}
remove_terrain: {}"#,
            self.x,
            self.y,
            self.id,
            self.do_not_overwrite_exiting,
            self.flip_y,
            self.remove_terrain,
        )
    }
}
