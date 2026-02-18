use std::path::Path;

use anyhow::{Ok, Result};
use rustlings::{
    game_data::read_game_data, sdl_rendering::texture_from_bitmap, sdl3_aux::get_canvas_vsync,
};
use sdl3::{event::Event, rect::Rect};

pub struct Config {
    pub data_dir: String,
}

fn event_is_quit(event: &Event) -> bool {
    match event {
        Event::Quit { .. } => true,
        Event::KeyDown {
            keycode: Some(code),
            ..
        } => *code == sdl3::keyboard::Keycode::Escape,
        _ => false,
    }
}

fn run_event_loop(sdl_context: &sdl3::Sdl) {
    let mut event_pump = sdl_context.event_pump().unwrap();

    loop {
        while let Some(event) = event_pump.wait_event_timeout(50) {
            if event_is_quit(&event) {
                return;
            }
        }
    }
}

pub fn run(config: &Config) -> Result<()> {
    let game_data = read_game_data(Path::new(&config.data_dir))?;

    let sdl_context = sdl3::init().expect("unable to initialize SDL3");
    sdl3::hint::set("SDL_RENDER_VSYNC", "1");
    sdl3::hint::set("SDL_FRAMEBUFFER_ACCELERATION", "1");

    let sdl_video = sdl_context.video().expect("failed to initialize video");

    let mut window = sdl_video
        .window("Rustlings", 640, 480)
        .position_centered()
        .set_flags(sdl3::video::WindowFlags::HIGH_PIXEL_DENSITY)
        .resizable()
        .build()
        .unwrap();
    let _ = window.set_minimum_size(640, 400);

    let mut canvas = window.into_canvas();
    println!(
        "got canvas: vsync = {}, driver = {}",
        get_canvas_vsync(&canvas),
        canvas.renderer_name
    );

    let texture_creator = canvas.texture_creator();

    let skill_panel_texture = texture_from_bitmap(
        &game_data.skill_panel,
        &game_data.resolve_skill_panel_palette(0),
        &texture_creator,
    )?;

    canvas.clear();

    let (canvas_width, canvas_height) = canvas.output_size()?;
    canvas.copy(
        &skill_panel_texture,
        Rect::new(
            0,
            0,
            skill_panel_texture.width(),
            skill_panel_texture.height(),
        ),
        Rect::new(
            0,
            0,
            (skill_panel_texture.width() * canvas_width * 2) / 640,
            (((skill_panel_texture.height() * canvas_height * 2) / 480) as f32 * 1.2) as u32,
        ),
    )?;

    canvas.present();

    run_event_loop(&sdl_context);

    Ok(())
}
