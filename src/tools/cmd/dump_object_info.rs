use std::path::Path;

use anyhow::Result;
use rustlings::game_data::read_game_data;

pub fn main(path: &Path) -> Result<()> {
    let game_data = read_game_data(path)?;

    for (i_tile_set, tile_set) in game_data.tilesets.iter().enumerate() {
        println!("TILESET {}", i_tile_set);
        println!("===");
        println!();

        for (i_object_info, object_info) in tile_set.object_info.iter().enumerate() {
            println!("OBJECT {}", i_object_info);
            println!("---");
            println!("{}", object_info);
            println!();
        }
    }

    Ok(())
}
