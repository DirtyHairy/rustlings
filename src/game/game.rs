use std::{path::Path, rc::Rc};

use anyhow::{Context, Result};
use rustlings::{
    game_data::{GameData, read_game_data},
    sdl::{current_refresh_rate, get_canvas_vsync},
};
use sdl3::{
    Sdl,
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext, WindowPos},
};

use crate::{
    scenes::create_scene,
    stage::{Stage, StopReason},
    state::{self, GameState},
};

pub struct Config {
    pub data_dir: String,
}

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;

fn init_sdl() -> Result<(Sdl, Window)> {
    let sdl_context = sdl3::init()?;
    sdl3::hint::set("SDL_RENDER_VSYNC", "1");
    sdl3::hint::set("SDL_FRAMEBUFFER_ACCELERATION", "1");
    sdl3::hint::set("SDL_VIDEO_MAC_FULLSCREEN_SPACES", "1");
    sdl3::hint::set("SDL_VIDEO_MAC_FULLSCREEN_MENU_VISIBILITY", "1");

    let sdl_video = sdl_context.video()?;

    let mut window = sdl_video
        .window("Rustlings", WINDOW_WIDTH, WINDOW_HEIGHT)
        .hidden()
        .high_pixel_density()
        .resizable()
        .build()?;

    let display_scale = window.display_scale();
    let pixel_density = window.pixel_density();

    let content_scale = if display_scale > 0. && pixel_density > 0. {
        display_scale / pixel_density
    } else {
        1.
    };

    window.set_minimum_size(WINDOW_WIDTH, WINDOW_HEIGHT)?;
    window.set_size(
        (content_scale * WINDOW_WIDTH as f32).round() as u32,
        (content_scale * WINDOW_HEIGHT as f32).round() as u32,
    )?;

    window.set_position(WindowPos::Centered, WindowPos::Centered);
    window.show();
    window.raise();

    sdl_context.mouse().show_cursor(false);

    Ok((sdl_context, window))
}

fn init_canvas(window: Window) -> Result<(Canvas<Window>, TextureCreator<WindowContext>)> {
    let canvas = window.into_canvas();
    println!(
        "got canvas: vsync = {}, driver = {}",
        get_canvas_vsync(&canvas),
        canvas.renderer_name
    );

    if let Some(refresh_rate) = current_refresh_rate(canvas.window()) {
        println!("Refresh rate: {}Hz", refresh_rate as usize);
    } else {
        println!("unable to determine refresh rate");
    }

    let texture_creator = canvas.texture_creator();

    Ok((canvas, texture_creator))
}

pub fn run(config: &Config) -> Result<()> {
    println!("loading game data");
    let game_data: Rc<GameData> = read_game_data(Path::new(&config.data_dir))
        .context("failed to load game data; see --help for options")?
        .into();

    let mut game_state = GameState {
        global: Default::default(),
        scene: state::Scene::MainScreen(None),
    };

    let (sdl_context, window) = init_sdl()?;

    // Creating the canvas consumes the window. However, we need to preserve it in order
    // to spawn a new canvas if we lose context (see below), so we clone it. Looks expensive,
    // but window is just an Arc around a handle, so it is actually cheap.
    let (mut canvas, mut texture_creator) = init_canvas(window.clone())?;

    loop {
        let run_result: StopReason;

        {
            let mut scene = create_scene(
                Rc::clone(&game_data),
                game_state,
                &mut canvas,
                &texture_creator,
            )?;

            let mut stage = Stage::new(
                &sdl_context,
                &mut canvas,
                &texture_creator,
                Rc::clone(&game_data),
            );

            run_result = stage.run(&mut *scene)?;

            if let Some(new_state) = scene.finish() {
                game_state = new_state;
            } else {
                break;
            }
        }

        match run_result {
            StopReason::Quit => {
                break;
            }
            StopReason::RenderReset => {
                println!("render reset");

                // release the renderer before creating a new one; otherwise, SDL will crash
                drop(texture_creator);
                drop(canvas);

                // See above.
                (canvas, texture_creator) = init_canvas(window.clone())?;
            }
            StopReason::NextScene => (),
        }
    }

    println!("shutting down");

    Ok(())
}
