use crate::state::FadeState;

pub struct FadeController {
    state: FadeState,
    clock_offset_msec: u64,
}

const FADE_MSEC: u64 = 750;

impl FadeController {
    pub fn new(fade_state: Option<FadeState>) -> Self {
        let state = fade_state.unwrap_or_default();

        Self {
            clock_offset_msec: state.clock_msec,
            state,
        }
    }

    pub fn tick(&mut self, clock_msec: u64) {
        self.state.clock_msec = clock_msec + self.clock_offset_msec;
    }

    pub fn next_tick_at_msec(&self) -> Option<u64> {
        if let Some(fade_out_started_at_msec) = self.state.fade_out_started_at_msec {
            let opacity = self.opacity();
            if opacity == 0 {
                return None;
            }

            let fade_in_duration_msec = fade_out_started_at_msec.min(FADE_MSEC);

            return Some(
                (fade_out_started_at_msec + fade_in_duration_msec)
                    .saturating_sub((opacity as u64 - 1) * FADE_MSEC / 255)
                    .saturating_sub(self.clock_offset_msec),
            );
        }

        if self.state.clock_msec < FADE_MSEC {
            return Some(
                (((self.opacity() as u64 + 1) * FADE_MSEC) / 255)
                    .saturating_sub(self.clock_offset_msec),
            );
        }

        None
    }

    pub fn opacity(&self) -> u8 {
        if self.state.clock_msec >= FADE_MSEC && self.state.fade_out_started_at_msec.is_none() {
            return 255;
        }

        if let Some(fade_out_started_at_msec) = self.state.fade_out_started_at_msec {
            let fade_in_duration_msec = fade_out_started_at_msec.min(FADE_MSEC);
            let fade_out_duration_msec = self
                .state
                .clock_msec
                .saturating_sub(fade_out_started_at_msec)
                .min(fade_in_duration_msec);

            return (((fade_in_duration_msec - fade_out_duration_msec) * 255) / FADE_MSEC) as u8;
        }

        if self.state.clock_msec < FADE_MSEC {
            return ((self.state.clock_msec * 255) / FADE_MSEC).min(255) as u8;
        }

        255
    }

    pub fn start_fade_out(&mut self) {
        if self.state.fade_out_started_at_msec.is_none() {
            self.state.fade_out_started_at_msec = Some(self.state.clock_msec);
        }
    }

    pub fn fade_out_started(&self) -> bool {
        self.state.fade_out_started_at_msec.is_some()
    }

    pub fn fade_out_done(&self) -> bool {
        match self.state.fade_out_started_at_msec {
            Some(fade_out_started_msec) => {
                self.state.clock_msec > fade_out_started_msec + fade_out_started_msec.min(FADE_MSEC)
            }
            None => false,
        }
    }

    pub fn finish(self) -> FadeState {
        self.state
    }
}
