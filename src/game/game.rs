use anyhow::{Ok, Result};
use sdl3::event::Event;

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

pub fn run() -> Result<()> {
    let sdl_context = sdl3::init().expect("unable to initialize SDL3");
    sdl3::hint::set("SDL_RENDER_VSYNC", "1");
    sdl3::hint::set("SDL_FRAMEBUFFER_ACCELERATION", "1");

    let sdl_video = sdl_context.video().expect("failed to initialize video");

    let mut window = sdl_video
        .window("Rustlings", 640, 400)
        .position_centered()
        .set_flags(sdl3::video::WindowFlags::HIGH_PIXEL_DENSITY)
        .resizable()
        .build()
        .unwrap();
    let _ = window.set_minimum_size(640, 400);

    run_event_loop(&sdl_context);

    Ok(())
}
