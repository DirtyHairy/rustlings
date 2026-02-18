use anyhow::Result;
use clap::{Arg, Command};

use crate::game::Config;

mod game;

fn main() -> Result<()> {
    let command = Command::new("rustlings")
        .about("rustlings is a replacement engine for Lemmings / DOS")
        .arg(
            Arg::new("DATA DIRECTORY")
                .required(true)
                .help("path to data files")
                .index(1),
        );

    let matches = command.get_matches();

    let config = Config {
        data_dir: matches
            .get_one::<String>("DATA DIRECTORY")
            .expect("unreachable")
            .clone(),
    };

    game::run(&config)
}
