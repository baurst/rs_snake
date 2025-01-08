extern crate clap;
use clap::{Arg, ArgAction, Command};

mod events;
mod screen_buffer;
mod snake;

use snake::SnakeGame;
use std::io::Result;

fn main() -> Result<()> {
    let matches = Command::new("snake")
        .version("0.4.0")
        .author("Author: baurst")
        .about("Classic snake game for your terminal")
        .arg(
            Arg::new("easy")
                .short('e')
                .long("easy")
                .help("sets difficulty to easy")
                .action(ArgAction::SetTrue)
                .conflicts_with("hard"),
        )
        .arg(
            Arg::new("hard")
                .short('d')
                .long("hard")
                .help("sets difficulty to hard")
                .action(ArgAction::SetTrue)
                .conflicts_with("easy"),
        )
        .arg(
            Arg::new("multiplayer")
                .short('m')
                .long("multi")
                .help("enables multiplayer mode")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("two_key_steering")
                .short('t')
                .long("two_key_steering")
                .help("steer the snakes using two keys only (increased difficulty)")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let mut target_fps = 8.0;
    if *matches.get_one::<bool>("hard").unwrap_or(&false) {
        target_fps *= 1.5;
    } else if *matches.get_one::<bool>("easy").unwrap_or(&false) {
        target_fps *= 0.7;
    }
    let target_fps = target_fps;

    let mut num_players = 1;

    if *matches.get_one::<bool>("multiplayer").unwrap_or(&false) {
        num_players += 1;
    }
    let num_players = num_players;

    let is_four_key_steering = !matches
        .get_one::<bool>("two_key_steering")
        .unwrap_or(&false);

    SnakeGame::new(num_players, target_fps, is_four_key_steering).run()
}
