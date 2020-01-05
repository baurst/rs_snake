extern crate termion;

use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

struct Coordinate {
    row: usize,
    col: usize,
}

struct ScreenBufferHelper {
    num_rows: usize,
    num_cols: usize,
}

fn clear_screen_buffer(screen_buffer: &mut Vec<char>) {
    for screen_char in screen_buffer {
        *screen_char = ' ';
    }
}

fn draw_screen_buffer(screen_buffer: &Vec<char>, screen_width: usize, screen_height: usize) {
    for row in 0..screen_height {
        for col in 0..screen_width {
            print!("{}", get_buffer_at(screen_buffer, screen_width, row, col));
        }
        print!("\n");
    }
}

fn add_game_border_to_buffer(
    screen_buffer: &mut Vec<char>,
    screen_width: usize,
    screen_height: usize,
) {
    let vert_border_symbol = '_';
    let hor_border_symbol = '|';

    for row in 0..screen_height {
        set_buffer_at(screen_buffer, screen_width, row, 0, hor_border_symbol);
        set_buffer_at(
            screen_buffer,
            screen_width,
            row,
            screen_width - 1,
            hor_border_symbol,
        );
    }
    for col in 0..screen_width {
        set_buffer_at(screen_buffer, screen_width, 0, col, vert_border_symbol);
        set_buffer_at(
            screen_buffer,
            screen_width,
            screen_height - 1,
            col,
            vert_border_symbol,
        );
    }
}

fn set_buffer_at(
    screen_buffer: &mut Vec<char>,
    screen_width: usize,
    row: usize,
    col: usize,
    sym: char,
) {
    screen_buffer[col + row * screen_width] = sym;
}

fn get_buffer_at(screen_buffer: &Vec<char>, screen_width: usize, row: usize, col: usize) -> char {
    return screen_buffer[col + row * screen_width];
}

fn main() {
    let screen_width = 40;
    let screen_height = 30;
    let mut screen_buffer = vec!['.'; screen_width * screen_height];

    // clear screen
    clear_screen_buffer(&mut screen_buffer);

    let mut food_pos = Coordinate { row: 20, col: 15 };

    set_buffer_at(
        &mut screen_buffer,
        screen_width,
        food_pos.row,
        food_pos.col,
        'O',
    );
    add_game_border_to_buffer(&mut screen_buffer, screen_width, screen_height);

    loop {
        // draw screen buffer
        draw_screen_buffer(&screen_buffer, screen_width, screen_height);

        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode().unwrap();

        let c = stdin.keys().nth(0).unwrap();
        match c.unwrap() {
            Key::Char('q') => {
                break;
            }
            Key::Esc => {
                break;
            }
            Key::Left => set_buffer_at(&mut screen_buffer, screen_width, 20, 20, 'l'),
            Key::Right => set_buffer_at(&mut screen_buffer, screen_width, 20, 20, 'r'),
            Key::Up => set_buffer_at(&mut screen_buffer, screen_width, 20, 20, 'u'),
            Key::Down => set_buffer_at(&mut screen_buffer, screen_width, 20, 20, 'd'),
            Key::Backspace => println!("×"),
            _ => {}
        }
        stdout.flush().unwrap();

        // write!(stdout, "{}", termion::cursor::Show).unwrap();
    }
}
