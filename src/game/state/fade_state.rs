#[derive(Clone, Default)]
pub struct FadeState {
    pub clock_msec: u64,
    pub fade_out_started_at_msec: Option<u64>,
}
