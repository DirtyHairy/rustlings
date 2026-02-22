use crate::{
    scenes::create_scene,
    stage::{RunResult, Stage},
    state::GameState,
};
use anyhow::Result;
use rustlings::{
    game_data::{GameData, read_game_data},
    sdl3_aux::get_canvas_vsync,
};
use sdl3::{
    Sdl,
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
};
use std::{path::Path, rc::Rc};

pub struct Config {
    pub data_dir: String,
}

fn init_sdl() -> Result<(Sdl, Window)> {
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

    Ok((sdl_context, window))
}

fn init_canvas(window: Window) -> Result<(Canvas<Window>, TextureCreator<WindowContext>)> {
    let canvas = window.into_canvas();
    println!(
        "got canvas: vsync = {}, driver = {}",
        get_canvas_vsync(&canvas),
        canvas.renderer_name
    );

    let texture_creator = canvas.texture_creator();

    Ok((canvas, texture_creator))
}

pub fn run(config: &Config) -> Result<()> {
    let game_data: Rc<GameData> = read_game_data(Path::new(&config.data_dir))?.into();
    let mut game_state: GameState = Default::default();
    let mut scene_state = Default::default();

    let (sdl_context, window) = init_sdl()?;
    let (mut canvas, mut texture_creator) = init_canvas(window.clone())?;

    loop {
        let run_result: RunResult;

        {
            let mut stage = Stage::new(&sdl_context, &mut canvas, &texture_creator)?;
            let scene = create_scene(
                game_data.clone(),
                &mut game_state,
                &scene_state,
                &texture_creator,
            )?;

            run_result = stage.run(&*scene)?;
            scene_state = scene.get_scene_state();
        }

        match run_result {
            RunResult::Quit => {
                println!("shutting down");
                break;
            }
            RunResult::RenderReset => {
                println!("render reset");

                // release the renderer before creating a new one; otherwise, SDL will crash
                drop(texture_creator);
                drop(canvas);

                (canvas, texture_creator) = init_canvas(window.clone())?;
            }
        }
    }

    Ok(())
}
