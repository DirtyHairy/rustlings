use crate::{scenes::scene_level::simulation::*, state::Direction};

pub struct TerrainFixtureBuilder {
    width: u32,
    height: u32,
    map: Vec<TerrainProps>,
}

impl TerrainFixtureBuilder {
    pub fn new(width: u32, height: u32) -> Self {
        let map: Vec<TerrainProps> = vec![Default::default(); (width * height) as usize];

        Self { width, height, map }
    }

    pub fn with(mut self, x: u32, y: u32, terrain_props: TerrainProps) -> Self {
        self.map[(x + y * self.width) as usize] = terrain_props.with_solid(true);

        self
    }

    pub fn with_non_solid(mut self, x: u32, y: u32, terrain_props: TerrainProps) -> Self {
        self.map[(x + y * self.width) as usize] = terrain_props;

        self
    }

    pub fn with_col(mut self, x: u32, y: u32, len: u32, terrain_props: TerrainProps) -> Self {
        for row in 0..len {
            self.map[(x + y.saturating_sub(row) * self.width) as usize] =
                terrain_props.with_solid(true);
        }

        self
    }

    pub fn with_row(mut self, x: u32, y: u32, len: u32, terrain_props: TerrainProps) -> Self {
        for col in 0..len {
            self.map[(x + col + y * self.width) as usize] = terrain_props.with_solid(true);
        }

        self
    }

    pub fn with_block(
        mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        terrain_props: TerrainProps,
    ) -> Self {
        for row in 0..height {
            self = self.with_row(x, y + row, width, terrain_props);
        }

        self
    }

    pub fn build(self) -> Terrain<'static> {
        let bitmap = Box::new(Bitmap::filled(self.width, self.height, 0, true));

        Terrain::new(
            Box::leak(bitmap),
            self.map.leak(),
            Box::leak(Box::default()),
        )
    }
}

impl LemmingState {
    pub fn fixture(x: i32, y: i32, direction: Direction, activity: Activity) -> Self {
        let mut lemming = LemmingState {
            x,
            y,
            direction,
            ..Default::default()
        };

        lemming.transition_to(activity);

        lemming
    }
}

impl Terrain<'_> {
    pub fn is_row_blank(&self, x: i32, y: i32, width: u32) -> bool {
        for i in 0..width {
            if self.is_solid(x + i as i32, y) {
                return false;
            }
        }

        true
    }

    pub fn is_block_blank(&self, x: i32, y: i32, width: u32, height: u32) -> bool {
        for i in 0..height {
            if !self.is_row_blank(x, y + i as i32, width) {
                return false;
            }
        }

        true
    }

    pub fn sorted_diff(&self) -> Vec<TerrainDiff> {
        let mut sorted = self.diff.clone();
        sorted.sort_by_key(|d| d.y);

        sorted
    }
}
