use rustlings::game_data::Bitmap;

#[derive(Default, Clone)]
pub enum Screen {
    #[default]
    Level,
}

#[derive(Default, Clone)]
pub struct GameState {
    pub screen: Screen,
    pub current_level: usize,
}

#[derive(Clone)]
pub struct ObjectState {
    pub triggered: bool,
    pub frame: usize,
}

#[derive(Clone)]
pub struct SceneStateLevel {
    pub level_x: usize,
    pub terrain: Bitmap,
    pub object_state: Vec<ObjectState>,

    pub current_clock_msec: u64,
}

#[derive(Default, Clone)]
pub enum SceneState {
    #[default]
    None,
    Level(SceneStateLevel),
}
