use rustlings::game_data::file::main::MaskSprite;

pub const DIG_LINE_WIDTH: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TerrainDiffKind {
    Dig,
    Mask(MaskSprite, usize),
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
            TerrainDiffKind::Mask(_, _) => VisibilityTarget::Early,
        }
    }
}
