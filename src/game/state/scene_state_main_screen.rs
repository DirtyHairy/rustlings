#[derive(Clone, PartialEq)]
pub enum ChoiceMainScreen {
    Play,
    Quit,
}

#[derive(Clone, PartialEq)]
pub enum ScrollerPhase {
    Scroll,
    Pause,
}

pub const SCROLLER_VISIBLE_CHARS: usize = 35;

#[derive(Clone)]
pub struct ScrollerState {
    pub ticks_in_phase: u64,
    pub clock_msec: u64,

    pub phase: ScrollerPhase,
    pub next_char_index: usize,

    pub current_scroller_frame: usize,
    pub current_line: [usize; SCROLLER_VISIBLE_CHARS],
}

#[derive(Clone)]
pub struct SceneStateMainScreen {
    pub choice: Option<ChoiceMainScreen>,
    pub scroller: ScrollerState,

    pub clock_msec: u64,
}
