use crate::state::FadeState;

pub struct FadeController {
    state: FadeState,

    clock_offset_musec: u64,
    fadein_duration_musec: u64,
    fadeout_duration_musec: u64,
}

pub const FADEIN_MUSEC_MENU: u64 = 500000;
pub const FADEOUT_MUSEC_MENU: u64 = 500000;

pub const FADEIN_MUSEC_LEVEL: u64 = 1000000;
pub const FADEOUT_MUSEC_LEVEL: u64 = 2000000;

impl FadeController {
    pub fn new(
        fade_state: Option<FadeState>,
        fadein_duration_musec: u64,
        fadeout_duration_musec: u64,
    ) -> Self {
        let state = fade_state.unwrap_or_default();

        Self {
            clock_offset_musec: state.clock_musec,
            fadein_duration_musec,
            fadeout_duration_musec,
            state,
        }
    }

    pub fn tick(&mut self, clock_musec: u64) {
        self.state.clock_musec = clock_musec + self.clock_offset_musec;
    }

    pub fn next_tick_since_creation_at_musec(&self) -> Option<u64> {
        if let Some(fadeout_started_at_musec) = self.state.fadeout_started_at_musec {
            let opacity = self.opacity();
            if opacity == 0 {
                return None;
            }

            let fadein_duration_musec = fadeout_started_at_musec.min(self.fadein_duration_musec);
            let fadeout_duration_musec = (((fadein_duration_musec * 255)
                / self.fadein_duration_musec)
                * self.fadeout_duration_musec)
                / 255;

            return Some(
                (fadeout_started_at_musec + fadeout_duration_musec)
                    .saturating_sub((opacity as u64 - 1) * self.fadeout_duration_musec / 255)
                    .saturating_sub(self.clock_offset_musec),
            );
        }

        if self.state.clock_musec < self.fadein_duration_musec {
            return Some(
                (((self.opacity() as u64 + 1) * self.fadein_duration_musec) / 255)
                    .saturating_sub(self.clock_offset_musec),
            );
        }

        None
    }

    pub fn opacity(&self) -> u8 {
        if self.state.clock_musec >= self.fadein_duration_musec
            && self.state.fadeout_started_at_musec.is_none()
        {
            return 255;
        }

        if let Some(fadeout_started_at_musec) = self.state.fadeout_started_at_musec {
            let fadein_duration_musec = fadeout_started_at_musec.min(self.fadein_duration_musec);

            return ((fadein_duration_musec * 255) / self.fadein_duration_musec).saturating_sub(
                ((self.state.clock_musec - fadeout_started_at_musec) * 255)
                    / self.fadeout_duration_musec,
            ) as u8;
        }

        if self.state.clock_musec < self.fadein_duration_musec {
            return ((self.state.clock_musec * 255) / self.fadein_duration_musec) as u8;
        }

        255
    }

    pub fn start_fadeout(&mut self) {
        if self.state.fadeout_started_at_musec.is_none() {
            self.state.fadeout_started_at_musec = Some(self.state.clock_musec);
        }
    }

    pub fn fadeout_started(&self) -> bool {
        self.state.fadeout_started_at_musec.is_some()
    }

    pub fn fadeout_done(&self) -> bool {
        match self.state.fadeout_started_at_musec {
            Some(_) => self.opacity() == 0,
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
