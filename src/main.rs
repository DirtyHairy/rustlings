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

fn main() -> Result<()> {
    let sdl_context = sdl3::init().expect("unable to initialize SDL3");
    let sdl_video = sdl_context.video().expect("failed to initialize video");

    let mut window = sdl_video
        .window("Rustlings", 640, 400)
        .position_centered()
        .set_flags(sdl3::video::WindowFlags::HIGH_PIXEL_DENSITY)
        .resizable()
        .build()
        .unwrap();
    let _ = window.set_minimum_size(640, 400);

    let mut event_pump = sdl_context.event_pump().unwrap();
    for event in event_pump.wait_iter() {
        if event_is_quit(&event) {
            break;
        }
    }

    Ok(())
}
