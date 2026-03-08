use std::ffi;

use sdl3::{
    render::{Canvas, RenderTarget, Texture},
    sys::{
        blendmode::SDL_BlendMode,
        init::SDL_IsMainThread,
        render::{SDL_GetRenderVSync, SDL_SetTextureBlendMode},
        video::{SDL_GetCurrentDisplayMode, SDL_GetDisplayForWindow},
    },
    video::{DisplayMode, Window},
};

pub fn get_canvas_vsync(canvas: &Canvas<impl RenderTarget>) -> bool {
    let mut vsync: ffi::c_int = 0;

    unsafe {
        SDL_GetRenderVSync(canvas.raw(), &raw mut vsync);
    }

    vsync != 0
}

pub fn is_main_thread() -> bool {
    unsafe {
        return SDL_IsMainThread();
    }
}

pub fn current_refresh_rate(window: &Window) -> Option<f32> {
    unsafe {
        let display_id = SDL_GetDisplayForWindow(window.raw());
        if display_id.0 == 0 {
            return None;
        }

        let mode_raw = SDL_GetCurrentDisplayMode(display_id);
        if mode_raw.is_null() {
            return None;
        }

        let display_mode = DisplayMode::from_ll(&*mode_raw);

        if display_mode.refresh_rate_numerator > 0 {
            Some(display_mode.refresh_rate)
        } else {
            None
        }
    }
}

pub fn apply_blend_mode(texture: &mut Texture, blend: SDL_BlendMode) -> bool {
    unsafe { SDL_SetTextureBlendMode(texture.raw(), blend.into()) }
}
