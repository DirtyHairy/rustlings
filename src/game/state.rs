#[derive(Default, Clone)]
pub enum Screen {
    #[default]
    Level,
}

#[derive(Default, Clone)]
pub struct GameState {
    pub screen: Screen,
}

#[derive(Default, Clone)]
pub struct SceneStateLevel {
    pub level_x: usize,
}

#[derive(Default, Clone)]
pub enum SceneState {
    #[default]
    None,
    Level(SceneStateLevel),
}
