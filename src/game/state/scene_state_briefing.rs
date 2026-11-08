use crate::state::ObjectsState;

#[derive(Clone)]
pub struct SceneStateBriefing {
    pub clock_msec: u64,

    pub objects: ObjectsState,
}
