use rand::Rng;
use std::io::stdout;
use std::thread;
use std::time::Duration;

use crossterm::{
    cursor::{self},
    event::{KeyCode, KeyEvent},
    terminal::{self, disable_raw_mode, enable_raw_mode},
    ExecutableCommand, Result,
};

use crate::events::{send_events, KeyEventQueue};
use crate::screen_buffer::{Coordinate, GameContent, ScreenBuffer};

pub struct SnakeGame {
    num_players: usize,
    target_fps: f64,
}

impl SnakeGame {
    pub fn new(num_players: usize, target_fps: f64) -> SnakeGame {
        return SnakeGame {
            num_players: num_players,
            target_fps: target_fps,
        };
    }

    pub fn run(self) -> Result<()> {
        let event_queue = KeyEventQueue::new();
        let thread_event_queue = event_queue.clone();

        // launch seperate thread to deal with keyboard input
        thread::spawn(move || send_events(thread_event_queue));

        let mut stdout = stdout();
        enable_raw_mode()?;
        stdout.execute(cursor::Hide)?;

        stdout.execute(terminal::Clear(terminal::ClearType::All))?;

        let screen_width = 40;
        let screen_height = 40;

        let mut screen_buffer = ScreenBuffer::new(screen_width, screen_height, GameContent::Empty);

        // clear screen
        screen_buffer.set_all(GameContent::Empty);
        screen_buffer.set_centered_text_at_row(screen_height / 2 - 6, "SNAKE");
        screen_buffer.set_centered_text_at_row(screen_height / 2 - 4, "ESC to stop");
        screen_buffer.set_centered_text_at_row(screen_height / 2 + 2, "~ CONTROLS ~");

        screen_buffer.set_centered_text_at_row(
            screen_height / 2 + 4,
            " Player 1: left and right arrow keys",
        );

        if self.num_players > 1 {
            screen_buffer.set_centered_text_at_row(screen_height / 2 + 6, "Player 2: A and D");
        }

        for n in (0..5).rev() {
            screen_buffer
                .set_centered_text_at_row(screen_height - 2, &format!("Starting in {}", n));
            screen_buffer.draw(&mut stdout)?;
            thread::sleep(Duration::from_secs(1));
        }

        let mut must_exit = false;
        while !must_exit {
            let mut players = vec![Player::new(
                KeyEvent::from(KeyCode::Left),
                KeyEvent::from(KeyCode::Right),
                0,
            )];
            if self.num_players == 2 {
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
            let horizontal_target_cycle_time = Duration::from_secs_f64(1.0 / self.target_fps);
            let mut score = 0;
            'outer: loop {
                // ensure constant cycle time of game loop (i.e. constant snake speed)
                let game_loop_runtime = game_loop_end.duration_since(game_loop_begin).unwrap();
                let target_cycle_time = horizontal_target_cycle_time;

                if game_loop_runtime < target_cycle_time {
                    thread::sleep(target_cycle_time - game_loop_runtime);
                }

                game_loop_begin = std::time::SystemTime::now();
                if let Some(events) = event_queue.get_all_events() {
                    if !events.is_empty() {
                        if !find_matches(
                            &events,
                            &vec![
                                KeyEvent::from(KeyCode::Esc),
                                KeyEvent::from(KeyCode::Char('q')),
                            ],
                        )
                        .is_empty()
                        {
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

                let mut food_found = false;
                for player in &mut players {
                    if player.snake.body_pos[0] == food_pos {
                        score += 1;
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
                    loop {
                        let new_food_pos = get_random_food_pos(screen_height, screen_width);
                        let has_collision = players
                            .iter()
                            .find(|player| {
                                snake_item_collision(&player.snake.body_pos, &new_food_pos)
                            })
                            .is_some();

                        if !has_collision {
                            food_pos = new_food_pos;
                            break;
                        }
                    }
                }

                // check for snake border and snake ego collisions
                for player in &mut players {
                    if check_border_and_ego_collision(
                        &player.snake.body_pos,
                        screen_width,
                        screen_height,
                    ) {
                        player.has_crashed = true;
                        break 'outer;
                    }
                }

                if self.num_players == 2 {
                    let collider = snake_snake_collision(
                        &players[0].snake.body_pos,
                        &players[1].snake.body_pos,
                    );
                    if collider >= 0 {
                        players[collider as usize].has_crashed = true;
                        break 'outer;
                    }
                }

                // clear, update and draw screen buffer
                screen_buffer.set_all(GameContent::Empty);
                let mut player_id = 0;
                for player in &players {
                    add_snake_to_buffer(&mut screen_buffer, &player.snake.body_pos, player_id);
                    player_id += 1;
                }
                screen_buffer.set_at(food_pos.row, food_pos.col, GameContent::Food);
                screen_buffer.add_border(GameContent::Border);

                screen_buffer.set_centered_text_at_row(0, &format!("Score: {}", score));

                screen_buffer.draw(&mut stdout)?;

                game_loop_end = std::time::SystemTime::now();
            }

            // draw empty buffer
            screen_buffer.set_all(GameContent::Empty);
            screen_buffer.draw(&mut stdout)?;

            screen_buffer.set_centered_text_at_row(screen_height / 2 - 8, "! GAME OVER !");

            screen_buffer.set_centered_text_at_row(
                screen_height / 2 - 2,
                &format!("Final Score: {}", score),
            );

            if !must_exit {
                for n in (0..40).rev() {
                    screen_buffer.set_centered_text_at_row(
                        screen_height / 2 + 10,
                        &format!("Restarting in ... {}s", n / 10),
                    );
                    screen_buffer.set_centered_text_at_row(screen_height - 4, "ESC to abort");
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
}

pub fn move_snake(snake: &mut Vec<Coordinate>, snake_direction: i64) {
    // add head in new direction
    let new_head = match snake_direction {
        0 => Coordinate {
            // up
            row: snake[0].row - 1,
            col: snake[0].col,
        },
        1 => Coordinate {
            // right
            row: snake[0].row,
            col: snake[0].col + 1,
        },
        2 => Coordinate {
            // down
            row: snake[0].row + 1,
            col: snake[0].col,
        },
        3 => Coordinate {
            // left
            row: snake[0].row,
            col: snake[0].col - 1,
        },
        _ => Coordinate {
            // no movement at all, invalid direction
            row: snake[0].row,
            col: snake[0].col,
        },
    };

    snake.insert(0, new_head);
    // remove tail
    snake.pop();
}

pub fn snake_item_collision(snake: &[Coordinate], item: &Coordinate) -> bool {
    let is_collision = snake.iter().position(|&r| r == *item);
    return is_collision.is_some();
}

pub fn check_border_and_ego_collision(
    snake_body: &[Coordinate],
    screen_width: usize,
    screen_height: usize,
) -> bool {
    return snake_body[0].row == 0
        || snake_body[0].row == screen_height - 1
        || snake_body[0].col == 0
        || snake_body[0].col == screen_width - 1
        || snake_item_collision(&snake_body[1..], &snake_body[0]);
}

pub fn snake_snake_collision(snake_a: &[Coordinate], snake_b: &[Coordinate]) -> i64 {
    if snake_item_collision(&snake_a[1..], &snake_b[0]) {
        return 1;
    } else if snake_item_collision(&snake_b[1..], &snake_a[0]) {
        return 0;
    } else {
        return -1;
    }
}

pub fn get_random_food_pos(screen_height: usize, screen_width: usize) -> Coordinate {
    let mut rng = rand::thread_rng();
    let row = rng.gen_range(1, screen_height - 1);
    let col = rng.gen_range(1, screen_width - 1);
    return Coordinate { row: row, col: col };
}

pub fn find_matches<T: PartialEq + Copy>(look_in: &[T], look_for: &[T]) -> Vec<T> {
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
pub struct Snake {
    pub body_pos: Vec<Coordinate>,
    // 0: up, 1: right, 2: down, 3: left
    pub direction: i64,
}

impl Snake {
    pub fn new(player_idx: usize) -> Snake {
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
pub struct Player {
    pub left_key: crossterm::event::KeyEvent,
    pub right_key: crossterm::event::KeyEvent,
    pub snake: Snake,
    pub player_idx: usize,
    pub has_crashed: bool,
}

impl Player {
    pub fn new(
        left_key: crossterm::event::KeyEvent,
        right_key: crossterm::event::KeyEvent,
        player_idx: usize,
    ) -> Player {
        Player {
            snake: Snake::new(player_idx),
            left_key: left_key,
            right_key: right_key,
            player_idx: player_idx,
            has_crashed: false,
        }
    }
}

pub fn add_snake_to_buffer(
    screen_buffer: &mut ScreenBuffer,
    snake: &Vec<Coordinate>,
    player_idx: usize,
) {
    let head_content = if player_idx == 1 {
        GameContent::SnakeHeadA
    } else {
        GameContent::SnakeHeadB
    };
    screen_buffer.set_at(snake[0].row, snake[0].col, head_content);

    // only use rest of the body
    let body_content = if player_idx == 1 {
        GameContent::SnakeBodyA
    } else {
        GameContent::SnakeBodyB
    };
    let snake_body: Vec<&Coordinate> = snake
        .into_iter()
        .enumerate()
        .filter(|&(i, _)| i != 0)
        .map(|(_, v)| v)
        .collect();

    for coord in snake_body {
        screen_buffer.set_at(coord.row, coord.col, body_content);
    }
}
