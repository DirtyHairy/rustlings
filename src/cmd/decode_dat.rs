use anyhow::{Context, Ok, Result};
use std::{fs, path::Path};

use crate::game_data::file::encoding::datfile;

pub fn main(path_name: &str) -> Result<()> {
    let path = Path::new(path_name);

    let compressed_data = fs::read(path.as_os_str())
        .with_context(|| format!("failed to load read '{}'", path_name))?;

    let datfile = datfile::parse(&compressed_data)?;

    for (index, section) in datfile.sections.iter().enumerate() {
        let file_name = format!(
            "{}.section.{}",
            path.file_name()
                .and_then(|s| s.to_str())
                .expect("weird filename - fix your file system"),
            index
        );

        println!("writing '{}'", file_name);

        fs::write(Path::new(&file_name), &section.data)
            .context(format!("failed to write '{}'", file_name))?;
    }

    Ok(())
}
