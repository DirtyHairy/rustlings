use std::{cmp::Ordering, fs, io::Write, path::Path};

use anyhow::Result;

use crate::file::level::Level;

use super::util::read_levels;

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

        level.terrain_tiles.sort_by(|t1, t2| {
            if t1.id == t2.id {
                if t1.x > t2.x {
                    Ordering::Greater
                } else if t1.x < t2.x {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            } else if t1.id > t2.id {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });

        for tile in &level.terrain_tiles {
            writeln!(&mut file, "{}", tile)?;
            writeln!(&mut file)?;
        }
    }

    Ok(())
}
