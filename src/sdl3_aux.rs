use std::ffi;

use sdl3::render::{Canvas, RenderTarget};

pub const SDL_EVENT_RENDER_DEVICE_LOST: u32 = 0x2002;

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
