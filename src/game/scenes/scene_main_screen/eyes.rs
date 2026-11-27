use std::array;

use anyhow::Result;
use rustlings::{
    game_data::{GameData, file::palette::MENU_SCREEN_PALETTE},
    sdl::{SdlAtlas, SdlAtlasBuilder},
};
use sdl3::{
    render::{Canvas, RenderTarget, TextureCreator},
    sys::blendmode::SDL_BLENDMODE_NONE,
};

use crate::{
    state::{EYE_COUNT, EyesState},
    timing::RenderConfiguration,
};

struct EyeSpec {
    x: i32,
    y: i32,
    counter_reset: isize,
    sprite_index: usize,
}

const TICKS_PER_SECOND: u32 = 20;
const FRAMES_PER_EYE: isize = 8;

const EYE_SPEC: &[EyeSpec] = &[
    EyeSpec {
        x: 0x20,
        y: 0x31,
        counter_reset: 0x190 >> 1,
        sprite_index: 0,
    },
    EyeSpec {
        x: 0x100,
        y: 0x2f,
        counter_reset: 0x1b8 >> 1,
        sprite_index: 1,
    },
    EyeSpec {
        x: 0x1f0,
        y: 0x28,
        counter_reset: 0x1e0 >> 1,
        sprite_index: 2,
    },
    EyeSpec {
        x: 0x6a,
        y: 0x84,
        counter_reset: 0x208 >> 1,
        sprite_index: 3,
    },
    EyeSpec {
        x: 0xeb,
        y: 0x84,
        counter_reset: 0x230 >> 1,
        sprite_index: 5,
    },
];

impl Default for EyesState {
    fn default() -> Self {
        Self {
            eye_counters: array::from_fn::<isize, EYE_COUNT, _>(|i| i as isize * (0x28 >> 1)),
            tick_duration_musec: 0,
            clock_musec: 0,
        }
    }
}

pub struct Eyes<'texture_creator> {
    atlas: SdlAtlas<'texture_creator>,
    clock_offset_musec: u64,
}

impl<'texture_creator> Eyes<'texture_creator> {
    pub fn new<T>(
        texture_creator: &'texture_creator TextureCreator<T>,
        game_data: &GameData,
        state: &mut EyesState,
        render_configuration: &RenderConfiguration,
    ) -> Result<Self> {
        let mut atlas_bulder = SdlAtlasBuilder::new();

        for eye in &game_data.main_menu.eyes {
            atlas_bulder.add_sprite(eye);
        }

        let mut atlas = atlas_bulder.build(&MENU_SCREEN_PALETTE, texture_creator)?;
        atlas.apply_blend_mode(SDL_BLENDMODE_NONE);

        state.update_tick_duration_musec(
            render_configuration.clamped_tick_duration_musec(TICKS_PER_SECOND),
        );

        Ok(Self {
            atlas,
            clock_offset_musec: state.clock_musec,
        })
    }

    pub fn tick(&self, clock_musec: u64, state: &mut EyesState) -> bool {
        tick(clock_musec + self.clock_offset_musec, state, EYE_SPEC)
    }

    pub fn next_tick_at_musec(&self, state: &EyesState) -> u64 {
        next_tick_at_musec(state, EYE_SPEC).saturating_sub(self.clock_offset_musec)
    }

    pub fn render<T: RenderTarget>(&self, state: &EyesState, canvas: &mut Canvas<T>) -> Result<()> {
        for (spec, counter) in EYE_SPEC.iter().zip(state.eye_counters) {
            let frame = if counter >= 0 {
                FRAMES_PER_EYE - 1
            } else {
                -counter - 1
            };

            self.atlas.blit(
                canvas,
                spec.sprite_index,
                spec.x,
                spec.y,
                frame as usize,
                false,
                false,
            )?;
        }

        Ok(())
    }
}

fn tick(clock_musec: u64, state: &mut EyesState, eye_spec: &[EyeSpec]) -> bool {
    let old_clock_musec = state.clock_musec;
    state.clock_musec = clock_musec;

    let ticks_old = old_clock_musec / state.tick_duration_musec;
    let ticks_new = clock_musec / state.tick_duration_musec;

    let ticks = ticks_new.strict_sub(ticks_old) as isize;
    if ticks == 0 {
        return false;
    }

    let mut redraw = false;
    for (spec, counter) in eye_spec.iter().zip(state.eye_counters.iter_mut()) {
        *counter -= ticks;

        if *counter < 0 {
            redraw = true;
        }

        if *counter < -FRAMES_PER_EYE {
            *counter = spec.counter_reset
                - ((-*counter - FRAMES_PER_EYE - 1) % (spec.counter_reset + FRAMES_PER_EYE + 1));
        }
    }

    redraw
}

fn next_tick_at_musec(state: &EyesState, eye_spec: &[EyeSpec]) -> u64 {
    let min_counter = state.eye_counters[0..eye_spec.len()]
        .iter()
        .copied()
        .min()
        .unwrap();

    let delta = if min_counter <= 0 {
        (state.clock_musec + 1).next_multiple_of(state.tick_duration_musec) - state.clock_musec
    } else {
        (1 + min_counter as u64) * state.tick_duration_musec
            - (state.clock_musec % state.tick_duration_musec)
    };

    state.clock_musec + delta
}

#[cfg(test)]
mod test {
    use crate::{
        scenes::scene_main_screen::eyes::{EyeSpec, next_tick_at_musec, tick},
        state::EyesState,
    };

    const EYE_SPEC_FIXTURE: &[EyeSpec] = &[
        EyeSpec {
            x: 0,
            y: 0,
            counter_reset: 50,
            sprite_index: 0,
        },
        EyeSpec {
            x: 0,
            y: 0,
            counter_reset: 100,
            sprite_index: 1,
        },
        EyeSpec {
            x: 0,
            y: 0,
            counter_reset: 150,
            sprite_index: 1,
        },
    ];

    const TEST_TICK_DURATION_MUSEC: u64 = 1000000 / 60;

    fn state_fixture() -> EyesState {
        EyesState {
            tick_duration_musec: TEST_TICK_DURATION_MUSEC,
            ..Default::default()
        }
    }

    #[test]
    fn counters_are_initialized_to_multiples_of_20() {
        let state: EyesState = Default::default();

        for (i, counter) in state.eye_counters.into_iter().enumerate() {
            assert_eq!(counter, i as isize * 20);
        }
    }

    #[test]
    fn a_tick_decrements_all_counters() {
        let mut state = state_fixture();

        tick(TEST_TICK_DURATION_MUSEC, &mut state, EYE_SPEC_FIXTURE);

        assert_eq!(state.eye_counters[0..=3], [-1, 19, 39, 60]);
    }

    #[test]
    fn multiple_ticks_are_aggregated() {
        let mut state = state_fixture();

        tick(3 * TEST_TICK_DURATION_MUSEC, &mut state, EYE_SPEC_FIXTURE);

        assert_eq!(state.eye_counters[0], -3);
    }

    #[test]
    fn counter_reaches_minus_8() {
        let mut state = state_fixture();

        tick(8 * TEST_TICK_DURATION_MUSEC, &mut state, EYE_SPEC_FIXTURE);

        assert_eq!(state.eye_counters[0], -8);
    }

    #[test]
    fn counter_wraps_at_minus_9() {
        let mut state = state_fixture();

        tick(9 * TEST_TICK_DURATION_MUSEC, &mut state, EYE_SPEC_FIXTURE);

        assert_eq!(state.eye_counters[0], 50);
    }

    #[test]
    fn counter_handles_multiple_wraps() {
        let mut state = state_fixture();

        tick(75 * TEST_TICK_DURATION_MUSEC, &mut state, EYE_SPEC_FIXTURE);

        assert_eq!(state.eye_counters[0], 43);
    }

    #[test]
    fn next_tick_at_msec_is_one_tick_as_long_as_one_counter_is_zero_or_below() {
        let mut state = state_fixture();

        assert_eq!(
            next_tick_at_musec(&state, EYE_SPEC_FIXTURE),
            TEST_TICK_DURATION_MUSEC
        );

        tick(21 * TEST_TICK_DURATION_MUSEC, &mut state, EYE_SPEC_FIXTURE);

        assert_eq!(
            next_tick_at_musec(&state, EYE_SPEC_FIXTURE),
            22 * TEST_TICK_DURATION_MUSEC
        );
    }

    #[test]
    fn next_tick_at_msec_returns_the_time_until_the_first_counter_becomes_negative() {
        let mut state = state_fixture();

        tick(9 * TEST_TICK_DURATION_MUSEC, &mut state, EYE_SPEC_FIXTURE);

        assert_eq!(
            next_tick_at_musec(&state, EYE_SPEC_FIXTURE),
            21 * TEST_TICK_DURATION_MUSEC
        );
    }

    #[test]
    fn next_tick_at_msec_accounts_for_partial_ticks_while_counter_is_negative() {
        let mut state = state_fixture();

        tick(
            8 * TEST_TICK_DURATION_MUSEC + 10,
            &mut state,
            EYE_SPEC_FIXTURE,
        );

        assert_eq!(
            next_tick_at_musec(&state, EYE_SPEC_FIXTURE),
            9 * TEST_TICK_DURATION_MUSEC
        );
    }

    #[test]
    fn next_tick_at_msec_accounts_for_partial_ticks_while_counter_is_positive() {
        let mut state = state_fixture();

        tick(
            9 * TEST_TICK_DURATION_MUSEC + 10,
            &mut state,
            EYE_SPEC_FIXTURE,
        );

        assert_eq!(
            next_tick_at_musec(&state, EYE_SPEC_FIXTURE),
            21 * TEST_TICK_DURATION_MUSEC
        );
    }
}
