use crate::state::FadeState;

pub struct FadeController {
    state: FadeState,

    clock_offset_msec: u64,
    fadein_duration_msec: u64,
    fadeout_duration_msec: u64,
}

pub const FADEIN_MSEC_MENU: u64 = 500;
pub const FADEOUT_MSEC_MENU: u64 = 500;

pub const FADEIN_MSEC_LEVEL: u64 = 1000;
pub const FADEOUT_MSEC_LEVEL: u64 = 2500;

impl FadeController {
    pub fn new(
        fade_state: Option<FadeState>,
        fadein_duration_msec: u64,
        fadeout_duration_msec: u64,
    ) -> Self {
        let state = fade_state.unwrap_or_default();

        Self {
            clock_offset_msec: state.clock_msec,
            fadein_duration_msec,
            fadeout_duration_msec,
            state,
        }
    }

    pub fn tick(&mut self, clock_msec: u64) {
        self.state.clock_msec = clock_msec + self.clock_offset_msec;
    }

    pub fn next_tick_at_msec(&self) -> Option<u64> {
        if let Some(fadeout_started_at_msec) = self.state.fadeout_started_at_msec {
            let opacity = self.opacity();
            if opacity == 0 {
                return None;
            }

            let fadein_duration_msec = fadeout_started_at_msec.min(self.fadein_duration_msec);
            let fadeout_duration_msec = (((fadein_duration_msec * 255)
                / self.fadein_duration_msec)
                * self.fadeout_duration_msec)
                / 255;

            return Some(
                (fadeout_started_at_msec + fadeout_duration_msec)
                    .saturating_sub((opacity as u64 - 1) * self.fadeout_duration_msec / 255)
                    .saturating_sub(self.clock_offset_msec),
            );
        }

        if self.state.clock_msec < self.fadein_duration_msec {
            return Some(
                (((self.opacity() as u64 + 1) * self.fadein_duration_msec) / 255)
                    .saturating_sub(self.clock_offset_msec),
            );
        }

        None
    }

    pub fn opacity(&self) -> u8 {
        if self.state.clock_msec >= self.fadein_duration_msec
            && self.state.fadeout_started_at_msec.is_none()
        {
            return 255;
        }

        if let Some(fadeout_started_at_msec) = self.state.fadeout_started_at_msec {
            let fadein_duration_msec = fadeout_started_at_msec.min(self.fadein_duration_msec);

            return ((fadein_duration_msec * 255) / self.fadein_duration_msec).saturating_sub(
                ((self.state.clock_msec - fadeout_started_at_msec) * 255)
                    / self.fadeout_duration_msec,
            ) as u8;
        }

        if self.state.clock_msec < self.fadein_duration_msec {
            return ((self.state.clock_msec * 255) / self.fadein_duration_msec) as u8;
        }

        255
    }

    pub fn start_fadeout(&mut self) {
        if self.state.fadeout_started_at_msec.is_none() {
            self.state.fadeout_started_at_msec = Some(self.state.clock_msec);
        }
    }

    pub fn fadeout_started(&self) -> bool {
        self.state.fadeout_started_at_msec.is_some()
    }

    pub fn fadeout_done(&self) -> bool {
        match self.state.fadeout_started_at_msec {
            Some(fadeout_started_at_msec) => {
                let fadein_duration_msec = fadeout_started_at_msec.min(self.fadein_duration_msec);
                let fadeout_duration_msec = (((fadein_duration_msec * 255)
                    / self.fadein_duration_msec)
                    * self.fadeout_duration_msec)
                    / 255;

                self.state.clock_msec > fadeout_started_at_msec + fadeout_duration_msec
            }
            None => false,
        }
    }

    pub fn finish(self) -> FadeState {
        self.state
    }
}

#[path = "./fade_controller_test.rs"]
#[cfg(test)]
mod test;
