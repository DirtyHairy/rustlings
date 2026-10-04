use std::path::Path;

use anyhow::{Result, anyhow};
use rustlings::game_data::{
    read_game_data, resolve_skill_panel_font_index, resolve_skill_panel_skill_font_index,
};

const CHARS_SKILL_PANEL: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ%- ";
const CHARS_SKILL: &str = "0123456789 ";

pub fn main(path: &Path) -> Result<()> {
    let game_data = read_game_data(path)?;

    println!("skill panel");
    println!("###########");
    println!();

    for c in CHARS_SKILL_PANEL.chars() {
        let index = resolve_skill_panel_font_index(c);

        let bitmap = game_data
            .skill_panel
            .font
            .frames
            .get(index)
            .ok_or(anyhow!("invalid skill panel font index {}", index))?;

        println!("’{}’", c);
        println!("===");
        println!("{}", bitmap);
    }

    println!("skills");
    println!("######");
    println!();

    for c in CHARS_SKILL.chars() {
        let index = resolve_skill_panel_skill_font_index(c);

        let bitmap = game_data
            .skill_panel
            .font_skills
            .frames
            .get(index)
            .ok_or(anyhow!("invalid skill panel font index {}", index))?;

        println!("’{}’", c);
        println!("===");
        println!("{}", bitmap);
    }

    println!("countdown");
    println!("######");
    println!();

    for i in 0..10 {
        let bitmap = game_data
            .font_countdown
            .frames
            .get(i)
            .ok_or(anyhow!("invalid countdown font index {}", i))?;

        println!("'{}'", i);
        println!("===");
        println!("{}", bitmap);
    }

    Ok(())
}
