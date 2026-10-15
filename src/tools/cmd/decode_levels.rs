use std::{fs, io::Write, path::Path};

use anyhow::Result;
use rustlings::game_data::read_game_data;

pub fn main(path: &Path, destination: &str) -> Result<()> {
    let game_data = read_game_data(path)?;

    for level in game_data.levels {
        let file_name = format!("{}.level.txt", level.parameters.name);
        let file_path = Path::new(destination).join(Path::new(&file_name));

        let mut file = fs::File::create(file_path)?;

        writeln!(&mut file, "{}", level)?;

        writeln!(&mut file)?;
        writeln!(&mut file, "Objects:")?;
        writeln!(&mut file)?;

        let mut objects = level.objects.clone();
        objects.sort_by_key(|o| (o.id, o.x));

        for object in &objects {
            writeln!(&mut file, "{}", object)?;
            writeln!(&mut file)?;
        }

        writeln!(&mut file)?;
        writeln!(&mut file, "Tiles:")?;
        writeln!(&mut file)?;

        let mut terrain_tiles = level.terrain_tiles.clone();
        terrain_tiles.sort_by_key(|t| (t.id, t.x));

        for tile in &terrain_tiles {
            writeln!(&mut file, "{}", tile)?;
            writeln!(&mut file)?;
        }
    }

    Ok(())
}
