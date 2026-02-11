use std::path::Path;

use crate::game_data::file::level::read_oddtable;

use super::{
    file::{self, level::read_level_file},
    GameData, Level, SpecialBackground, TileSet, PALETTE_SIZE,
};

use anyhow::Result;

const NUM_LEVELS_FILES: usize = 10;
const LEVELS_PER_FILE: usize = 8;
const NUM_TILESETS: usize = 5;
const NUM_SPECIAL_BACKGROUND: usize = 4;

const STATIC_PALETTE: [(usize, usize, usize); PALETTE_SIZE] = [
    (0, 0, 0),
    (64, 64, 224),
    (0, 176, 0),
    (240, 208, 208),
    (176, 176, 0),
    (240, 32, 32),
    (128, 128, 128),
    (0, 0, 0),
    (64, 64, 224),
    (0, 176, 0),
    (240, 208, 208),
    (176, 176, 0),
    (240, 32, 32),
    (128, 128, 128),
    (0, 0, 0),
    (64, 64, 224),
];

pub fn read_game_data(path: &Path) -> Result<GameData> {
    let mut levels: Vec<Level> = Vec::with_capacity(LEVELS_PER_FILE * NUM_LEVELS_FILES);

    for i in 0..NUM_LEVELS_FILES {
        levels.append(&mut read_level_file(path, i)?)
    }

    let oddtable = read_oddtable(path)?;

    let mut tilesets: Vec<TileSet> = Vec::with_capacity(NUM_TILESETS);

    for i in 0..NUM_TILESETS {
        let ground_dat = file::ground::read_ground(path, i)?;
        let vgagr =
            file::vgagr::read_vgagr(path, i, &ground_dat.object_info, &ground_dat.terrain_info)?;

        tilesets.push(TileSet {
            object_info: ground_dat.object_info,
            terrain_info: ground_dat.terrain_info,
            palettes: ground_dat.palettes,
            object_sprites: vgagr.object_sprites,
            tiles: vgagr.tiles,
        })
    }

    let mut special_backgrounds: Vec<SpecialBackground> =
        Vec::with_capacity(NUM_SPECIAL_BACKGROUND);

    for i in 0..NUM_SPECIAL_BACKGROUND {
        let vgaspec = file::vgaspec::read_vgaspec(path, i)?;

        special_backgrounds.push(SpecialBackground {
            palette: vgaspec.palette,
            bitmap: vgaspec.bitmap,
        });
    }

    let main = file::main::read_main(path)?;

    Ok(GameData {
        levels,
        oddtable,
        tilesets,
        special_backgrounds,
        static_palette: STATIC_PALETTE,
        lemming_sprites: main.lemming_sprites,
    })
}
