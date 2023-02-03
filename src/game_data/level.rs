use std::fmt;

use super::skill::{NUM_SKILLS, SKILLS};

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
pub struct Level {
    pub release_rate: u32,
    pub released: u32,
    pub required: u32,
    pub time_limit: u32,
    pub skills: [u32; NUM_SKILLS],
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

        for skill in SKILLS {
            writeln!(
                f,
                "  {}: {}",
                skill.to_string(),
                self.skills[skill as usize]
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
