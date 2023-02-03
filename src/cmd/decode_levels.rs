use std::{cmp::Ordering, fs, io::Write, path::Path};

use anyhow::Result;

use crate::game_data::{read_game_data, LevelStructure};

fn comparator<T: LevelStructure>(o1: &T, o2: &T) -> Ordering {
    if o1.get_id() == o2.get_id() {
        if o1.get_x() > o2.get_x() {
            Ordering::Greater
        } else if o1.get_x() < o2.get_x() {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    } else if o1.get_id() > o2.get_id() {
        Ordering::Greater
    } else {
        Ordering::Less
    }
}

pub fn main(path: &Path, destination: &str) -> Result<()> {
    let game_data = read_game_data(path)?;

    for level in game_data.levels {
        let file_name = format!("{}.level.txt", level.name);
        let file_path = Path::new(destination).join(Path::new(&file_name));

        let mut file = fs::File::create(file_path)?;

        writeln!(&mut file, "{}", level)?;

        writeln!(&mut file)?;
        writeln!(&mut file, "Objects:")?;
        writeln!(&mut file)?;

        let mut objects = level.objects.clone();
        objects.sort_by(comparator);

        for object in &objects {
            writeln!(&mut file, "{}", object)?;
            writeln!(&mut file)?;
        }

        writeln!(&mut file)?;
        writeln!(&mut file, "Tiles:")?;
        writeln!(&mut file)?;

        let mut terrain_tiles = level.terrain_tiles.clone();
        terrain_tiles.sort_by(comparator);

        for tile in &terrain_tiles {
            writeln!(&mut file, "{}", tile)?;
            writeln!(&mut file)?;
        }
    }

    Ok(())
}
