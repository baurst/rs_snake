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
    if matches.is_present("multiplayer"){
        num_players +=1;
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

    let screen_width = 60;
    let screen_height = 30;

    let mut must_exit = false;

    let mut screen_buffer = ScreenBuffer::new(screen_width, screen_height, GameContent::Empty);

    // clear screen
    screen_buffer.set_all(GameContent::Empty);
    screen_buffer.set_centered_text_at_row(screen_height / 2 - 2, "SNAKE");

    screen_buffer
        .set_centered_text_at_row(screen_height / 2, "steer using left and right arrow keys");

    screen_buffer.set_centered_text_at_row(screen_height / 2 + 4, "press ESC to stop");

    for n in (0..5).rev() {
        screen_buffer
            .set_centered_text_at_row(screen_height / 2 + 2, &format!("Starting in {}", n));
        screen_buffer.draw(&mut stdout)?;
        thread::sleep(Duration::from_secs(1));
    }

    while !must_exit {
        screen_buffer.set_all(GameContent::Empty);

        let mut food_pos = Coordinate { row: 10, col: 15 };

        let mut snake = vec![
            Coordinate { row: 18, col: 15 },
            Coordinate { row: 19, col: 15 },
            Coordinate { row: 20, col: 15 },
        ];

        // 0: up, 1: right, 2: down, 3: left
        let mut snake_direction = 0;

        let mut score = 0;
        let mut game_loop_begin = std::time::SystemTime::now();
        let mut game_loop_end = std::time::SystemTime::now();
        let horizontal_target_cycle_time = Duration::from_secs_f32(1.0 / target_fps);
        let vertical_target_cycle_time = Duration::from_secs_f32(1.5 / target_fps);

        loop {
            // ensure constant cycle time of game loop (i.e. constant snake speed)
            let game_loop_runtime = game_loop_end.duration_since(game_loop_begin).unwrap();
            let target_cycle_time = if snake_direction % 2 == 1 {
                horizontal_target_cycle_time
            } else {
                vertical_target_cycle_time
            };
            if game_loop_runtime < target_cycle_time {
                thread::sleep(target_cycle_time - game_loop_runtime);
            }

            game_loop_begin = std::time::SystemTime::now();
            if let Some(event) = event_queue.get_latest_event() {
                if event == KeyEvent::from(KeyCode::Esc)
                    || event == KeyEvent::from(KeyCode::Char('q'))
                {
                    must_exit = true;
                    break;
                } else if event == KeyEvent::from(KeyCode::Left) {
                    snake_direction -= 1;
                } else if event == KeyEvent::from(KeyCode::Right) {
                    snake_direction += 1;
                }
            }

            snake_direction = match snake_direction {
                -1 => 3,
                _ => snake_direction % 4,
            };

            move_snake(&mut snake, snake_direction);

            if snake[0] == food_pos {
                score += 1;
                // place new food
                let mut new_food_pos = get_random_food_pos(screen_height, screen_width);
                while snake_item_collision(&snake, &new_food_pos) {
                    new_food_pos = get_random_food_pos(screen_height, screen_width);
                }
                food_pos = new_food_pos;

                // grow snake
                for _i in 0..3 {
                    snake.push(*snake.last().unwrap());
                }
            }

            // check for collisions
            if snake[0].row == 0
                || snake[0].row == screen_height - 1
                || snake[0].col == 0
                || snake[0].col == screen_width - 1
                || snake_item_collision(&snake[1..], &snake[0])
            {
                break;
            }

            // clear, update and draw screen buffer
            screen_buffer.set_all(GameContent::Empty);
            screen_buffer.set_at(food_pos.row, food_pos.col, GameContent::Food);
            add_snake_to_buffer(&mut screen_buffer, &snake);
            screen_buffer.add_border(GameContent::Border);
            screen_buffer.set_centered_text_at_row(0, &format!("SNAKE - Score: {}", score));

            screen_buffer.draw(&mut stdout)?;

            game_loop_end = std::time::SystemTime::now();
        }

        // draw empty buffer
        screen_buffer.set_all(GameContent::Empty);
        screen_buffer.draw(&mut stdout)?;

        screen_buffer.set_centered_text_at_row(screen_height / 2 - 2, "GAME OVER");
        screen_buffer
            .set_centered_text_at_row(screen_height / 2, &format!("SNAKE - Score: {}", score));
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
