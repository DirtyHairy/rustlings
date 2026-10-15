use anyhow::{Error, Result};

pub fn create_window(sdl_video: &sdl3::VideoSubsystem, hidpi: bool) -> Result<sdl3::video::Window> {
    let mut builder = sdl_video.window("Rustlings", 1280, 800);

    if hidpi {
        builder.set_flags(sdl3::video::WindowFlags::HIGH_PIXEL_DENSITY);
    }

    builder.position_centered().build().map_err(Error::from)
}
