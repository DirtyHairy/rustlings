use rustlings::game_data::file::main::MaskSprite;

pub const DIG_LINE_WIDTH: u32 = 9;
pub const BRICK_LINE_WIDTH: u32 = 6;
pub const BRICK_COLOR: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TerrainDiffKind {
    Dig,
    Mask(MaskSprite, usize),
    Brick,
}

#[derive(Clone, Copy, PartialEq)]
pub enum VisibilityTarget {
    Early,
    Late,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainDiff {
    pub x: i32,
    pub y: i32,

    pub kind: TerrainDiffKind,
}

impl TerrainDiff {
    pub fn visibility_target(self) -> VisibilityTarget {
        match self.kind {
            TerrainDiffKind::Dig => VisibilityTarget::Late,
            TerrainDiffKind::Mask(_, _) => VisibilityTarget::Late,
            TerrainDiffKind::Brick => VisibilityTarget::Late,
        }
    }
}
