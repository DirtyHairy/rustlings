mod cmd;
mod definitions;
mod file;
mod sdl_display;

use anyhow::Result;
use clap::{Arg, Command};

use std::path::Path;

fn main() -> Result<()> {
    let mut command = Command::new("rustlings")
        .about("rust by rodent")
        .arg(
            Arg::new("PATH")
                .required(true)
                .help("path to .dat files")
                .index(1),
        )
        .subcommand(Command::new("sprites").about("display lemming sprites"))
        .subcommand(Command::new("tilesets").about("display tilesets"));

    let matches = command.clone().get_matches();

    let path = Path::new(matches.get_one::<String>("PATH").expect("unreachable"));

    match matches.subcommand() {
        Some(("sprites", _)) => cmd::sprites::main(path),
        Some(("tilesets", _)) => cmd::tilesets::main(path),
        _ => drop(command.print_help()),
    }

    Ok(())
}
