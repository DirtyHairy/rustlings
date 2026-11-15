use crate::state::ObjectsState;

#[derive(Clone)]
pub enum ChoiceBriefing {
    Launch,
    NextLevel,
    PreviousLevel,
    MainScreen,
}

#[derive(Clone)]
pub struct SceneStateBriefing {
    pub clock_msec: u64,
    pub choice: Option<ChoiceBriefing>,

    pub objects: ObjectsState,
}
