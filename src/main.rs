use rand::Rng;
use std::thread;
use std::time::Duration;

use std::io::{stdout, Write};

use crossterm::{
    cursor::{self},
    event::{poll, read, Event},
    event::{KeyCode, KeyEvent},
    style::{self, Colorize},
    terminal::{self, disable_raw_mode, enable_raw_mode},
    ExecutableCommand, QueueableCommand, Result,
};

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
struct EventQueue<T: Send + Copy> {
    inner: Arc<Mutex<VecDeque<T>>>,
}

#[derive(Clone, Copy, Debug)]
enum GameContent {
    SnakeHead,
    SnakeBody,
    Food,
    Border,
    Empty,
    Character(char),
}

impl<T: Send + Copy> EventQueue<T> {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn _get_event(&self) -> Option<T> {
        let maybe_queue = self.inner.lock();

        if let Ok(mut queue) = maybe_queue {
            queue.pop_front()
        } else {
            panic!("poisoned mutex");
        }
    }

    fn get_latest_event(&self) -> Option<T> {
        let maybe_queue = self.inner.lock();

        if let Ok(mut queue) = maybe_queue {
            let el = queue.pop_back();
            queue.clear();
            return el;
        } else {
            panic!("poisoned mutex");
        }
    }

    fn add_event(&self, event: T) -> usize {
        if let Ok(mut queue) = self.inner.lock() {
            queue.push_back(event);
            queue.len()
        } else {
            panic!("poisoned mutex");
        }
    }
}

fn send_events(event_queue: EventQueue<KeyEvent>) -> crossterm::Result<()> {
    loop {
        if poll(Duration::from_millis(3))? {
            match read()? {
                // will not block
                Event::Key(event) => {
                    event_queue.add_event(event);
                }
                Event::Mouse(_event) => {}
                Event::Resize(_width, _height) => {}
            }
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
struct Coordinate {
    row: usize,
    col: usize,
}

fn clear_screen_buffer(screen_buffer: &mut Vec<GameContent>) {
    for screen_char in screen_buffer {
        *screen_char = GameContent::Empty;
    }
}

fn draw_screen_buffer(
    screen_buffer: &Vec<GameContent>,
    stdout: &mut std::io::Stdout,
    screen_width: usize,
    screen_height: usize,
) -> Result<()> {
    for row_idx in 0..screen_height {
        for col_idx in 0..screen_width {
            let content = get_buffer_at(&screen_buffer, screen_width, row_idx, col_idx);
            match content {
                GameContent::Border => {
                    stdout
                        .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                        .queue(style::PrintStyledContent("█".dark_blue()))?;
                }
                GameContent::SnakeHead | GameContent::SnakeBody => {
                    stdout
                        .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                        .queue(style::PrintStyledContent("█".dark_green()))?;
                }
                GameContent::Food => {
                    stdout
                        .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                        .queue(style::PrintStyledContent("█".red()))?;
                }
                GameContent::Empty => {
                    stdout
                        .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                        .queue(style::PrintStyledContent("█".dark_grey()))?;
                }
                GameContent::Character(character) => {
                    let styled_c: crossterm::style::StyledContent<String> =
                        crossterm::style::style(character.to_string())
                            .red()
                            .on_dark_grey();
                    stdout
                        .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                        .queue(style::PrintStyledContent(styled_c))?;
                }
            }
        }
    }
    stdout.flush()?;
    Ok(())
}

fn add_game_border_to_buffer(
    screen_buffer: &mut Vec<GameContent>,
    screen_width: usize,
    screen_height: usize,
) {
    for row in 0..screen_height {
        set_buffer_at(screen_buffer, screen_width, row, 0, GameContent::Border);
        set_buffer_at(
            screen_buffer,
            screen_width,
            row,
            screen_width - 1,
            GameContent::Border,
        );
    }
    for col in 0..screen_width {
        set_buffer_at(screen_buffer, screen_width, 0, col, GameContent::Border);
        set_buffer_at(
            screen_buffer,
            screen_width,
            screen_height - 1,
            col,
            GameContent::Border,
        );
    }
}

fn add_snake_to_buffer(
    screen_buffer: &mut Vec<GameContent>,
    snake: &Vec<Coordinate>,
    screen_width: usize,
) {
    set_buffer_at(
        screen_buffer,
        screen_width,
        snake[0].row,
        snake[0].col,
        GameContent::SnakeHead,
    );

    // only use rest of the body
    let snake_body: Vec<&Coordinate> = snake
        .into_iter()
        .enumerate()
        .filter(|&(i, _)| i != 0)
        .map(|(_, v)| v)
        .collect();

    for coord in snake_body {
        set_buffer_at(
            screen_buffer,
            screen_width,
            coord.row,
            coord.col,
            GameContent::SnakeBody,
        );
    }
}

fn add_centered_text_to_buffer(
    screen_buffer: &mut Vec<GameContent>,
    screen_width: usize,
    target_row: usize,
    message: &str,
) {
    let str_chars = message.chars();
    let str_len = str_chars.clone().count();
    let header_start_idx = ((screen_width - str_len) / 2 as usize).max(0);

    let mut col_idx = header_start_idx;

    for sym in str_chars {
        set_buffer_at(
            screen_buffer,
            screen_width,
            target_row,
            col_idx,
            GameContent::Character(sym),
        );
        col_idx += 1;
    }
}

fn set_buffer_at(
    screen_buffer: &mut Vec<GameContent>,
    screen_width: usize,
    row: usize,
    col: usize,
    content: GameContent,
) {
    screen_buffer[col + row * screen_width] = content;
}

fn get_buffer_at(
    screen_buffer: &Vec<GameContent>,
    screen_width: usize,
    row: usize,
    col: usize,
) -> GameContent {
    return screen_buffer[col + row * screen_width];
}

fn move_snake(snake: &mut Vec<Coordinate>, snake_direction: i64) {
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

fn snake_item_collision(snake: &[Coordinate], item: &Coordinate) -> bool {
    let is_collision = snake.iter().position(|&r| r == *item);
    return is_collision.is_some();
}

fn get_random_food_pos(screen_height: usize, screen_width: usize) -> Coordinate {
    let mut rng = rand::thread_rng();
    let row = rng.gen_range(1, screen_height - 1);
    let col = rng.gen_range(1, screen_width - 1);
    return Coordinate { row: row, col: col };
}

fn main() -> Result<()> {
    let event_queue = EventQueue::new();
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

    while !must_exit {
        let mut screen_buffer = vec![GameContent::Empty; screen_width * screen_height];

        // clear screen
        clear_screen_buffer(&mut screen_buffer);

        add_centered_text_to_buffer(
            &mut screen_buffer,
            screen_width,
            screen_height / 2 - 2,
            "SNAKE",
        );
        add_centered_text_to_buffer(
            &mut screen_buffer,
            screen_width,
            screen_height / 2,
            "steer using left and right arrow keys",
        );

        add_centered_text_to_buffer(
            &mut screen_buffer,
            screen_width,
            screen_height / 2 + 4,
            "ESC to stop",
        );

        for n in (0..5).rev() {
            add_centered_text_to_buffer(
                &mut screen_buffer,
                screen_width,
                screen_height / 2 + 2,
                &format!("Starting in {}", n),
            );
            draw_screen_buffer(&screen_buffer, &mut stdout, screen_width, screen_height)?;
            thread::sleep(Duration::from_secs(1));
        }

        clear_screen_buffer(&mut screen_buffer);

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
        let horizontal_target_cycle_time = Duration::from_millis(50);
        let vertical_target_cycle_time = Duration::from_millis(75);

        loop {
            // ensure constant cycle time of game loop (i.e. constant snake speed)
            let game_loop_runtime = game_loop_end.duration_since(game_loop_begin).unwrap();
            let target_cycle_time = if snake_direction % 2 == 1 {
                horizontal_target_cycle_time
            } else {
                vertical_target_cycle_time
            };
            let sleep_time =
                (target_cycle_time - game_loop_runtime).max(std::time::Duration::from_millis(0));
            thread::sleep(sleep_time);

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
                for _i in 0..2 {
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
            clear_screen_buffer(&mut screen_buffer);

            set_buffer_at(
                &mut screen_buffer,
                screen_width,
                food_pos.row,
                food_pos.col,
                GameContent::Food,
            );

            add_snake_to_buffer(&mut screen_buffer, &snake, screen_width);
            add_game_border_to_buffer(&mut screen_buffer, screen_width, screen_height);
            add_centered_text_to_buffer(
                &mut screen_buffer,
                screen_width,
                0,
                &format!("SNAKE - Score: {}", score),
            );
            draw_screen_buffer(&screen_buffer, &mut stdout, screen_width, screen_height)?;

            game_loop_end = std::time::SystemTime::now();
        }

        // draw empty buffer
        clear_screen_buffer(&mut screen_buffer);
        draw_screen_buffer(&screen_buffer, &mut stdout, screen_width, screen_height)?;

        add_centered_text_to_buffer(
            &mut screen_buffer,
            screen_width,
            screen_height / 2 - 2,
            "GAME OVER",
        );
        add_centered_text_to_buffer(
            &mut screen_buffer,
            screen_width,
            screen_height / 2 + 2,
            &format!("Final Score: {}", score),
        );
        if !must_exit {
            add_centered_text_to_buffer(
                &mut screen_buffer,
                screen_width,
                screen_height / 2 + 4,
                "Restarting...",
            );
        }

        draw_screen_buffer(&screen_buffer, &mut stdout, screen_width, screen_height)?;
        if !must_exit {
            thread::sleep(Duration::from_secs(5));
        }
    }
    stdout.execute(cursor::Show)?;
    disable_raw_mode()
}
