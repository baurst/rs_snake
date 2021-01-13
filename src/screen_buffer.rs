use std::io::Write;

use crossterm::{
    cursor::{self},
    style::{self, Colorize, StyledContent},
    QueueableCommand, Result,
};

#[derive(Clone, Copy, Debug)]
pub enum GameContent {
    SnakeHead(usize),
    SnakeBody(usize),
    Food,
    Border,
    Empty,
    Character(char),
    CharacterOnBorder(char),
}

fn map_game_content_to_color(gc: &GameContent, is_padded_char: bool) -> StyledContent<String> {
    // \u{2588} is a full block symbol
    match gc {
        GameContent::SnakeHead(player_idx) => {
            if *player_idx == 0_usize {
                "\u{2588}".to_string().dark_green()
            } else {
                "\u{2588}".to_string().dark_yellow()
            }
        }
        GameContent::SnakeBody(player_idx) => {
            if *player_idx == 0_usize {
                "\u{2588}".to_string().green()
            } else {
                "\u{2588}".to_string().yellow()
            }
        }
        GameContent::Food => "\u{2588}".to_string().red(),
        GameContent::Border => "\u{2588}".to_string().dark_blue(),
        GameContent::Empty => "\u{2588}".to_string().black(),
        GameContent::Character(some_char) => {
            if is_padded_char {
                "\u{2588}".to_string().black()
            } else {
                some_char.to_string().white().on_black()
            }
        }
        GameContent::CharacterOnBorder(some_char) => {
            if is_padded_char {
                "\u{2588}".to_string().dark_blue()
            } else {
                some_char.to_string().white().on_dark_blue()
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
            screen_height,
            screen_width,
            buffer: vec![initial_content; screen_height * screen_width],
        }
    }

    pub fn set_all(&mut self, content: GameContent) {
        for screen_char in &mut self.buffer {
            *screen_char = content;
        }
    }

    pub fn set_centered_text_at_row(&mut self, target_row: usize, message: &str) {
        let str_chars = message.chars();
        let str_len = str_chars.clone().count();
        let header_start_idx = ((self.screen_width - str_len) / 2usize).max(0);

        let mut col_idx = header_start_idx;

        for sym in str_chars {
            let gc = if target_row == 0 {
                GameContent::CharacterOnBorder(sym)
            } else {
                GameContent::Character(sym)
            };
            self.set_at(target_row, col_idx, gc);
            col_idx += 1;
        }
    }

    pub fn get_at(&self, row: usize, col: usize) -> GameContent {
        self.buffer[col + row * self.screen_width]
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
            for col_idx_buffer in 0..self.screen_width {
                let content = self.get_at(row_idx, col_idx_buffer);
                // draw each element twice horizontally, so that we get square "pixels"
                for i in 0..2 {
                    let col_idx = 2 * col_idx_buffer + i;

                    let styled_content = map_game_content_to_color(&content, i != 0);
                    stdout
                        .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                        .queue(style::PrintStyledContent(styled_content))?;
                }
            }
        }
        stdout.flush()?;
        Ok(())
    }
}
