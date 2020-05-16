#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use std::io::{stdout, Write};
use std::{thread, time};
use rand::seq::SliceRandom;

use crossterm::{
    cursor,
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
const PIXEL_EMPTY: u8 = b" "[0];
const PIXEL_SOLID: u8 = b"X"[0];
const PIXEL_BORDER: u8 = b"#"[0];
const BRICKS: [&[u8]; 7] = [
    r#" X   X   X   X  "#.as_bytes(),
    r#" XX  XX         "#.as_bytes(),
    r#" X   X   XX     "#.as_bytes(),
    r#"  X   X  XX     "#.as_bytes(),
    r#"XX   XX         "#.as_bytes(),
    r#"  XX XX         "#.as_bytes(),
    r#" X   XX  X      "#.as_bytes(),
];

fn main() -> Result<()> {
    let interval = time::Duration::from_millis(200);
    // let now = time::Instant::now();

    let mut bucket_full: bool = false;
    let mut brick_falling: bool;
    let mut field: [u8; FIELD_SIZE] = [PIXEL_EMPTY; FIELD_SIZE];
    let mut stdout = stdout();
    let mut score: usize = 0;

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

    while !bucket_full {
        let mut brick = BRICKS.choose(&mut rand::thread_rng()).expect("Raarrrr!");
        let mut y_brick: usize = 0;
        let mut x_brick: usize = XMAX / 2 - 2;
        brick_falling = true;

        // Per-brick loop

        while brick_falling {
            thread::sleep(interval);

            // Clean up block before movement

            for yb in 0..4 {
                for xb in 0..4 {
                    if brick[xb + yb * 4] == PIXEL_SOLID {
                        field[(x_brick + xb) + (y_brick + yb) * XMAX] = b" "[0];
                    }
                }
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
                            brick_falling = false;
                            y_brick -= 1;
                        }
                    }
                }
            }

            // Bucket full detection

            if y_brick == 0 {
                bucket_full = true;
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

    Ok(())
}
