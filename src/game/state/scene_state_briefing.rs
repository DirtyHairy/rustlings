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
    pub clock_musec: u64,
    pub tick_duration_musec: u64,
    pub choice: Option<ChoiceBriefing>,

    pub objects: ObjectsState,
}

impl SceneStateBriefing {
    pub fn update_tick_duration_musec(&mut self, tick_duration_musec: u64) {
        let old_tick_duration_musec = self.tick_duration_musec;
        self.tick_duration_musec = tick_duration_musec;

        if old_tick_duration_musec == 0 {
            return;
        }

        self.clock_musec = ((self.clock_musec * tick_duration_musec) / old_tick_duration_musec)
            .next_multiple_of(tick_duration_musec);
    }
}
