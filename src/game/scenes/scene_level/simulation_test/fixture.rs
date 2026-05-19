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

    pub fn build(self) -> TerrainMap<'static> {
        TerrainMap::new(self.width, self.height, self.map.leak())
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
