use crate::state::SceneStateLevel;

#[derive(Default, Clone)]
pub enum SceneState {
    #[default]
    None,
    Level(Box<SceneStateLevel>),
}
