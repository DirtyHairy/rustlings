use std::ffi;

use sdl3::{
    render::{Canvas, RenderTarget},
    video::{DisplayMode, Window},
};

pub const SDL_EVENT_RENDER_DEVICE_LOST: u32 = 0x2002;
pub const SDL_EVENT_WINDOW_ENTER_FULLSCREEN: u32 = 0x0217;
pub const SDL_EVENT_WINDOW_LEAVE_FULLSCREEN: u32 = 0x0218;

pub fn get_canvas_vsync(canvas: &Canvas<impl RenderTarget>) -> bool {
    let mut vsync: ffi::c_int = 0;

    unsafe {
        sdl3::sys::render::SDL_GetRenderVSync(canvas.raw(), &raw mut vsync);
    }

    vsync != 0
}

pub fn is_main_thread() -> bool {
    unsafe {
        return sdl3::sys::init::SDL_IsMainThread();
    }
}

pub fn current_refresh_rate(window: &Window) -> Option<f32> {
    unsafe {
        let display_id = sdl3::sys::video::SDL_GetDisplayForWindow(window.raw());
        if display_id.0 == 0 {
            return None;
        }

        let mode_raw = sdl3::sys::video::SDL_GetCurrentDisplayMode(display_id);
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
