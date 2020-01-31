use rand::Rng;
use std::time::Duration;

use std::io::Write;

use crossterm::{
    cursor::{self},
    event::KeyEvent,
    event::{poll, read, Event},
    style::{self, Attribute, Colorize},
    QueueableCommand, Result,
};

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
pub struct KeyEventQueue<T: Send + Copy> {
    inner: Arc<Mutex<VecDeque<T>>>,
}

#[derive(Clone, Copy, Debug)]
pub enum GameContent {
    SnakeHeadA,
    SnakeHeadB,
    SnakeBodyA,
    SnakeBodyB,
    Food,
    Border,
    Empty,
    Character(char),
}

impl<T: Send + Copy> KeyEventQueue<T> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn get_latest_event(&self) -> Option<T> {
        let maybe_queue = self.inner.lock();

        if let Ok(mut queue) = maybe_queue {
            let el = queue.pop_back();
            queue.clear();
            return el;
        } else {
            panic!("poisoned mutex");
        }
    }

    pub fn get_all_events(&self) -> Option<Vec<T>> {
        let maybe_queue = self.inner.lock();

        if let Ok(mut queue) = maybe_queue {
            let drained = queue.drain(..).collect::<Vec<_>>();
            queue.clear();
            return Some(drained);
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

pub fn send_events(event_queue: KeyEventQueue<KeyEvent>) -> crossterm::Result<()> {
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
pub struct Coordinate {
    pub row: usize,
    pub col: usize,
}

pub struct ScreenBuffer {
    screen_width: usize,
    screen_height: usize,
    buffer: Vec<GameContent>,
}

impl ScreenBuffer {
    pub fn new(
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

    pub fn set_all(&mut self, content: GameContent) {
        for screen_char in self.buffer.iter_mut() {
            *screen_char = content;
        }
    }

    pub fn set_centered_text_at_row(&mut self, target_row: usize, message: &str) {
        let str_chars = message.chars();
        let str_len = str_chars.clone().count();
        let header_start_idx = ((self.screen_width - str_len) / 2 as usize).max(0);

        let mut col_idx = header_start_idx;

        for sym in str_chars {
            self.set_at(target_row, col_idx, GameContent::Character(sym));
            col_idx += 1;
        }
    }

    pub fn get_at(&self, row: usize, col: usize) -> GameContent {
        return self.buffer[col + row * self.screen_width];
    }

    pub fn set_at(&mut self, row: usize, col: usize, content: GameContent) {
        self.buffer[col + row * self.screen_width] = content;
    }

    pub fn add_border(&mut self, border_symbol: GameContent) {
        for row in 0..self.screen_height {
            self.set_at(row, 0, border_symbol);
            self.set_at(row, self.screen_width - 1, border_symbol);
        }
        for col in 0..self.screen_width {
            self.set_at(0, col, border_symbol);
            self.set_at(self.screen_height - 1, col, border_symbol);
        }
    }

    pub fn draw(&self, stdout: &mut std::io::Stdout) -> Result<()> {
        for row_idx in 0..self.screen_height {
            for col_idx in 0..self.screen_width {
                let content = self.get_at(row_idx, col_idx);
                match content {
                    GameContent::Border => {
                        stdout
                            .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                            .queue(style::PrintStyledContent("█".dark_blue()))?;
                    }
                    GameContent::SnakeHeadA => {
                        stdout
                            .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                            .queue(style::PrintStyledContent("█".dark_green()))?;
                    }
                    GameContent::SnakeBodyA => {
                        stdout
                            .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                            .queue(style::PrintStyledContent("█".green()))?;
                    }
                    GameContent::SnakeHeadB => {
                        stdout
                            .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                            .queue(style::PrintStyledContent("█".dark_yellow()))?;
                    }
                    GameContent::SnakeBodyB => {
                        stdout
                            .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                            .queue(style::PrintStyledContent("█".yellow()))?;
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
                                .attribute(Attribute::Bold)
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
    pub score: usize,
    pub left_key: crossterm::event::KeyEvent,
    pub right_key: crossterm::event::KeyEvent,
    pub snake: Snake,
    pub player_idx: usize,
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
            score: 0,
        }
    }
}
