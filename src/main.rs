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
struct KeyEventQueue<T: Send + Copy> {
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

impl<T: Send + Copy> KeyEventQueue<T> {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
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

fn send_events(event_queue: KeyEventQueue<KeyEvent>) -> crossterm::Result<()> {
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

struct ScreenBuffer {
    screen_width: usize,
    screen_height: usize,
    buffer: Vec<GameContent>,
}

impl ScreenBuffer {
    fn new(
        screen_width: usize,
        screen_height: usize,
        initial_content: GameContent,
    ) -> ScreenBuffer {
        ScreenBuffer {
            screen_height: screen_height,
            screen_width: screen_width,
            buffer: vec![initial_content; screen_height * screen_width],
        }
    }

    fn set_all(&mut self, content: GameContent) {
        for screen_char in self.buffer.iter_mut() {
            *screen_char = content;
        }
    }

    fn set_centered_text_at_row(&mut self, target_row: usize, message: &str) {
        let str_chars = message.chars();
        let str_len = str_chars.clone().count();
        let header_start_idx = ((self.screen_width - str_len) / 2 as usize).max(0);

        let mut col_idx = header_start_idx;

        for sym in str_chars {
            self.set_at(target_row, col_idx, GameContent::Character(sym));
            col_idx += 1;
        }
    }

    fn get_at(&self, row: usize, col: usize) -> GameContent {
        return self.buffer[col + row * self.screen_width];
    }

    fn set_at(&mut self, row: usize, col: usize, content: GameContent) {
        self.buffer[col + row * self.screen_width] = content;
    }

    fn add_border(&mut self, border_symbol: GameContent) {
        for row in 0..self.screen_height {
            self.set_at(row, 0, border_symbol);
            self.set_at(row, self.screen_width - 1, border_symbol);
        }
        for col in 0..self.screen_width {
            self.set_at(0, col, border_symbol);
            self.set_at(self.screen_height - 1, col, border_symbol);
        }
    }

    fn draw(&self, stdout: &mut std::io::Stdout) -> Result<()> {
        for row_idx in 0..self.screen_height {
            for col_idx in 0..self.screen_width {
                let content = self.get_at(row_idx, col_idx);
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
}

fn add_snake_to_buffer(screen_buffer: &mut ScreenBuffer, snake: &Vec<Coordinate>) {
    screen_buffer.set_at(snake[0].row, snake[0].col, GameContent::SnakeHead);

    // only use rest of the body
    let snake_body: Vec<&Coordinate> = snake
        .into_iter()
        .enumerate()
        .filter(|&(i, _)| i != 0)
        .map(|(_, v)| v)
        .collect();

    for coord in snake_body {
        screen_buffer.set_at(coord.row, coord.col, GameContent::SnakeBody);
    }
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

    while !must_exit {
        // clear screen
        screen_buffer.set_all(GameContent::Empty);
        screen_buffer.set_centered_text_at_row(screen_height / 2 - 2, "SNAKE");

        screen_buffer
            .set_centered_text_at_row(screen_height / 2, "steer using left and right arrow keys");

        screen_buffer.set_centered_text_at_row(screen_height / 2 + 4, "ESC to stop");

        for n in (0..5).rev() {
            screen_buffer
                .set_centered_text_at_row(screen_height / 2 + 2, &format!("Starting in {}", n));
            screen_buffer.draw(&mut stdout)?;
            thread::sleep(Duration::from_secs(1));
        }

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
            screen_buffer.set_centered_text_at_row(screen_height / 2 + 4, "Restarting...");
        }

        screen_buffer.draw(&mut stdout)?;
        if !must_exit {
            thread::sleep(Duration::from_secs(5));
        }
    }
    stdout.execute(cursor::Show)?;
    disable_raw_mode()
}
