#[derive(Clone, Copy)]
pub enum TerrainDiffKind {
    Dig,
}

#[derive(Clone, Copy)]
pub enum VisibilityTarget {
    Now,
    Next,
}

#[derive(Clone, Copy)]
pub struct TerrainDiff {
    pub x: i32,
    pub y: i32,

    pub kind: TerrainDiffKind,
}

impl TerrainDiff {
    pub fn visibility_target(self) -> VisibilityTarget {
        match self.kind {
            TerrainDiffKind::Dig => VisibilityTarget::Next,
        }
    }
}
