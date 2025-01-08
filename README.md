![Build Status](https://github.com/baurst/rs_snake/actions/workflows/rust.yml/badge.svg)
# Snake ...

... with your friends ...

... on your command line!

This is an implementation of the classic game Snake in Rust.
It runs in all UNIX and Windows terminals without using any GUI framework or window manager.

![Demo](img/snake.gif)

## Try it out!

### Using prebuilt binaries

We provide prebuilt binaries for both Windows and Linux.
So if you just want to play a quick game you can head over to the Releases section and download the latest release for your platform: [Latest Release](https://github.com/baurst/rs_snake/releases/latest)

### Building from Source

Building it yourself is straightforward:

```bash
git clone https://github.com/baurst/rs_snake.git
cd rs_snake
cargo run --release
```

## Controls

At the moment, up to two players are supported. The controls for making the snake turn left or right are:

* Player 1: arrow keys
* Player 2: WASD keys

Pressing Esc or q will terminate the game.

## Options

The game provides options to change to __multiplayer__ mode (using __--multi__).

Difficulty of the game (i.e. speed of the snake) is adjustable using either __--easy__ or __--hard__:

```
$ rs_snake --help
snake 0.3.0
Author: baurst
Classic snake game for your terminal

USAGE:
    rs_snake [FLAGS]

FLAGS:
    -e, --easy                sets difficulty to easy
    -d, --hard                sets difficulty to hard
    -h  --help                Prints help information
    -m, --multi               enables multiplayer mode
    -t, --two_key_steering    steer the snakes using two keys only (increased difficulty)
    -V, --version             Prints version information
```
