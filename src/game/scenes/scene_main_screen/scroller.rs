use anyhow::{Result, bail};
use rustlings::{
    game_data::{
        Bitmap, GameData, Sprite,
        file::{
            main::{FONT_MAIN_SIZE, resolve_main_font_index},
            palette::MENU_SCREEN_PALETTE,
        },
    },
    sdl::{SdlAtlas, SdlAtlasBuilder, with_texture_canvas},
};
use sdl3::{
    pixels::PixelFormat,
    render::{Canvas, RenderTarget, Texture, TextureCreator},
    sys::blendmode::SDL_BLENDMODE_NONE,
};

use crate::state::{SCROLLER_VISIBLE_CHARS, ScrollerPhase, ScrollerState};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 16;

const MSEC_PER_TICK: u64 = 1000 / 30;

const TICKS_PAUSE: u64 = 100;
const TICKS_PER_LETTER: u64 = 4;
const PIXELS_PER_TICK: u32 = 4;

const LEMMING_ANIMATION_FRAMES: usize = 16;

const TEXT: &[u8] = "_________Lemmings_By_DMA_Design______
_______Programming_By_Russell_Kay____
_______Animation_By_Gary_Timmons_____
_______Graphics_By_Scott_Johnston____
_Music_By_Brian_Johnston_&_Tim_Wright_______PC_Music_By_Tony_Williams_____
______Copyright_1991_Psygnosis_Ltd.__
________Rustlings_By_C._Speckner.____
_______Open Source under the GPLv3___
"
.as_bytes();

impl Default for ScrollerState {
    fn default() -> Self {
        Self {
            ticks_in_phase: 0,
            clock_msec: 0,
            phase: ScrollerPhase::Scroll,
            next_char_index: 0,
            current_scroller_frame: 0,
            current_line: [FONT_MAIN_SIZE; SCROLLER_VISIBLE_CHARS],
        }
    }
}

pub struct Scroller<'texture_creator> {
    atlas: SdlAtlas<'texture_creator>,

    texture_scroller: Texture<'texture_creator>,

    index_lemming_left: usize,
    index_lemming_right: usize,
}

impl<'texture_creator> Scroller<'texture_creator> {
    pub fn new<T>(
        texture_creator: &'texture_creator TextureCreator<T>,
        game_data: &GameData,
    ) -> Result<Self> {
        let font_frames: Vec<Bitmap> = game_data
            .font_main
            .frames
            .iter()
            .map(|f| compose_onto(f, &game_data.main_menu.scroller_reel.frames[0]))
            .collect::<Result<_>>()?;

        let mut font = Sprite {
            width: game_data.font_main.width,
            height: game_data.font_main.height,
            frames: font_frames,
        };

        font.add_frame(game_data.main_menu.scroller_reel.frames[0].clone())?;

        let mut atlas_builder = SdlAtlasBuilder::new();

        atlas_builder.add_sprite(&font);

        let index_lemming_left = atlas_builder.add_sprite(&game_data.main_menu.scroller_left);
        let index_lemming_right = atlas_builder.add_sprite(&game_data.main_menu.scroller_right);

        let mut atlas = atlas_builder.build(&MENU_SCREEN_PALETTE, texture_creator)?;
        atlas.apply_blend_mode(SDL_BLENDMODE_NONE);

        let texture_scroller =
            texture_creator.create_texture_target(PixelFormat::RGBA8888, WIDTH, HEIGHT)?;

        Ok(Self {
            atlas,
            texture_scroller,
            index_lemming_left,
            index_lemming_right,
        })
    }

    pub fn tick(&mut self, clock_msec: u64, state: &mut ScrollerState) -> bool {
        tick(state, clock_msec)
    }

    pub fn next_tick_at_msec(&self, state: &ScrollerState) -> u64 {
        next_tick_at_msec(state)
    }

    pub fn render<T: RenderTarget>(
        &mut self,
        state: &ScrollerState,
        canvas: &mut Canvas<T>,
    ) -> Result<()> {
        with_texture_canvas(canvas, &mut self.texture_scroller, |canvas| {
            let shift = if state.phase == ScrollerPhase::Scroll {
                (state.ticks_in_phase % TICKS_PER_LETTER) as i32 * PIXELS_PER_TICK as i32
            } else {
                (TICKS_PER_LETTER as i32) * (PIXELS_PER_TICK as i32)
            };

            for (i, char_index) in state.current_line.into_iter().enumerate() {
                self.atlas.blit(
                    canvas,
                    0,
                    48 + 16 * (i as i32) - shift,
                    0,
                    char_index,
                    false,
                    false,
                )?;
            }

            self.atlas.blit(
                canvas,
                self.index_lemming_left,
                0,
                0,
                state.current_scroller_frame,
                false,
                false,
            )?;

            self.atlas.blit(
                canvas,
                self.index_lemming_right,
                WIDTH as i32 - 48,
                0,
                state.current_scroller_frame,
                false,
                false,
            )?;

            Ok(())
        })
    }

    pub fn texture(&mut self) -> &mut Texture<'texture_creator> {
        &mut self.texture_scroller
    }
}

fn compose_onto(fg: &Bitmap, bg: &Bitmap) -> Result<Bitmap> {
    if fg.width != bg.width || fg.height != bg.height {
        bail!("unable to compose sprites: dimensions do not match");
    }

    let mut data: Vec<u8> = vec![0; fg.data.len()];
    let mut transparency: Vec<bool> = vec![false; fg.transparency.len()];

    for i in 0..data.len() {
        if fg.transparency[i] {
            data[i] = bg.data[i];
            transparency[i] = bg.transparency[i];
        } else {
            data[i] = fg.data[i];
        }
    }

    Ok(Bitmap {
        width: fg.width,
        height: fg.height,
        data,
        transparency,
    })
}

fn tick(state: &mut ScrollerState, clock_msec: u64) -> bool {
    let old_clock_msec = state.clock_msec;
    state.clock_msec = clock_msec;

    let old_ticks = old_clock_msec / MSEC_PER_TICK;
    let new_ticks = clock_msec / MSEC_PER_TICK;
    let mut ticks = new_ticks.saturating_sub(old_ticks);

    let mut redraw = false;
    while ticks > 0 {
        match state.phase {
            ScrollerPhase::Pause => {
                let distribute = ticks.min(TICKS_PAUSE.saturating_sub(state.ticks_in_phase));

                ticks -= distribute;
                state.ticks_in_phase += distribute;

                if state.ticks_in_phase == TICKS_PAUSE {
                    state.phase = ScrollerPhase::Scroll;
                    next_char(state);

                    state.ticks_in_phase = 0;
                }
            }
            ScrollerPhase::Scroll => {
                let ticks_in_phase_for_next_letter =
                    (state.ticks_in_phase + 1).next_multiple_of(TICKS_PER_LETTER);
                let distribute = ticks.min(ticks_in_phase_for_next_letter - state.ticks_in_phase);

                ticks -= distribute;
                state.ticks_in_phase += distribute;
                state.current_scroller_frame =
                    (state.current_scroller_frame + distribute as usize) % LEMMING_ANIMATION_FRAMES;

                if state.ticks_in_phase == ticks_in_phase_for_next_letter {
                    next_char(state);
                }

                redraw = true;
            }
        }
    }

    redraw
}

fn next_tick_at_msec(state: &ScrollerState) -> u64 {
    match state.phase {
        ScrollerPhase::Pause => {
            state.clock_msec + TICKS_PAUSE.saturating_sub(state.ticks_in_phase) * MSEC_PER_TICK
                - (state.clock_msec % MSEC_PER_TICK)
        }
        ScrollerPhase::Scroll => (state.clock_msec + 1).next_multiple_of(MSEC_PER_TICK),
    }
}

fn next_char(state: &mut ScrollerState) {
    let mut next_char: u8 = TEXT[state.next_char_index];
    state.next_char_index = (state.next_char_index + 1) % TEXT.len();

    if next_char == b'_' {
        next_char = b' ';
    } else if next_char == b'\n' {
        state.phase = ScrollerPhase::Pause;
        state.ticks_in_phase = 0;

        return;
    }

    for i in 0..SCROLLER_VISIBLE_CHARS - 1 {
        state.current_line[i] = state.current_line[i + 1];
    }

    state.current_line[SCROLLER_VISIBLE_CHARS - 1] = resolve_font_index(next_char);
}

fn resolve_font_index(c: u8) -> usize {
    if c == b' ' {
        FONT_MAIN_SIZE
    } else {
        resolve_main_font_index(c as char)
    }
}
