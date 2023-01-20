use crate::{file, level::Level};
use std::{fs, path::Path};

use anyhow::{Context, Result};

pub fn main(path_name: &str, verbose: bool) -> Result<()> {
    let path = Path::new(path_name);

    let compressed_data = fs::read(path.as_os_str())
        .with_context(|| format!("failed to load read '{}'", path_name))?;

    let decompressed_sections = file::encoding::datfile::parse(&compressed_data)?;

    for (index, section) in decompressed_sections.sections.iter().enumerate() {
        let level = Level::decode(&section.data)?;

        println!("Level {}:", index);
        println!("{}", level);

        if verbose {
            println!("");

            for (index, tile) in level.terrain_tiles.iter().enumerate() {
                println!("tile {}:\n{}\n", index, tile);
            }
        }
    }

    Ok(())
}
