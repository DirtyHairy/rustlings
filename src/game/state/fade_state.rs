#[derive(Clone, Default)]
pub struct FadeState {
    pub clock_musec: u64,
    pub fadeout_started_at_musec: Option<u64>,
}
