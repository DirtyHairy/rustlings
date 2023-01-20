use std::{
    convert::TryFrom,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Context, Error, Ok, Result};

use crate::file;

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

pub fn read_ground(
    path: &Path,
) -> Result<(Vec<file::ground::Content>, Vec<file::tileset::Content>)> {
    let mut ground: Vec<file::ground::Content> = Vec::new();
    let mut tileset: Vec<file::tileset::Content> = Vec::new();

    for i in 0..5 {
        let ground_dat =
            file::ground::read(path, i).context(format!("failed to read ground data set {}", i))?;

        tileset.push(
            file::tileset::read(path, i, &ground_dat)
                .context(format!("failed to read ground data set {}", i))?,
        );

        ground.push(ground_dat);
    }

    Ok((ground, tileset))
}
