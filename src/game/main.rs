use anyhow::Result;
use clap::{Arg, Command};

use crate::game::Config;

mod code;
mod game;
mod geometry;
mod scene;
mod scenes;
mod stage;
mod state;

fn main() -> Result<()> {
    let command = Command::new("rustlings")
        .about("rustlings is a replacement engine for Lemmings / DOS")
        .arg(
            Arg::new("DATA DIRECTORY")
                .required(false)
                .help("path to data files")
                .default_value(".")
                .index(1),
        );

    let matches = command.get_matches();

    let config = Config {
        data_dir: matches.get_one::<String>("DATA DIRECTORY").unwrap().into(),
    };

    game::run(&config)
}
