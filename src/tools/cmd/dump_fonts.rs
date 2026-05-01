use std::path::Path;

use anyhow::{Result, format_err};
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
            .get(index as usize)
            .ok_or(format_err!("invalid skill panel font index {}", index))?;

        println!("'{}‘", c);
        println!("===");
        for y in 0..bitmap.height {
            for x in 0..bitmap.width {
                print!("{:#04x} ", bitmap.data[y * bitmap.width + x]);
            }

            println!();
        }

        println!();
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
            .get(index as usize)
            .ok_or(format_err!("invalid skill panel font index {}", index))?;

        println!("'{}‘", c);
        println!("===");
        for y in 0..bitmap.height {
            for x in 0..bitmap.width {
                print!("{:#04x} ", bitmap.data[y * bitmap.width + x]);
            }

            println!();
        }

        println!();
    }

    Ok(())
}
