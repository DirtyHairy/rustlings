use std::{cmp::Ordering, fs, io::Write, path::Path};

use anyhow::Result;

use crate::file::level::{Level, LevelStructure};

use super::util::read_levels;

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

pub fn main(data_path: &Path, destination: &str) -> Result<()> {
    let mut levels: Vec<Level> = Vec::new();

    for i in 0..10 {
        levels.append(&mut read_levels(
            data_path
                .join(format!("level00{}.dat", i))
                .to_str()
                .unwrap(),
        )?)
    }

    for level in &mut levels {
        let file_name = format!("{}.level.txt", level.name);
        let file_path = Path::new(destination).join(Path::new(&file_name));

        let mut file = fs::File::create(file_path)?;

        writeln!(&mut file, "{}", level)?;

        writeln!(&mut file)?;
        writeln!(&mut file, "Objects:")?;
        writeln!(&mut file)?;

        level.objects.sort_by(comparator);

        for object in &level.objects {
            writeln!(&mut file, "{}", object)?;
            writeln!(&mut file)?;
        }

        writeln!(&mut file)?;
        writeln!(&mut file, "Tiles:")?;
        writeln!(&mut file)?;

        level.terrain_tiles.sort_by(comparator);

        for tile in &level.terrain_tiles {
            writeln!(&mut file, "{}", tile)?;
            writeln!(&mut file)?;
        }
    }

    Ok(())
}
