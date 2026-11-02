use std::{fs, path::Path};

use anyhow::{Context, Result, anyhow};

pub fn read_game_file(path: &Path, name_lc: &str) -> Result<Vec<u8>> {
    let full_path_lc = path.join(name_lc);

    if fs::exists(&full_path_lc).unwrap_or(false) {
        return fs::read(&full_path_lc).with_context(|| format!("failed to read {}", name_lc));
    }

    let name_uc = name_lc.to_uppercase();
    let full_path_uc = path.join(&name_uc);

    if fs::exists(&full_path_uc).unwrap_or(false) {
        return fs::read(&full_path_uc).with_context(|| format!("failed to read {}", name_uc));
    }

    Err(anyhow!("unable to find {} or {}", name_lc, name_uc))
}
