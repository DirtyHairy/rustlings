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
}

#[derive(Clone)]
pub struct ScrollerState {
    pub ticks_in_phase: u64,
    pub clock_musec: u64,

    pub phase: ScrollerPhase,
    pub next_char_index: usize,

    pub current_scroller_frame: usize,
    pub current_line: [usize; SCROLLER_VISIBLE_CHARS],
}

#[derive(Clone)]
pub struct SceneStateMainScreen {
    pub choice: Option<ChoiceMainScreen>,
    pub scroller: ScrollerState,
    pub eyes: EyesState,

    pub clock_musec: u64,
}
