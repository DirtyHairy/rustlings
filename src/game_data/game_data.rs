use anyhow::{Result, bail, format_err};

pub use crate::game_data::file::ground::{
    OBJECTS_PER_TILESET, ObjectInfo, Palettes, TILES_PER_TILESET, TerrainInfo,
};
pub use crate::game_data::file::level::{
    Level, LevelParamters, LevelStructure, Object, TerrainTile,
};
pub use crate::game_data::file::main::NUM_LEMMING_SPRITES;
use crate::game_data::file::palette::{LOWER_PALETTE_FIXED, UPPER_PALETTE_SKILL_PANEL};
pub use crate::game_data::file::palette::{PALETTE_SIZE, PaletteEntry};
pub use crate::game_data::file::sprite::{Bitmap, Sprite};

pub const LEVEL_WIDTH: usize = 1600;
pub const LEVEL_HEIGHT: usize = 160;
pub const VGASPEC_POSITION: usize = 304;

const LEVEL_TABLE: [u8; 120] = [
    0x93, 0x9b, 0x9d, 0x95, 0x97, 0x99, 0x9f, 0x0e, 0x16, 0x36, 0x46, 0x10, 0x1d, 0x20, 0x26, 0x2a,
    0x30, 0x48, 0x54, 0x68, 0x8a, 0x17, 0x44, 0x60, 0x62, 0x74, 0x4e, 0x64, 0x6c, 0x86, 0x01, 0x1e,
    0x24, 0x32, 0x34, 0x38, 0x3a, 0x50, 0x66, 0x78, 0x80, 0x82, 0x88, 0x05, 0x94, 0x98, 0x9a, 0x9c,
    0xa0, 0x07, 0x0b, 0x0d, 0x0f, 0x11, 0x13, 0x15, 0x19, 0x1b, 0x21, 0x1f, 0x25, 0x27, 0x29, 0x2b,
    0x2d, 0x2f, 0x31, 0x33, 0x35, 0x37, 0x39, 0x3b, 0x3d, 0x3f, 0x03, 0x41, 0x43, 0x45, 0x47, 0x49,
    0x4b, 0x4d, 0x4f, 0x51, 0x53, 0x55, 0x57, 0x59, 0x23, 0x6f, 0x5b, 0x5d, 0x5f, 0x61, 0x63, 0x65,
    0x67, 0x69, 0x6b, 0x6d, 0x70, 0x71, 0x73, 0x75, 0x77, 0x79, 0x7b, 0x7d, 0x7f, 0x96, 0x81, 0x09,
    0x83, 0x85, 0x87, 0x89, 0x8b, 0x8d, 0x8f, 0x91,
];

#[derive(Clone, Copy)]
pub enum DifficultyRating {
    Fun,
    Tricky,
    Taxing,
    Mayhem,
}

impl ToString for DifficultyRating {
    fn to_string(&self) -> String {
        match self {
            Self::Fun => "Fun".to_string(),
            Self::Tricky => "Tricky".to_string(),
            Self::Taxing => "Taxing".to_string(),
            Self::Mayhem => "Mayhem".to_string(),
        }
    }
}

impl From<usize> for DifficultyRating {
    fn from(value: usize) -> Self {
        match value % 4 {
            0 => Self::Fun,
            1 => Self::Tricky,
            2 => Self::Taxing,
            3 => Self::Mayhem,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub struct Image {
    pub palette: [PaletteEntry; PALETTE_SIZE],
    pub bitmap: Bitmap,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct TileSet {
    pub object_info: [ObjectInfo; OBJECTS_PER_TILESET],
    pub terrain_info: [TerrainInfo; TILES_PER_TILESET],
    pub palettes: Palettes,
    pub object_sprites: [Option<Sprite>; OBJECTS_PER_TILESET],
    pub tiles: [Option<Bitmap>; TILES_PER_TILESET],
}

#[derive(Clone)]
pub struct GameData {
    pub levels: Vec<Level>,
    pub oddtable: Vec<LevelParamters>,
    pub tilesets: Vec<TileSet>,
    pub special_backgrounds: Vec<Image>,
    pub skill_panel: Bitmap,
    pub lemming_sprites: [Sprite; NUM_LEMMING_SPRITES],
    pub static_palette: [PaletteEntry; PALETTE_SIZE],
}

impl GameData {
    pub fn resolve_level(&self, index: usize) -> Result<Level> {
        if index >= LEVEL_TABLE.len() {
            bail!("no level with index {}", index);
        }

        let entry = LEVEL_TABLE[index] - 1;

        let level_index = 8 * (entry >> 4) + ((entry >> 1) & 0x07);
        let oddtable_index = entry >> 1;

        let level = self
            .levels
            .get(level_index as usize)
            .ok_or(format_err!("invalid level_index {}", level_index))?;

        if (entry & 0x01) == 0 {
            return Ok(level.clone());
        } else {
            return Ok(Level {
                parameters: self
                    .oddtable
                    .get(oddtable_index as usize)
                    .ok_or(format_err!("invalid oddtable_index {}", oddtable_index))?
                    .clone(),
                ..level.clone()
            });
        }
    }

    pub fn resolve_skill_panel_palette(&self, tileset: usize) -> [PaletteEntry; PALETTE_SIZE] {
        let mut palette: [PaletteEntry; PALETTE_SIZE] = [(0, 0, 0); PALETTE_SIZE];

        for i in 0..PALETTE_SIZE {
            palette[i] = match i {
                0..7 => LOWER_PALETTE_FIXED[i],
                7 => self
                    .tilesets
                    .get(tileset)
                    .map(|tileset| tileset.palettes.custom[i])
                    .unwrap_or((0, 0, 0)),
                8.. => UPPER_PALETTE_SKILL_PANEL[i - 8],
            };
        }

        palette
    }

    pub fn resolve_palette(&self, level: &Level) -> Result<[PaletteEntry; PALETTE_SIZE]> {
        if level.extended_graphics_set > 0 {
            self.special_backgrounds
                .get(level.extended_graphics_set as usize - 1)
                .ok_or(format_err!(
                    "invalid extended graphics set {}",
                    level.extended_graphics_set
                ))
                .map(|x| x.palette)
        } else {
            self.tilesets
                .get(level.graphics_set as usize)
                .ok_or(format_err!("invlid graphics set {}", level.graphics_set))
                .map(|x| x.palettes.custom)
        }
    }

    pub fn compose_terrain(&self, level: &Level) -> Result<Bitmap> {
        let mut data: Vec<u8> = vec![0; LEVEL_HEIGHT * LEVEL_WIDTH];
        let mut transparency: Vec<bool> = vec![true; LEVEL_HEIGHT * LEVEL_WIDTH];

        if level.extended_graphics_set > 0 {
            let special_background = self
                .special_backgrounds
                .get(level.extended_graphics_set as usize - 1)
                .ok_or(format_err!(
                    "bad extended graphics set {}",
                    level.extended_graphics_set
                ))?;

            for y in 0..special_background.bitmap.height {
                for x in 0..special_background.bitmap.width {
                    let i_src = y * special_background.bitmap.width + x;
                    let i_dest = y * LEVEL_WIDTH as usize + VGASPEC_POSITION + x;

                    data[i_dest] = if special_background.bitmap.transparency[i_src] {
                        0
                    } else {
                        special_background.bitmap.data[i_src]
                    };

                    transparency[i_dest] = special_background.bitmap.transparency[i_src];
                }
            }
        }

        for tile in &level.terrain_tiles {
            let bitmap_optional = self
                .tilesets
                .get(level.graphics_set as usize)
                .ok_or(format_err!("bad graphics set {}", level.graphics_set))?
                .tiles
                .get(tile.id as usize)
                .and_then(|x| x.as_ref());

            match bitmap_optional {
                None => continue,
                Some(bitmap) => {
                    compose_tile_onto_background(tile, bitmap, &mut data, &mut transparency);
                }
            }
        }

        Ok(Bitmap {
            width: LEVEL_WIDTH,
            height: LEVEL_HEIGHT,
            data,
            transparency,
        })
    }
}

fn compose_tile_onto_background(
    tile: &TerrainTile,
    bitmap: &Bitmap,
    data: &mut Vec<u8>,
    transparency: &mut Vec<bool>,
) -> () {
    for y in 0..bitmap.height {
        for x in 0..bitmap.width {
            let y_transformed = if tile.flip_y {
                bitmap.height - 1 - y
            } else {
                y
            };

            let x_dest = tile.x + x as i32;
            let y_dest = tile.y + y as i32;
            if x_dest < 0
                || x_dest >= LEVEL_WIDTH as i32
                || y_dest < 0
                || y_dest >= LEVEL_HEIGHT as i32
            {
                continue;
            }

            let src_index = (y_transformed * bitmap.width + x) as usize;
            let dest_index = (y_dest * LEVEL_WIDTH as i32 + x_dest) as usize;

            if tile.do_not_overwrite {
                if transparency[dest_index] && !bitmap.transparency[src_index] {
                    data[dest_index] = bitmap.data[src_index];
                    transparency[dest_index] = false;
                }
            } else if tile.remove_terrain {
                if !bitmap.transparency[src_index] {
                    transparency[dest_index] = true;
                }
            } else {
                if !bitmap.transparency[src_index] {
                    data[dest_index] = bitmap.data[src_index];
                    transparency[dest_index] = false;
                }
            }
        }
    }
}
