pub use crate::game_data::file::main::{
    resolve_skill_panel_font_index, resolve_skill_panel_skill_font_index,
};
use crate::game_data::{Bitmap, SCREEN_WIDTH, Sprite};

pub const SKILL_PANEL_HEIGHT: u32 = 40;

pub const MINIMAP_FRAME_HEIGHT: u32 = 20;
pub const MINIMAP_FRAME_WIDTH: u32 = 22;
const FRAME_COLOR: u8 = 3;

pub const MINIMAP_AREA_WIDTH: u32 = 104;
pub const MINIMAP_AREA_HEIGHT: u32 = 20;
pub const MINIMAP_AREA_X: u32 = 208;
pub const MINIMAP_AREA_Y: u32 = 18;

pub const MINIMAP_VIEW_WIDTH: u32 = 100;
pub const MINIMAP_VIEW_HEIGHT: u32 = 18;
pub const MINIMAP_VIEW_X: u32 = MINIMAP_AREA_X + 2;
pub const MINIMAP_VIEW_Y: u32 = MINIMAP_AREA_Y + 1;

pub const SKILL_TILE_WIDTH: u32 = 16;
pub const SKILL_TILE_HEIGHT: u32 = 24;
pub const SKILL_TILE_Y: u32 = 16;
pub const SKILL_TILE_LABEL_X: u32 = 4;
pub const SKILL_TILE_LABEL_Y: u32 = 1;
pub const SKILL_TILE_LABEL_WIDTH: u32 = 8;
pub const SKILL_TILE_LABEL_HEIGHT: u32 = 8;

#[derive(Clone)]
pub struct SkillPanel {
    pub panel: Bitmap,
    pub minimap_frame: Bitmap,
    pub skill_tile_frame: Bitmap,
    pub font_skills: Sprite,
    pub font: Sprite,
}

impl SkillPanel {
    pub fn new(mut panel: Bitmap, font: Sprite, font_skills: Sprite) -> SkillPanel {
        for y in MINIMAP_AREA_Y..MINIMAP_AREA_Y + MINIMAP_AREA_HEIGHT {
            for x in MINIMAP_AREA_X..MINIMAP_AREA_X + MINIMAP_AREA_WIDTH {
                panel.transparency[(x + y * SCREEN_WIDTH) as usize] = true;
            }
        }

        SkillPanel {
            panel,
            minimap_frame: create_frame(MINIMAP_FRAME_WIDTH, MINIMAP_FRAME_HEIGHT),
            skill_tile_frame: create_frame(SKILL_TILE_WIDTH, SKILL_PANEL_HEIGHT),
            font,
            font_skills,
        }
    }
}

fn create_frame(width: u32, height: u32) -> Bitmap {
    let size = (width * height) as usize + 1;
    let mut data: Vec<u8> = vec![0; size];
    let mut transparency: Vec<bool> = vec![true; size];

    let mut i: usize = 0;
    for y in 0..height {
        for x in 0..width {
            if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                data[i] = FRAME_COLOR;
                transparency[i] = false;
            }

            i += 1;
        }
    }

    Bitmap {
        width,
        height,
        data,
        transparency,
    }
}
