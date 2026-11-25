use anyhow::{Result, bail};
use clap::{Arg, ArgAction, Command};

use crate::game::Config;

mod code;
mod game;
mod geometry;
mod scene;
mod scenes;
mod stage;
mod state;

fn validate_render_driver(render_driver: &str) -> bool {
    if sdl3::render::drivers()
        .find(|s| s == render_driver)
        .is_some()
    {
        return true;
    }

    let drivers: Vec<String> = sdl3::render::drivers().collect();
    if drivers.is_empty() {
        println!("no drivers available!");
    } else {
        println!("available drivers: {}", drivers.join(", "));
    }

    false
}

fn main() -> Result<()> {
    let command = Command::new("rustlings")
        .about("rustlings is a replacement engine for Lemmings / DOS")
        .arg(
            Arg::new("DATA DIRECTORY")
                .required(false)
                .help("path to data files")
                .default_value(".")
                .index(1),
        )
        .arg(
            Arg::new("video driver")
            .required(false)
            .long("video-driver")
            .short('v')
            .num_args(0..=1)
            .default_missing_value("")
            .help("SDL render driver. Default is auto, leave blank to get a list of possible values")
        )
        .arg(Arg::new("disable vsync")
            .long("disable-vsync")
            .action(ArgAction::SetTrue)
            .help("disable vsync")
        );

    let matches = command.get_matches();

    let render_driver = matches.get_one::<String>("video driver");
    if let Some(driver) = render_driver
        && !validate_render_driver(driver)
    {
        if driver.trim().is_empty() {
            return Ok(());
        } else {
            bail!("invalid render driver {}", driver);
        }
    }

    let config = Config {
        data_dir: matches.get_one::<String>("DATA DIRECTORY").unwrap().into(),
        render_driver: render_driver.cloned(),
        vsync: !matches.get_one::<bool>("disable vsync").copied().unwrap(),
    };

    game::run(&config)
}
