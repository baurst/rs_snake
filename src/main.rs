extern crate clap;
use clap::{App, Arg};

mod events;
mod screen_buffer;
mod snake;

use crossterm::Result;
use snake::SnakeGame;

fn main() -> Result<()> {
    let matches = App::new("snake")
        .version("0.2.1")
        .author("Author: baurst")
        .about("Classic snake game for your terminal")
        .arg(
            Arg::with_name("easy")
                .short("e")
                .long("easy")
                .help("sets difficulty to easy")
                .takes_value(false)
                .conflicts_with("hard"),
        )
        .arg(
            Arg::with_name("hard")
                .short("h")
                .long("hard")
                .help("sets difficulty to hard")
                .takes_value(false)
                .conflicts_with("easy"),
        )
        .arg(
            Arg::with_name("multiplayer")
                .short("m")
                .long("multi")
                .help("enables multiplayer mode")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("two_key_steering")
                .short("t")
                .long("two_key_steering")
                .help("steer the snakes using two keys only (increased difficulty)")
                .takes_value(false),
        )
        .get_matches();

    let mut target_fps = 8.0;
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

    let is_four_key_steering = !matches.is_present("two_key_steering");

    SnakeGame::new(num_players, target_fps, is_four_key_steering).run()
}
