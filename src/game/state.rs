#[derive(Default)]
pub enum Screen {
    #[default]
    Level,
}

#[derive(Default)]
pub struct GameState {
    pub screen: Screen,
}

#[derive(Default, Clone)]
pub struct SceneStateLevel {}

#[derive(Default)]
pub enum SceneState {
    #[default]
    None,
    Level(SceneStateLevel),
}
