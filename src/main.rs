#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use rand::seq::SliceRandom;
use std::io::{stdout, Write};
use std::time;

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    style::{self, SetBackgroundColor, SetForegroundColor},
    terminal, ExecutableCommand, Result,
};
use style::{
    Color::{Black, White},
    Print,
};

const XMAX: usize = 12;
const YMAX: usize = 18;
const FIELD_SIZE: usize = (XMAX * YMAX);
const XMARGIN: usize = 5;
const YMARGIN: usize = 2;
const PIXEL_EMPTY: u8 = b' ';
const PIXEL_SOLID: u8 = b'X';
const PIXEL_BORDER: u8 = b'#';
const BRICKS: [&[u8]; 7] = [
    b" X   X   X   X  ",
    b" XX  XX         ",
    b" X   X   XX     ",
    b"  X   X  XX     ",
    b"XX   XX         ",
    b"  XX XX         ",
    b" X   XX  X      ",
];

fn main() -> Result<()> {
    let interval = time::Duration::from_millis(200);
    // let now = time::Instant::now();

    let _ = terminal::enable_raw_mode()?;
    let mut is_bucket_full: bool = false;
    let mut is_brick_falling: bool;
    let mut is_free_fall: bool;
    let mut is_paused = false;

    let mut field: [u8; FIELD_SIZE] = [PIXEL_EMPTY; FIELD_SIZE];
    let mut score: usize = 0;
    let mut stdout = stdout();

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    // Initialize play field

    for y in 0..YMAX {
        field[(0 + y * XMAX) as usize] = PIXEL_BORDER;
        field[(XMAX - 1 + y * XMAX) as usize] = PIXEL_BORDER;
    }

    for x in 0..XMAX {
        field[(x + (YMAX - 1) * XMAX) as usize] = PIXEL_BORDER;
    }

    // Paint once

    stdout
        .execute(SetBackgroundColor(Black))?
        .execute(SetForegroundColor(White))?
        .execute(cursor::MoveTo(
            (XMAX + XMARGIN + 10) as u16,
            (0 + YMARGIN + 2) as u16,
        ))?
        .execute(Print("SCORE:"))?
        .flush()?;

    // Game loop

    while !is_bucket_full {
        let mut brick = BRICKS.choose(&mut rand::thread_rng()).expect("Raarrrr!");
        let mut y_brick: usize = 0;
        let mut x_brick: usize = XMAX / 2 - 2;
        is_brick_falling = true;
        is_free_fall = false;

        // Per-brick loop

        while is_brick_falling {
            // thread::sleep(interval);

            // Clean up block before movement

            for yb in 0..4 {
                for xb in 0..4 {
                    if brick[xb + yb * 4] == PIXEL_SOLID {
                        field[(x_brick + xb) + (y_brick + yb) * XMAX] = b" "[0];
                    }
                }
            }

            // Get user input

            if !is_free_fall && poll(interval)? {
                let event = read()?;
                if event == Event::Key(KeyCode::Char('q').into()) {
                    let _ = terminal::disable_raw_mode()?;
                    panic!("Quit!");
                }
                if event == Event::Key(KeyCode::Char('p').into()) {
                    is_paused = !is_paused;
                }
                if event == Event::Key(KeyCode::Left.into()) && x_brick > 1 {
                    // TODO: here we need collison detecktion, because the brick's
                    // x=0 axis may be empty an should not hit the left wall!
                    x_brick -= 1;
                }
                if event == Event::Key(KeyCode::Right.into()) && x_brick < XMAX - 3 {
                    // TODO: here we need collison detecktion, because the brick's
                    // x=3 axis may be empty an should not hit the right wall!
                    x_brick += 1;
                }
                if event == Event::Key(KeyCode::Up.into()) {
                    // rotate 90deg cw
                }
                if event == Event::Key(KeyCode::Down.into()) {
                    is_free_fall = true;
                }
            }

            if is_paused {
                continue;
            }

            // Move stuff

            y_brick += 1;

            // Collision detection

            for yb in 0..4 {
                for xb in 0..4 {
                    let field_idx = (x_brick + xb) + (y_brick + yb) * XMAX;
                    if field_idx < FIELD_SIZE {
                        let brick_pixel = brick[xb + yb * 4];
                        let field_pixel = field[field_idx];
                        if brick_pixel == PIXEL_SOLID && field_pixel != PIXEL_EMPTY {
                            is_brick_falling = false;
                            y_brick -= 1;
                        }
                    }
                }
            }

            // Bucket full detection

            if y_brick == 0 {
                is_bucket_full = true;
            }

            // Update field

            for yb in 0..4 {
                for xb in 0..4 {
                    if brick[xb + yb * 4] == PIXEL_SOLID {
                        field[(x_brick + xb) + (y_brick + yb) * XMAX] = PIXEL_SOLID;
                    }
                }
            }

            // Paint

            for y in 0..YMAX {
                for x in 0..XMAX {
                    let i = (x + y * XMAX) as usize;
                    stdout
                        .execute(cursor::MoveTo((x + XMARGIN) as u16, (y + YMARGIN) as u16))?
                        .execute(Print(field[i] as char))?
                        .flush()?;
                }
            }
        }
    }

    let _ = terminal::disable_raw_mode()?;
    Ok(())
}
