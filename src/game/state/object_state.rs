#[derive(Clone, Default)]
pub struct ObjectState {
    pub triggered: bool,
    pub frame: usize,
}

pub type ObjectsState = Vec<ObjectState>;
