use crate::file;
use anyhow::{Context, Ok, Result};
use std::{fs, path::Path};

pub fn main(path_name: &str) -> Result<()> {
    let path = Path::new(path_name);

    let compressed_data = fs::read(path.as_os_str())
        .with_context(|| format!("failed to load read '{}'", path_name))?;

    let decompressed_sections = file::encoding::datfile::parse(&compressed_data)?;

    for (index, section) in decompressed_sections.sections.iter().enumerate() {
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
