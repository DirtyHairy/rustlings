pub use super::file::ground::{
    OBJECTS_PER_TILESET, ObjectInfo, Palettes, TILES_PER_TILESET, TerrainInfo,
};
pub use super::file::level::{Level, LevelParamters, LevelStructure, Object, TerrainTile};
pub use super::file::main::NUM_LEMMING_SPRITES;
pub use super::file::palette::{PALETTE_SIZE, PaletteEntry};
pub use super::file::sprite::{Bitmap, Sprite};

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

pub const DIFFICULTY_RATINGS: [&str; 4] = ["Fun", "Tricky", "Taxing", "Mayhem"];

#[derive(Clone)]
pub struct SpecialBackground {
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
    pub special_backgrounds: Vec<SpecialBackground>,
    pub lemming_sprites: [Sprite; NUM_LEMMING_SPRITES],
    pub static_palette: [PaletteEntry; PALETTE_SIZE],
}

impl GameData {
    pub fn resolve_level(&self, index: usize) -> Option<Level> {
        if index >= LEVEL_TABLE.len() {
            return None;
        }

        let entry = LEVEL_TABLE[index] - 1;

        let level_index = 8 * (entry >> 4) + ((entry >> 1) & 0x07);
        let oddtable_index = entry >> 1;

        let level = self.levels.get(level_index as usize)?;

        if (entry & 0x01) == 0 {
            return Some(level.clone());
        } else {
            return Some(Level {
                parameters: self.oddtable.get(oddtable_index as usize)?.clone(),
                ..level.clone()
            });
        }
    }
}
