#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TerrainDiffKind {
    Dig,
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
        }
    }
}
