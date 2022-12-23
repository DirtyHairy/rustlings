use std::{
    convert::TryFrom,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Error, Result};

pub fn timestamp() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32
}

pub fn create_window(sdl_video: &sdl2::VideoSubsystem) -> Result<sdl2::video::Window> {
    sdl_video
        .window("Rustlings", 1280, 800)
        .position_centered()
        .build()
        .map_err(|e| Error::from(e))
}

pub fn create_pixel_format() -> Result<sdl2::pixels::PixelFormat> {
    sdl2::pixels::PixelFormat::try_from(sdl2::pixels::PixelFormatEnum::RGBA8888)
        .map_err(|s| anyhow!(s))
}
