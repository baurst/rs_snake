
extern crate termion;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::io::{Write, stdout, stdin};

fn clear_screen_buffer(screen_buffer: &mut Vec<char>) {
    for screen_char in screen_buffer {
        *screen_char = ' ';
    }
}

fn draw_screen_buffer(screen_buffer: Vec<char>, screen_width: usize, screen_height: usize) {
    for row in 0..screen_height {
        for col in 0..screen_width {
            print!("{}", screen_buffer[col + row * screen_height]);
        }
        print!("\n");
    }
}

fn main() {
    let screen_width = 40;
    let screen_height = 30;
    let mut screen_buffer = vec!['.'; screen_width * screen_height];

    // clear screen
    clear_screen_buffer(&mut screen_buffer);

    // draw screen buffer
    draw_screen_buffer(screen_buffer, screen_width, screen_height);

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    for c in stdin.keys() {

        match c.unwrap() {
            Key::Char('q') => break,
            Key::Char(c) => println!("{}", c),
            Key::Alt(c) => println!("^{}", c),
            Key::Ctrl(c) => println!("*{}", c),
            Key::Esc => println!("ESC"),
            Key::Left => println!("←"),
            Key::Right => println!("→"),
            Key::Up => println!("↑"),
            Key::Down => println!("↓"),
            Key::Backspace => println!("×"),
            _ => {}
        }
        stdout.flush().unwrap();
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
}
