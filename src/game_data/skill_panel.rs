use crate::game_data::{Bitmap, SCREEN_WIDTH};

pub const MINIMAP_FRAME_HEIGHT: usize = 20;
pub const MINIMAP_FRAME_WIDTH: usize = 22;
const MINIMAP_FRAME_COLOR: u8 = 3;

pub const MINIMAP_AREA_WIDTH: usize = 104;
pub const MINIMAP_AREA_HEIGHT: usize = 20;
pub const MINIMAP_AREA_X: usize = 208;
pub const MINIMAP_AREA_Y: usize = 18;

pub const MINIMAP_VIEW_WIDTH: usize = 100;
pub const MINIMAP_VIEW_HEIGHT: usize = 18;
pub const MINIMAP_VIEW_X: usize = MINIMAP_AREA_X + 2;
pub const MINIMAP_VIEW_Y: usize = MINIMAP_AREA_Y + 1;

#[derive(Clone)]
pub struct SkillPanel {
    pub panel: Bitmap,
    pub minimap_frame: Bitmap,
}

impl SkillPanel {
    pub fn new(mut panel: Bitmap) -> SkillPanel {
        for y in MINIMAP_AREA_Y..MINIMAP_AREA_Y + MINIMAP_AREA_HEIGHT {
            for x in MINIMAP_AREA_X..MINIMAP_AREA_X + MINIMAP_AREA_WIDTH {
                panel.transparency[x + y * SCREEN_WIDTH] = true;
            }
        }

        SkillPanel {
            panel,
            minimap_frame: create_minimap_frame(),
        }
    }
}

fn create_minimap_frame() -> Bitmap {
    let mut data: Vec<u8> = vec![0; MINIMAP_FRAME_WIDTH * MINIMAP_FRAME_HEIGHT + 1];
    let mut transparency: Vec<bool> = vec![true; MINIMAP_FRAME_WIDTH * MINIMAP_FRAME_HEIGHT + 1];

    let mut i: usize = 0;
    for y in 0..MINIMAP_FRAME_HEIGHT {
        for x in 0..MINIMAP_FRAME_WIDTH {
            if x == 0 || x == MINIMAP_FRAME_WIDTH - 1 || y == 0 || y == MINIMAP_FRAME_HEIGHT - 1 {
                data[i] = MINIMAP_FRAME_COLOR;
                transparency[i] = false;
            }

            i += 1;
        }
    }

    Bitmap {
        width: MINIMAP_FRAME_WIDTH,
        height: MINIMAP_FRAME_HEIGHT,
        data,
        transparency,
    }
}
