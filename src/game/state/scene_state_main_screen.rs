#[derive(Clone, PartialEq)]
pub enum ChoiceMainScreen {
    Play,
    Quit,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ScrollerPhase {
    Scroll,
    Pause,
}

pub const SCROLLER_VISIBLE_CHARS: usize = 35;
pub const EYE_COUNT: usize = 9;

#[derive(Clone)]
pub struct EyesState {
    pub clock_musec: u64,
    pub eye_counters: [isize; EYE_COUNT],
    pub tick_duration_musec: u64,
}

impl EyesState {
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

#[derive(Clone)]
pub struct ScrollerState {
    pub ticks_in_phase: u64,
    pub clock_musec: u64,
    pub tick_duration_musec: u64,

    pub phase: ScrollerPhase,
    pub next_char_index: usize,

    pub current_scroller_frame: usize,
    pub current_line: [usize; SCROLLER_VISIBLE_CHARS],
}

impl ScrollerState {
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

#[derive(Clone)]
pub struct SceneStateMainScreen {
    pub choice: Option<ChoiceMainScreen>,
    pub scroller: ScrollerState,
    pub eyes: EyesState,
}
