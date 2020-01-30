extern crate clap;
use clap::{App, Arg};

mod snake;

use snake::{
    add_snake_to_buffer, get_random_food_pos, move_snake, send_events, snake_item_collision,
    Coordinate, GameContent, KeyEventQueue, ScreenBuffer,
};

use std::thread;
use std::time::Duration;

use std::io::stdout;

use crossterm::{
    cursor::{self},
    event::{KeyCode, KeyEvent},
    terminal::{self, disable_raw_mode, enable_raw_mode},
    ExecutableCommand, Result,
};

fn find_matches<T: PartialEq + Copy>(look_in: &[T], look_for: &[T]) -> Vec<T> {
    let mut found: Vec<T> = vec![];
    for a in look_for {
        for b in look_in {
            if a == b {
                found.push(*b);
            }
        }
    }
    return found;
}

#[derive(PartialEq, Clone, Debug)]
struct Snake {
    body_pos: Vec<Coordinate>,
    // 0: up, 1: right, 2: down, 3: left
    direction: i64,
}

impl Snake {
    fn new(player_idx: usize) -> Snake {
        let snake_body = vec![
            Coordinate {
                row: 18,
                col: 10 + player_idx * 5,
            },
            Coordinate {
                row: 19,
                col: 10 + player_idx * 5,
            },
            Coordinate {
                row: 20,
                col: 10 + player_idx * 5,
            },
        ];
        Snake {
            body_pos: snake_body,
            direction: 0,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
struct Player {
    score: usize,
    left_key: crossterm::event::KeyEvent,
    right_key: crossterm::event::KeyEvent,
    snake: Snake,
    player_idx: usize,
}

impl Player {
    fn new(
        left_key: crossterm::event::KeyEvent,
        right_key: crossterm::event::KeyEvent,
        player_idx: usize,
    ) -> Player {
        Player {
            snake: Snake::new(player_idx),
            left_key: left_key,
            right_key: right_key,
            player_idx: player_idx,
            score: 0,
        }
    }
}

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

    let mut target_fps = 12.0;
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

    let event_queue = KeyEventQueue::new();
    let thread_event_queue = event_queue.clone();

    // launch seperate thread to deal with keyboard input
    thread::spawn(move || send_events(thread_event_queue));

    let mut stdout = stdout();
    enable_raw_mode()?;
    stdout.execute(cursor::Hide)?;

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    let screen_width = 80;
    let screen_height = 40;

    let mut must_exit = false;

    let mut screen_buffer = ScreenBuffer::new(screen_width, screen_height, GameContent::Empty);

    // clear screen
    screen_buffer.set_all(GameContent::Empty);
    screen_buffer.set_centered_text_at_row(screen_height / 2 - 4, "SNAKE");
    screen_buffer.set_centered_text_at_row(screen_height / 2 - 2, "press ESC to stop");

    screen_buffer.set_centered_text_at_row(
        screen_height / 2 + 2,
        " Player 1: steer using left and right arrow keys",
    );

    if num_players > 1 {
        screen_buffer
            .set_centered_text_at_row(screen_height / 2 + 4, "Player 2: steer using A and D arrow");
    }

    for n in (0..5).rev() {
        screen_buffer.set_centered_text_at_row(screen_height - 4, &format!("Starting in {}", n));
        screen_buffer.draw(&mut stdout)?;
        thread::sleep(Duration::from_secs(1));
    }

    while !must_exit {
        let mut players = vec![Player::new(
            KeyEvent::from(KeyCode::Left),
            KeyEvent::from(KeyCode::Right),
            0,
        )];
        if num_players == 2 {
            players.push(Player::new(
                KeyEvent::from(KeyCode::Char('a')),
                KeyEvent::from(KeyCode::Char('d')),
                1,
            ));
        }

        screen_buffer.set_all(GameContent::Empty);

        let mut food_pos = Coordinate { row: 10, col: 15 };

        // 0: up, 1: right, 2: down, 3: left
        let mut game_loop_begin = std::time::SystemTime::now();
        let mut game_loop_end = std::time::SystemTime::now();
        let horizontal_target_cycle_time = Duration::from_secs_f32(1.0 / target_fps);

        'outer: loop {
            // ensure constant cycle time of game loop (i.e. constant snake speed)
            let game_loop_runtime = game_loop_end.duration_since(game_loop_begin).unwrap();
            let target_cycle_time = horizontal_target_cycle_time;
            if game_loop_runtime < target_cycle_time {
                thread::sleep(target_cycle_time - game_loop_runtime);
            }

            game_loop_begin = std::time::SystemTime::now();
            if let Some(events) = event_queue.get_all_events() {
                // TODO: GET EVENTS MATCHING ONE OF ...
                if !events.is_empty() {
                    let esc_matches = find_matches(
                        &events,
                        &vec![
                            KeyEvent::from(KeyCode::Esc),
                            KeyEvent::from(KeyCode::Char('q')),
                        ],
                    );

                    if !esc_matches.is_empty() {
                        must_exit = true;
                        break 'outer;
                    }
                    for mut player in &mut players {
                        let event_matches =
                            find_matches(&events, &vec![player.left_key, player.right_key]);
                        if !event_matches.is_empty() {
                            if *event_matches.last().unwrap() == player.left_key {
                                player.snake.direction -= 1;
                            } else if *event_matches.last().unwrap() == player.right_key {
                                player.snake.direction += 1;
                            }
                        }
                    }
                }
            }
            for mut player in &mut players {
                player.snake.direction = match player.snake.direction {
                    -1 => 3,
                    _ => player.snake.direction % 4,
                };

                move_snake(&mut player.snake.body_pos, player.snake.direction);
            }
            // TODO EVENTS CLEAR

            let mut food_found = false;
            for mut player in &mut players {
                if player.snake.body_pos[0] == food_pos {
                    player.score += 1;
                    food_found = true;

                    // grow snake
                    for _i in 0..3 {
                        player
                            .snake
                            .body_pos
                            .push(*player.snake.body_pos.last().unwrap());
                    }
                }
            }

            if food_found {
                let mut new_food_pos = get_random_food_pos(screen_height, screen_width);
                let mut no_collision = false;
                while !no_collision {
                    new_food_pos = get_random_food_pos(screen_height, screen_width);
                    for player in &players {
                        let collides = snake_item_collision(&player.snake.body_pos, &new_food_pos);
                        if !collides {
                            no_collision = true;
                        }
                    }
                }
                food_pos = new_food_pos;
            }

            // check for snake border and snake ego collisions
            for player in &players {
                if player.snake.body_pos[0].row == 0
                    || player.snake.body_pos[0].row == screen_height - 1
                    || player.snake.body_pos[0].col == 0
                    || player.snake.body_pos[0].col == screen_width - 1
                    || snake_item_collision(&player.snake.body_pos[1..], &player.snake.body_pos[0])
                {
                    break 'outer;
                }
            }
            // TODO check snake vs snake collision!
            if num_players == 2 {
                let coll_a_b = snake_item_collision(
                    &players[0].snake.body_pos[1..],
                    &players[1].snake.body_pos[0],
                );
                let coll_b_a = snake_item_collision(
                    &players[1].snake.body_pos[1..],
                    &players[0].snake.body_pos[0],
                );
                if coll_a_b || coll_b_a {
                    break 'outer;
                }
            }

            // clear, update and draw screen buffer
            screen_buffer.set_all(GameContent::Empty);
            for player in &players {
                add_snake_to_buffer(&mut screen_buffer, &player.snake.body_pos);
            }
            screen_buffer.set_at(food_pos.row, food_pos.col, GameContent::Food);
            screen_buffer.add_border(GameContent::Border);

            if num_players == 1 {
                screen_buffer
                    .set_centered_text_at_row(0, &format!("SNAKE - Score: {}", players[0].score));
            } else if num_players == 2 {
                screen_buffer.set_centered_text_at_row(
                    0,
                    &format!(
                        "SNAKE - Score: P1: {} - P2 {}",
                        players[0].score, players[1].score
                    ),
                );
            }

            screen_buffer.draw(&mut stdout)?;

            game_loop_end = std::time::SystemTime::now();
        }

        // draw empty buffer
        screen_buffer.set_all(GameContent::Empty);
        screen_buffer.draw(&mut stdout)?;

        screen_buffer.set_centered_text_at_row(screen_height / 2 - 2, "GAME OVER");

        if num_players == 1 {
            screen_buffer.set_centered_text_at_row(
                screen_height - 1,
                &format!("SNAKE - Final Score: {}", players[0].score),
            );
        } else if num_players == 2 {
            screen_buffer.set_centered_text_at_row(
                screen_height - 1,
                &format!(
                    "SNAKE - Final Score: P1: {} - P2: {}",
                    players[0].score, players[1].score
                ),
            );
        }
        if !must_exit {
            for n in (0..40).rev() {
                screen_buffer.set_centered_text_at_row(
                    screen_height / 2 + 2,
                    &format!("Restarting in {}", n / 10),
                );
                screen_buffer.set_centered_text_at_row(screen_height / 2 + 4, "ESC to abort");
                screen_buffer.draw(&mut stdout)?;
                if let Some(event) = event_queue.get_latest_event() {
                    if event == KeyEvent::from(KeyCode::Esc)
                        || event == KeyEvent::from(KeyCode::Char('q'))
                    {
                        must_exit = true;
                        break;
                    }
                }
                thread::sleep(Duration::from_secs_f32(0.1));
            }
        }
    }
    stdout.execute(cursor::Show)?;
    disable_raw_mode()
}
