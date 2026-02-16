use super::file::ground::read_ground;
use super::file::level::{Level, read_level_file, read_oddtable};
use super::file::main::read_main;
use super::file::palette::{LOWER_PALETTE_FIXED, PALETTE_SIZE, PaletteEntry};
use super::file::vgagr::read_vgagr;
use super::file::vgaspec::read_vgaspec;
use super::game_data::{GameData, Image, TileSet};
use anyhow::Result;
use std::path::Path;

const NUM_LEVELS_FILES: usize = 10;
const LEVELS_PER_FILE: usize = 8;
const NUM_TILESETS: usize = 5;
const NUM_SPECIAL_BACKGROUND: usize = 4;

pub fn read_game_data(path: &Path) -> Result<GameData> {
    let mut levels: Vec<Level> = Vec::with_capacity(LEVELS_PER_FILE * NUM_LEVELS_FILES);

    for i in 0..NUM_LEVELS_FILES {
        levels.append(&mut read_level_file(path, i)?)
    }

    let oddtable = read_oddtable(path)?;

    let mut tilesets: Vec<TileSet> = Vec::with_capacity(NUM_TILESETS);

    for i in 0..NUM_TILESETS {
        let ground_dat = read_ground(path, i)?;
        let vgagr = read_vgagr(path, i, &ground_dat.object_info, &ground_dat.terrain_info)?;

        tilesets.push(TileSet {
            object_info: ground_dat.object_info,
            terrain_info: ground_dat.terrain_info,
            palettes: ground_dat.palettes,
            object_sprites: vgagr.object_sprites,
            tiles: vgagr.tiles,
        })
    }

    let mut special_backgrounds: Vec<Image> = Vec::with_capacity(NUM_SPECIAL_BACKGROUND);

    for i in 0..NUM_SPECIAL_BACKGROUND {
        let vgaspec = read_vgaspec(path, i)?;

        special_backgrounds.push(Image {
            palette: vgaspec.palette,
            bitmap: vgaspec.bitmap,
        });
    }

    let main = read_main(path)?;

    let mut static_palette: [PaletteEntry; PALETTE_SIZE] = [(0, 0, 0); PALETTE_SIZE];
    for i in 0..PALETTE_SIZE {
        static_palette[i] = LOWER_PALETTE_FIXED[i % LOWER_PALETTE_FIXED.len()];
    }

    Ok(GameData {
        levels,
        oddtable,
        tilesets,
        special_backgrounds,
        static_palette,
        skill_panel: main.skill_panel,
        lemming_sprites: main.lemming_sprites,
    })
}
