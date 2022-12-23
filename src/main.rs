mod cmd;
mod definitions;
mod file;
mod sdl_display;

use anyhow::Result;
use clap::{Arg, ArgMatches, Command};

use std::path::Path;

const ARG_GAME_DATA_PATH: &str = "GAME_DATA";

fn game_data_path(matches: &ArgMatches) -> &Path {
    Path::new(
        matches
            .get_one::<String>(ARG_GAME_DATA_PATH)
            .expect("unreachable"),
    )
}

fn main() -> Result<()> {
    let arg_data_path = Arg::new(ARG_GAME_DATA_PATH)
        .required(true)
        .help("path to lemmings data files")
        .index(1);

    let mut command = Command::new("rustlings")
        .about("rust by rodent")
        .subcommand(
            Command::new("sprites")
                .about("display lemming sprites")
                .arg(arg_data_path.clone()),
        )
        .subcommand(
            Command::new("tilesets")
                .about("display tilesets")
                .arg(arg_data_path.clone()),
        );

    let matches = command.clone().get_matches();

    match matches.subcommand() {
        Some(("sprites", subcommand_matches)) => {
            cmd::sprites::main(game_data_path(subcommand_matches))
        }
        Some(("tilesets", subcommand_matches)) => {
            cmd::tilesets::main(game_data_path(subcommand_matches))
        }
        _ => drop(command.print_help()),
    }

    Ok(())
}
