use std::io::Write;

use crossterm::{
    cursor::{self},
    style::{self, Attribute, Colorize},
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
            for col_idx_buffer in 0..self.screen_width {
                let content = self.get_at(row_idx, col_idx_buffer);
                for i in 0..2 {
                    let col_idx = 2 * col_idx_buffer + i;
                    match content {
                        GameContent::Border => {
                            stdout
                                .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                                .queue(style::PrintStyledContent("█".dark_blue()))?;
                        }
                        GameContent::SnakeHead(player_idx) => {
                            if player_idx == 0 {
                                stdout
                                    .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                                    .queue(style::PrintStyledContent("█".dark_green()))?;
                            } else if player_idx == 1 {
                                stdout
                                    .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                                    .queue(style::PrintStyledContent("█".dark_yellow()))?;
                            }
                        }
                        GameContent::SnakeBody(player_idx) => {
                            if player_idx == 0 {
                                stdout
                                    .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                                    .queue(style::PrintStyledContent("█".green()))?;
                            } else if player_idx == 1 {
                                stdout
                                    .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                                    .queue(style::PrintStyledContent("█".yellow()))?;
                            }
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
                            let is_first_char = i == 0;
                            let styled_c: crossterm::style::StyledContent<String> =
                                match is_first_char {
                                    true => crossterm::style::style(character.to_string())
                                        .attribute(Attribute::Bold)
                                        .red()
                                        .on_dark_grey(),
                                    _ => crossterm::style::style("█".to_string()).dark_grey(),
                                };
                            stdout
                                .queue(cursor::MoveTo(col_idx as u16, row_idx as u16))?
                                .queue(style::PrintStyledContent(styled_c))?;
                        }
                    }
                }
            }
        }
        stdout.flush()?;
        Ok(())
    }
}
