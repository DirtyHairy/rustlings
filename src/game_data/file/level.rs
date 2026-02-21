use crate::game_data::file::encoding::datfile;
use crate::game_data::file::read::{read_byte, read_word_le};
use crate::game_data::skill::{NUM_SKILLS, SKILLS};
use anyhow::{Context, Result, bail};
use std::fmt;
use std::{fs, path::Path};

const ODDTABLE_ENTRIES: usize = 80;
const ODDTABLE_ENTRY_SIZE: usize = 0x38;
const ODDTABLE_FILENAME: &str = "oddtable.dat";

#[derive(Clone)]
pub struct TerrainTile {
    pub x: i32,
    pub y: i32,
    pub id: u32,
    pub do_not_overwrite: bool,
    pub flip_y: bool,
    pub remove_terrain: bool,
}

#[derive(Clone)]
pub struct Object {
    pub x: i32,
    pub y: i32,
    pub id: u32,
    pub do_not_overwrite: bool,
    pub flip_y: bool,
    pub draw_only_over_terrain: bool,
}

#[derive(Clone)]
pub struct LevelParamters {
    pub release_rate: u32,
    pub released: u32,
    pub required: u32,
    pub time_limit: u32,
    pub skills: [u32; NUM_SKILLS],
    pub name: String,
}

#[derive(Clone)]
pub struct Level {
    pub parameters: LevelParamters,
    pub start_x: u32,
    pub graphics_set: u32,
    pub extended_graphics_set: u32,
    pub terrain_tiles: Vec<TerrainTile>,
    pub objects: Vec<Object>,
}

#[allow(dead_code)]
pub trait LevelStructure {
    fn get_id(&self) -> u32;
    fn get_x(&self) -> i32;
    fn get_y(&self) -> i32;
}

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

pub fn read_oddtable(path: &Path) -> Result<Vec<LevelParamters>> {
    println!("reading {}", ODDTABLE_FILENAME);

    let oddtable_data = fs::read(path.join(ODDTABLE_FILENAME).as_os_str())?;

    if oddtable_data.len() != ODDTABLE_ENTRY_SIZE * ODDTABLE_ENTRIES {
        bail!("invalid {}", ODDTABLE_FILENAME);
    }

    let mut oddtable: Vec<LevelParamters> = Vec::with_capacity(ODDTABLE_ENTRIES);
    for i in 0..80 {
        oddtable.push(decode_oddtable_entry(
            &oddtable_data[i * ODDTABLE_ENTRY_SIZE..(i + 1) * ODDTABLE_ENTRY_SIZE],
        )?);
    }

    Ok(oddtable)
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
        parameters: LevelParamters {
            release_rate: read16(data, 0)? as u32,
            released: read16(data, 0x02)? as u32,
            required: read16(data, 0x04)? as u32,
            time_limit: read16(data, 0x06)? as u32,
            skills,
            name: read_name(data, 0x07e0)?,
        },
        start_x: read16(data, 0x18)? as u32,
        graphics_set: read16(data, 0x1a)? as u32,
        extended_graphics_set: read16(data, 0x1c)? as u32,
        terrain_tiles,
        objects,
    })
}

fn decode_oddtable_entry(data: &[u8]) -> Result<LevelParamters> {
    let mut skills = [0 as u32; NUM_SKILLS];
    for i in 0..NUM_SKILLS {
        skills[i] = read16(data, 0x08 + 2 * i)? as u32;
    }

    Ok(LevelParamters {
        release_rate: read16(data, 0)? as u32,
        released: read16(data, 0x02)? as u32,
        required: read16(data, 0x04)? as u32,
        time_limit: read16(data, 0x06)? as u32,
        skills,
        name: read_name(data, 0x18)?,
    })
}

fn read8(data: &[u8], offset: usize) -> Result<u8> {
    Ok(read_byte(data, offset)?.0)
}

fn read16(data: &[u8], offset: usize) -> Result<u16> {
    Ok(read_word_le(data, offset)?.0)
}

fn read_name(data: &[u8], offset: usize) -> Result<String> {
    let mut name = String::new();

    for i in 0..32 {
        let charcode = read8(data, offset + i)?;
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
            self.parameters.name,
            self.parameters.release_rate,
            self.parameters.released,
            self.parameters.required,
            self.parameters.time_limit,
            self.start_x,
            self.graphics_set,
            self.extended_graphics_set,
        )?;

        for skill in SKILLS {
            writeln!(
                f,
                "  {}: {}",
                skill.to_string(),
                self.parameters.skills[skill as usize]
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
