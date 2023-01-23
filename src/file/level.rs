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
    pub do_not_overwrite: bool,
    pub flip_y: bool,
    pub remove_terrain: bool,
}

pub struct Object {
    pub x: i32,
    pub y: i32,
    pub id: u32,
    pub do_not_overwrite: bool,
    pub flip_y: bool,
    pub draw_only_over_terrain: bool,
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
    pub objects: Vec<Object>,
}

pub trait LevelStructure {
    fn get_id(&self) -> u32;
    fn get_x(&self) -> i32;
    fn get_y(&self) -> i32;
}

impl LevelStructure for Object {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_x(&self) -> i32 {
        self.x
    }

    fn get_y(&self) -> i32 {
        self.y
    }
}

impl LevelStructure for TerrainTile {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_x(&self) -> i32 {
        self.x
    }

    fn get_y(&self) -> i32 {
        self.y
    }
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

    let y0 = read8(data, 0x122 + 4 * index)? as u32;
    let y1 = read8(data, 0x123 + 4 * index)? as u32;

    Ok(Option::Some(TerrainTile {
        x: (x_and_flags & 0x0fff) as i32 - 16,
        // sign extend 9 bits and shift zero
        y: (((((y0 << 1) | (y1 >> 7)) ^ 0x0100).wrapping_add(0xffffff00)) as i32) - 4,
        id: (y1) & 0x3f,
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

impl Level {
    pub fn decode(data: &[u8]) -> Result<Level> {
        if data.len() != 2048 {
            bail!("not a level: invalid length");
        }

        let mut skills = [0 as u32; Skill::Digger as usize + 1];
        for i in 0..Skill::Digger as usize {
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
do_not_overwrite: {}
flip_y: {}
remove_terrain: {}"#,
            self.x, self.y, self.id, self.do_not_overwrite, self.flip_y, self.remove_terrain,
        )
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            r#"x: {}
y: {}
id: {}
do_not_overwrite: {}
flip_y: {}
draw_only_over_terrain: {}"#,
            self.x,
            self.y,
            self.id,
            self.do_not_overwrite,
            self.flip_y,
            self.draw_only_over_terrain,
        )
    }
}
