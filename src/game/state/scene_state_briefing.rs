use crate::state::{FadeState, ObjectsState};

#[derive(Clone)]
pub struct StateBriefing {
    pub clock_msec: u64,

    pub objects: ObjectsState,
}

pub type SceneStateBriefing = (FadeState, Option<StateBriefing>);
