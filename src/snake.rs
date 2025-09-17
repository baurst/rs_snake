use rand::Rng;
use std::io::stdout;
use std::thread;
use std::time::Duration;

use crossterm::{
    cursor::{self},
    event::{KeyCode, KeyEvent},
    terminal::{self, disable_raw_mode, enable_raw_mode},
    ExecutableCommand,
};

use std::io::Result;

use crate::events::{send_events, KeyEventQueue};
use crate::screen_buffer::{Coordinate, GameContent, ScreenBuffer};

pub struct SnakeGame {
    num_players: usize,
    target_fps: f64,
    is_four_key_steering: bool,
}

impl SnakeGame {
    pub fn new(num_players: usize, target_fps: f64, is_four_key_steering: bool) -> SnakeGame {
        SnakeGame {
            num_players,
            target_fps,
            is_four_key_steering,
        }
    }

    pub fn run(self) -> Result<()> {
        let event_queue = KeyEventQueue::new();
        let thread_event_queue = event_queue.clone();

        // launch seperate thread to deal with keyboard input
        thread::spawn(move || send_events(&thread_event_queue));

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
            if self.is_four_key_steering {
                "Player 1 (green): arrow keys"
            } else {
                "Player 1 (green): left and right arrow keys"
            },
        );

        if self.num_players > 1 {
            screen_buffer.set_centered_text_at_row(
                screen_height / 2 + 6,
                if self.is_four_key_steering {
                    "Player 2 (yellow): W A S D keys"
                } else {
                    "Player 2 (yellow): A and D keys"
                },
            );
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
                KeyEvent::from(KeyCode::Up),
                KeyEvent::from(KeyCode::Down),
                0,
            )];
            if self.num_players == 2 {
                players.push(Player::new(
                    KeyEvent::from(KeyCode::Char('a')),
                    KeyEvent::from(KeyCode::Char('d')),
                    KeyEvent::from(KeyCode::Char('w')),
                    KeyEvent::from(KeyCode::Char('s')),
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
                            &[
                                KeyEvent::from(KeyCode::Esc),
                                KeyEvent::from(KeyCode::Char('q')),
                            ],
                        )
                        .is_empty()
                        {
                            must_exit = true;
                            break 'outer;
                        }
                        for player in &mut players {
                            let event_matches = find_matches(
                                &events,
                                &[
                                    player.left_key,
                                    player.right_key,
                                    player.up_key,
                                    player.down_key,
                                ],
                            );

                            if !event_matches.is_empty() {
                                player.update_snake_direction(
                                    *event_matches.last().unwrap(),
                                    self.is_four_key_steering,
                                );
                            }
                        }
                    }
                }

                for player in &mut players {
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
                        let has_collision = players.iter().any(|player| {
                            snake_item_collision(&player.snake.body_pos, &new_food_pos)
                        });

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
                for (player_id, player) in players.iter().enumerate() {
                    add_snake_to_buffer(&mut screen_buffer, &player.snake.body_pos, player_id);
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

pub fn move_snake(snake: &mut Vec<Coordinate>, snake_direction: Direction) {
    // add head in new direction
    let new_head = match snake_direction {
        Direction::Up => Coordinate {
            // up
            row: snake[0].row - 1,
            col: snake[0].col,
        },
        Direction::Right => Coordinate {
            // right
            row: snake[0].row,
            col: snake[0].col + 1,
        },
        Direction::Down => Coordinate {
            // down
            row: snake[0].row + 1,
            col: snake[0].col,
        },
        Direction::Left => Coordinate {
            // left
            row: snake[0].row,
            col: snake[0].col - 1,
        },
    };

    snake.insert(0, new_head);
    // remove tail
    snake.pop();
}

pub fn snake_item_collision(snake: &[Coordinate], item: &Coordinate) -> bool {
    let is_collision = snake.iter().position(|&r| r == *item);
    is_collision.is_some()
}

pub fn check_border_and_ego_collision(
    snake_body: &[Coordinate],
    screen_width: usize,
    screen_height: usize,
) -> bool {
    snake_body[0].row == 0
        || snake_body[0].row == screen_height - 1
        || snake_body[0].col == 0
        || snake_body[0].col == screen_width - 1
        || snake_item_collision(&snake_body[1..], &snake_body[0])
}

pub fn snake_snake_collision(snake_a: &[Coordinate], snake_b: &[Coordinate]) -> i64 {
    if snake_item_collision(&snake_a[1..], &snake_b[0]) {
        1
    } else if snake_item_collision(&snake_b[1..], &snake_a[0]) {
        0
    } else {
        -1
    }
}

pub fn get_random_food_pos(screen_height: usize, screen_width: usize) -> Coordinate {
    let mut rng = rand::thread_rng();
    // screen width and height -2, since -1 is the index of the border
    let row = rng.gen_range(1..(screen_height - 2));
    let col = rng.gen_range(1..(screen_width - 2));
    Coordinate { row, col }
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
    found
}

#[derive(PartialEq, Clone, Debug)]
pub struct Snake {
    pub body_pos: Vec<Coordinate>,
    // 0: up, 1: right, 2: down, 3: left
    pub direction: Direction,
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
            direction: Direction::Up,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Player {
    pub left_key: crossterm::event::KeyEvent,
    pub right_key: crossterm::event::KeyEvent,
    pub up_key: crossterm::event::KeyEvent,
    pub down_key: crossterm::event::KeyEvent,
    pub snake: Snake,
    pub player_idx: usize,
    pub has_crashed: bool,
}

impl Player {
    pub fn new(
        left_key: crossterm::event::KeyEvent,
        right_key: crossterm::event::KeyEvent,
        up_key: crossterm::event::KeyEvent,
        down_key: crossterm::event::KeyEvent,
        player_idx: usize,
    ) -> Player {
        Player {
            snake: Snake::new(player_idx),
            left_key,
            right_key,
            up_key,
            down_key,
            player_idx,
            has_crashed: false,
        }
    }
    pub fn update_snake_direction(
        &mut self,
        key_event: crossterm::event::KeyEvent,
        is_four_key_steering: bool,
    ) {
        if is_four_key_steering {
            self._update_direction_four_keys(key_event);
        } else {
            self._update_direction_two_keys(key_event);
        }
    }

    fn _update_direction_four_keys(&mut self, key_event: crossterm::event::KeyEvent) {
        if key_event == self.up_key
            && self.snake.direction != Direction::Up
            && self.snake.direction != Direction::Down
        {
            self.snake.direction = Direction::Up;
        } else if key_event == self.down_key
            && self.snake.direction != Direction::Up
            && self.snake.direction != Direction::Down
        {
            self.snake.direction = Direction::Down;
        } else if key_event == self.left_key
            && self.snake.direction != Direction::Right
            && self.snake.direction != Direction::Left
        {
            self.snake.direction = Direction::Left;
        } else if key_event == self.right_key
            && self.snake.direction != Direction::Right
            && self.snake.direction != Direction::Left
        {
            self.snake.direction = Direction::Right;
        }
    }

    fn _update_direction_two_keys(&mut self, key_event: crossterm::event::KeyEvent) {
        let directions_ordered = [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ];
        let mut current_dir_index = directions_ordered
            .iter()
            .position(|&r| r == self.snake.direction)
            .unwrap() as i64;

        if key_event == self.left_key {
            current_dir_index -= 1;
        } else if key_event == self.right_key {
            current_dir_index += 1;
        }

        current_dir_index = match current_dir_index {
            -1 => 3,
            _ => current_dir_index % 4,
        };

        self.snake.direction = directions_ordered[current_dir_index as usize];
    }
}

pub fn add_snake_to_buffer(
    screen_buffer: &mut ScreenBuffer,
    snake: &[Coordinate],
    player_idx: usize,
) {
    screen_buffer.set_at(
        snake[0].row,
        snake[0].col,
        GameContent::SnakeHead(player_idx),
    );

    // only use rest of the body
    let snake_body: Vec<&Coordinate> = snake
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != 0)
        .map(|(_, v)| v)
        .collect();

    for coord in snake_body {
        screen_buffer.set_at(coord.row, coord.col, GameContent::SnakeBody(player_idx));
    }
}
