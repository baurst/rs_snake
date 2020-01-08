use rand::Rng;
use std::thread;
use std::time::Duration;

use device_query::{DeviceQuery, DeviceState};

use std::sync::mpsc;

#[derive(PartialEq, Clone, Copy, Debug)]
struct Coordinate {
    row: usize,
    col: usize,
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
    let upper_vert_border_symbol = '_';
    let lower_vert_border_symbol = '‾';
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
        set_buffer_at(
            screen_buffer,
            screen_width,
            0,
            col,
            upper_vert_border_symbol,
        );
        set_buffer_at(
            screen_buffer,
            screen_width,
            screen_height - 1,
            col,
            lower_vert_border_symbol,
        );
    }
}

fn add_snake_to_buffer(
    screen_buffer: &mut Vec<char>,
    snake: &Vec<Coordinate>,
    screen_width: usize,
) {
    let snake_head_symbol = '@';
    let snake_body_symbol = 'o';

    set_buffer_at(
        screen_buffer,
        screen_width,
        snake[0].row,
        snake[0].col,
        snake_head_symbol,
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
            snake_body_symbol,
        )
    }
}

fn add_header_to_buffer(screen_buffer: &mut Vec<char>, screen_width: usize, message: &str) {
    let str_chars = message.chars();
    let str_len = str_chars.clone().count();
    let header_start_idx = ((screen_width - str_len) / 2 as usize).max(0);

    let mut col_idx = header_start_idx;

    for sym in str_chars {
        set_buffer_at(screen_buffer, screen_width, 0, col_idx, sym);
        col_idx += 1;
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

fn move_snake(snake: &mut Vec<Coordinate>, snake_direction: i64) {
    // add head in new direction
    let new_head = match snake_direction {
        0 => Coordinate {
            row: snake[0].row - 1,
            col: snake[0].col,
        }, // up
        1 => Coordinate {
            row: snake[0].row,
            col: snake[0].col + 1,
        }, // right
        2 => Coordinate {
            row: snake[0].row + 1,
            col: snake[0].col,
        }, // down
        3 => Coordinate {
            row: snake[0].row,
            col: snake[0].col - 1,
        }, // left
        _ => Coordinate {
            row: snake[0].row,
            col: snake[0].col,
        }, // no movement at all, invalid direction
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

fn main() {
    let screen_width = 80;
    let screen_height = 30;
    let mut screen_buffer = vec!['.'; screen_width * screen_height];

    // clear screen
    clear_screen_buffer(&mut screen_buffer);

    let mut food_pos = Coordinate { row: 10, col: 15 };

    let mut snake = vec![
        Coordinate { row: 18, col: 15 },
        Coordinate { row: 19, col: 15 },
        Coordinate { row: 20, col: 15 },
    ];

    // 0: up, 1: right, 2: down, 3: left
    let mut snake_direction = 0;

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let device_state = DeviceState::new();
        let mut prev_keys = device_state.get_keys();

        loop {
            let keys = device_state.get_keys();
            if !keys.is_empty() && keys != prev_keys {
                tx.send(keys.clone()).unwrap();
            }
            prev_keys = keys.clone();
        }
    });
    let mut score = 0;
    loop {
        let sleep_time = if snake_direction % 2 == 1 { 120 } else { 200 };
        thread::sleep(Duration::from_millis(sleep_time));

        // move snake
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
            clear_screen_buffer(&mut screen_buffer);
            draw_screen_buffer(&screen_buffer, screen_width, screen_height);
            draw_screen_buffer(&screen_buffer, screen_width, screen_height);
            println!("Snake has hit something - Game over!");
            println!("Total score: {}", score);
            break;
        }

        // clear, update and draw screen buffer
        clear_screen_buffer(&mut screen_buffer);

        set_buffer_at(
            &mut screen_buffer,
            screen_width,
            food_pos.row,
            food_pos.col,
            'O',
        );

        add_snake_to_buffer(&mut screen_buffer, &snake, screen_width);
        add_game_border_to_buffer(&mut screen_buffer, screen_width, screen_height);
        add_header_to_buffer(
            &mut screen_buffer,
            screen_width,
            &format!("SNAKE - Score: {}", score),
        );
        draw_screen_buffer(&screen_buffer, screen_width, screen_height);

        let keys = &rx.try_recv();
        if !keys.is_err() {
            let keys = keys.clone().unwrap();

            if !keys.is_empty() && keys.last().is_some() {
                let last_key_un = keys.last().unwrap();
                match last_key_un {
                    device_query::Keycode::Q => {
                        break;
                    }
                    device_query::Keycode::Escape => {
                        break;
                    }
                    device_query::Keycode::A => snake_direction -= 1,
                    device_query::Keycode::D => snake_direction += 1,
                    _ => {}
                }
            }
        }
        snake_direction = match snake_direction {
            -1 => 3,
            _ => snake_direction % 4,
        };
    }
}
