# Snake ...

... with your firends ...

... on your command line!


[![Build Status](https://travis-ci.com/baurst/rs_snake.svg?token=KGmoNyosUqTq92iqGZE9&branch=master)](https://travis-ci.com/baurst/rs_snake) 


This is an implementation of the classic game Snake in Rust.
It runs in all UNIX and Windows terminals without using any GUI framework or window manager.

![Demo](img/snake.png)

## Try it out!
### Using prebuilt binaries
We provide prebuilt binaries for both Windows and Linux.
So if you just want to play a quick game you can head over to the Releases section and download the latest release for your platform: [Latest Release](https://github.com/baurst/rs_snake/releases/latest)


### Building from Source
Building should also be straightforward:
```bash
git clone https://github.com/baurst/rs_snake.git
cd rs_snake
cargo run --release
```

## Controls
At the moment, up to two players are supported. The controls for making the snake turn left or right are:
* Player 1: :arrow_left: and :arrow_right: arrow keys
* Player 2: A and D keys
Pressing Esc or q will terminate the game.


## Options
The game provides multiple options, all of which should be self explanatory when looking at the following help text:
```bash
$: snake --help
snake 0.1
Author: baurst
Classic snake game for your terminal

USAGE:
    rs_snake [FLAGS]

FLAGS:
    -e, --easy       sets difficulty to easy
    -h, --hard       sets difficulty to hard
        --help       Prints help information
    -m, --multi      enables multiplayer mode
    -V, --version    Prints version information
```
