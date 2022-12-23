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

impl Level {
    pub fn decode(data: &Vec<u8>) -> Result<Level> {
        if data.len() != 2048 {
            bail!("not a level: invalid length");
        }

        let mut skills = [0 as u32; Skill::Digger as usize + 1];

        for i in 0..Skill::Digger as usize {
            skills[i] = read16(data, 0x08 + 2 * i)?;
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
Skills:"#,
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
