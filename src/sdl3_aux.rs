use std::ffi;

use sdl3::render::{Canvas, RenderTarget};

pub fn get_canvas_vsync(canvas: &Canvas<impl RenderTarget>) -> bool {
    let mut vsync: ffi::c_int = 0;

    unsafe {
        sdl3::sys::render::SDL_GetRenderVSync(canvas.raw(), &raw mut vsync);
    }

    vsync != 0
}
