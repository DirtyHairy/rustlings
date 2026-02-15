mod cmd;
mod sdl_sprite;

use anyhow::Result;
use clap::{Arg, ArgMatches, Command};

use std::path::Path;

const ARG_GAME_DATA_PATH: &str = "GAME_DATA_PATH";
const ARG_DAT_FILE_PATH: &str = "DAT_FILE";
const ARG_DESTINATION_PATH: &str = "DESTINATION_PATH";

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
        )
        .subcommand(
            Command::new("decode-dat")
                .about("decode dat file into sections")
                .arg(
                    Arg::new(ARG_DAT_FILE_PATH)
                        .help("dat file to decode")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("view-levels")
                .about("view levels in dat file")
                .arg(
                    Arg::new(ARG_GAME_DATA_PATH)
                        .required(true)
                        .help("path to lemmings data files")
                        .index(1),
                )
                .arg(
                    Arg::new("level")
                        .help("start with level")
                        .short('l')
                        .long("level"),
                ),
        )
        .subcommand(
            Command::new("decode-levels")
                .about("decode level data into files")
                .arg(
                    Arg::new(ARG_GAME_DATA_PATH)
                        .required(true)
                        .help("path to lemmings data files")
                        .index(1),
                )
                .arg(
                    Arg::new(ARG_DESTINATION_PATH)
                        .required(true)
                        .help("destination path")
                        .index(2),
                ),
        );

    let matches = command.clone().get_matches();

    match matches.subcommand() {
        Some(("sprites", subcommand_matches)) => {
            cmd::sprites::main(game_data_path(subcommand_matches))
        }

        Some(("tilesets", subcommand_matches)) => {
            cmd::tilesets::main(game_data_path(subcommand_matches))
        }

        Some(("decode-dat", subcommand_matches)) => cmd::decode_dat::main(
            subcommand_matches
                .get_one::<String>(ARG_DAT_FILE_PATH)
                .expect("unreachable"),
        ),

        Some(("view-levels", subcommand_matches)) => cmd::view_levels::main(
            game_data_path(subcommand_matches),
            subcommand_matches.get_one::<String>("level"),
        ),

        Some(("decode-levels", subcommand_matches)) => cmd::decode_levels::main(
            game_data_path(subcommand_matches),
            subcommand_matches
                .get_one::<String>(ARG_DESTINATION_PATH)
                .expect("unreachable"),
        ),

        _ => command.print_help().map_err(anyhow::Error::from),
    }
}
