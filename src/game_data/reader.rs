use std::path::Path;
use std::thread::{self, ScopedJoinHandle};

use anyhow::Result;

use crate::game_data::SkillPanel;
use crate::game_data::file::ground::read_ground;
use crate::game_data::file::level::{Level, read_level_file, read_oddtable};
use crate::game_data::file::main::read_main;
use crate::game_data::file::palette::{LOWER_PALETTE_FIXED, PALETTE_SIZE, PaletteEntry};
use crate::game_data::file::vgagr::read_vgagr;
use crate::game_data::file::vgaspec::read_vgaspec;
use crate::game_data::{Cursors, GameData, Image, TileSet};

const NUM_LEVELS_FILES: usize = 10;
const LEVELS_PER_FILE: usize = 8;
const NUM_TILESETS: usize = 5;
const NUM_SPECIAL_BACKGROUND: usize = 4;

pub fn read_game_data(path: &Path) -> Result<GameData> {
    thread::scope(|s| {
        let tileset_handles: Vec<ScopedJoinHandle<Result<TileSet>>> = (0..NUM_TILESETS)
            .map(|i| {
                s.spawn(move || -> Result<TileSet> {
                    let ground_dat = read_ground(path, i)?;
                    let vgagr =
                        read_vgagr(path, i, &ground_dat.object_info, &ground_dat.terrain_info)?;

                    Ok(TileSet {
                        object_info: ground_dat.object_info,
                        terrain_info: ground_dat.terrain_info,
                        palettes: ground_dat.palettes,
                        object_sprites: vgagr.object_sprites,
                        tiles: vgagr.tiles,
                    })
                })
            })
            .collect();

        let special_background_handles: Vec<ScopedJoinHandle<Result<Image>>> = (0
            ..NUM_SPECIAL_BACKGROUND)
            .map(|i| {
                s.spawn(move || -> Result<Image> {
                    let vgaspec = read_vgaspec(path, i)?;

                    Ok(Image {
                        palette: vgaspec.palette,
                        bitmap: vgaspec.bitmap,
                    })
                })
            })
            .collect();

        let main_handle = s.spawn(|| read_main(path));

        let levels: Vec<Level> = (0..NUM_LEVELS_FILES).try_fold::<_, _, Result<Vec<Level>>>(
            Vec::with_capacity(LEVELS_PER_FILE * NUM_LEVELS_FILES),
            |mut acc, i| {
                acc.append(&mut read_level_file(path, i)?);
                Ok(acc)
            },
        )?;

        let oddtable = read_oddtable(path)?;

        let mut static_palette: [PaletteEntry; PALETTE_SIZE] = [(0, 0, 0); PALETTE_SIZE];
        for i in 0..PALETTE_SIZE {
            static_palette[i] = LOWER_PALETTE_FIXED[i % LOWER_PALETTE_FIXED.len()];
        }

        let tilesets: Vec<TileSet> = tileset_handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Result<Vec<TileSet>>>()?;

        let special_backgrounds: Vec<Image> = special_background_handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Result<Vec<Image>>>()?;

        let main = main_handle.join().unwrap()?;

        println!();

        Ok(GameData {
            levels,
            oddtable,
            tilesets,
            special_backgrounds,
            static_palette,
            skill_panel: SkillPanel::new(
                main.skill_panel,
                main.font_skill_panel,
                main.font_skill_panel_skills,
            ),
            lemming_sprites: main.lemming_sprites,
            cursors: Cursors::new(),
        })
    })
}
