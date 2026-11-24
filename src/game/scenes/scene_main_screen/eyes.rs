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

use crate::state::{EYE_COUNT, EyesState};

struct EyeSpec {
    x: i32,
    y: i32,
    counter_reset: isize,
    sprite_index: usize,
}

const MUSEC_PER_TICK: u64 = 1000000 / 20;
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
            clock_musec: 0,
        }
    }
}

pub struct Eyes<'texture_creator> {
    atlas: SdlAtlas<'texture_creator>,
}

impl<'texture_creator> Eyes<'texture_creator> {
    pub fn new<T>(
        texture_creator: &'texture_creator TextureCreator<T>,
        game_data: &GameData,
    ) -> Result<Self> {
        let mut atlas_bulder = SdlAtlasBuilder::new();

        for eye in &game_data.main_menu.eyes {
            atlas_bulder.add_sprite(eye);
        }

        let mut atlas = atlas_bulder.build(&MENU_SCREEN_PALETTE, texture_creator)?;
        atlas.apply_blend_mode(SDL_BLENDMODE_NONE);

        Ok(Self { atlas })
    }

    pub fn tick(&self, clock_musec: u64, state: &mut EyesState) -> bool {
        tick(clock_musec, state)
    }

    pub fn next_tick_at_musec(&self, state: &EyesState) -> u64 {
        next_tick_at_musec(state)
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

pub fn tick(clock_musec: u64, state: &mut EyesState) -> bool {
    let old_clock_musec = state.clock_musec;
    state.clock_musec = clock_musec;

    let ticks_old = old_clock_musec / MUSEC_PER_TICK;
    let ticks_new = clock_musec / MUSEC_PER_TICK;

    let ticks = ticks_new.strict_sub(ticks_old) as isize;
    if ticks == 0 {
        return false;
    }

    let mut redraw = false;
    for (spec, counter) in EYE_SPEC.iter().zip(state.eye_counters.iter_mut()) {
        *counter -= ticks;

        if *counter < 0 {
            redraw = true;
        }

        if *counter < -FRAMES_PER_EYE {
            *counter = (spec.counter_reset - (-*counter - FRAMES_PER_EYE - 1))
                % (spec.counter_reset + FRAMES_PER_EYE + 1);
        }
    }

    redraw
}

pub fn next_tick_at_musec(state: &EyesState) -> u64 {
    let min_counter = state.eye_counters[0..EYE_SPEC.len()]
        .iter()
        .copied()
        .min()
        .unwrap();

    let delta = if min_counter <= 0 {
        (state.clock_musec + 1).next_multiple_of(MUSEC_PER_TICK) - state.clock_musec
    } else {
        (1 + min_counter as u64) * MUSEC_PER_TICK - (state.clock_musec % MUSEC_PER_TICK)
    };

    state.clock_musec + delta
}
