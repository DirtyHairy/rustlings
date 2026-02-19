use crate::{scenes::SceneLevel, stage::Stage};
use anyhow::Result;
use rustlings::{game_data::read_game_data, sdl3_aux::get_canvas_vsync};
use sdl3::{Sdl, render::Canvas, video::Window};
use std::path::Path;

pub struct Config {
    pub data_dir: String,
}

fn init_sdl() -> Result<(Sdl, Canvas<Window>)> {
    let sdl_context = sdl3::init()?;
    sdl3::hint::set("SDL_RENDER_VSYNC", "1");
    sdl3::hint::set("SDL_FRAMEBUFFER_ACCELERATION", "1");
    sdl3::hint::set("SDL_VIDEO_MAC_FULLSCREEN_SPACES", "1");

    let sdl_video = sdl_context.video()?;

    let mut window = sdl_video
        .window("Rustlings", 640, 480)
        .position_centered()
        .set_flags(sdl3::video::WindowFlags::HIGH_PIXEL_DENSITY)
        .resizable()
        .build()?;
    let _ = window.set_minimum_size(640, 480);

    let canvas = window.into_canvas();
    println!(
        "got canvas: vsync = {}, driver = {}",
        get_canvas_vsync(&canvas),
        canvas.renderer_name
    );

    Ok((sdl_context, canvas))
}

pub fn run(config: &Config) -> Result<()> {
    let game_data = read_game_data(Path::new(&config.data_dir))?;

    let (sdl_context, mut canvas) = init_sdl()?;
    let texture_creator = canvas.texture_creator();

    let mut stage = Stage::new(&sdl_context, &mut canvas, &texture_creator)?;
    let scene_level = SceneLevel::new(&game_data, &texture_creator)?;

    stage.run(&scene_level)
}
