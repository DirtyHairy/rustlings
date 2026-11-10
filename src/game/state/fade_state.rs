#[derive(Clone, Default)]
pub struct FadeState {
    pub clock_msec: u64,
    pub fadeout_started_at_msec: Option<u64>,
}
