extern crate clap;
use clap::{App, Arg};

mod snake;

use crossterm::Result;
use snake::SnakeGame;

fn main() -> Result<()> {
    let matches = App::new("snake")
        .version("0.1")
        .author("baurst")
        .about("classic snake game for your terminal")
        .arg(
            Arg::with_name("easy")
                .short("e")
                .long("easy")
                .help("difficulty easy")
                .takes_value(false)
                .conflicts_with("hard"),
        )
        .arg(
            Arg::with_name("hard")
                .short("h")
                .long("hard")
                .help("difficulty hard")
                .takes_value(false)
                .conflicts_with("easy"),
        )
        .arg(
            Arg::with_name("multiplayer")
                .short("m")
                .long("multi")
                .help("enable multiplayer support")
                .takes_value(false),
        )
        .get_matches();

    let mut target_fps = 10.0;
    if matches.is_present("hard") {
        target_fps *= 1.5;
    } else if matches.is_present("easy") {
        target_fps *= 0.7;
    }
    let target_fps = target_fps;

    let mut num_players = 1;

    if matches.is_present("multiplayer") {
        num_players += 1;
    }
    let num_players = num_players;

    SnakeGame::new(num_players, target_fps).run()
}
