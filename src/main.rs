#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use rand::Rng;
use std::io::{stdout, Write};
use std::process::exit;
use std::thread;
use std::time;

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal, ExecutableCommand, Result,
};

const XMAX: isize = 12;
const YMAX: isize = 18;
const FIELD_SIZE: isize = (XMAX * YMAX);
const XMARGIN: isize = 5;
const YMARGIN: isize = 2;

const XSCORE: isize = (XMAX + XMARGIN + 10);
const YSCORE: isize = (0 + YMARGIN + 2);

const PIXEL_EMPTY: u8 = b' ';
const PIXEL_SOLID: u8 = b'X';
const PIXEL_BORDER: u8 = b'#';
const BRICKMAX: usize = 4;
const BRICKS: [&[u8; (BRICKMAX * BRICKMAX) as usize]; 7] = [
    b" X   X   X   X  ", // |
    b" XX  XX         ", // o
    b" X   X   XX     ", // |_
    b"  X   X  XX     ", // _|
    b"XX   XX         ", // Z
    b"  XX XX         ", // S
    b" X   XX  X      ", // T
];
const BRICKCOLORS: [Color; 7] = [
    Color::Cyan,
    Color::Green,
    Color::Blue,
    Color::Red,
    Color::Yellow,
    Color::Magenta,
    Color::Grey,
];

type Field = [u8; FIELD_SIZE as usize];

fn main() -> Result<()> {
    let _ = terminal::enable_raw_mode()?;
    let mut rng = rand::thread_rng();

    let mut is_bucket_full: bool = false;
    let mut is_brick_falling: bool;
    let mut is_free_fall: bool = false;
    let mut is_paused = false;

    let mut field: Field = [PIXEL_EMPTY; FIELD_SIZE as usize];
    let mut previous_field: Field = [PIXEL_EMPTY; FIELD_SIZE as usize];

    let mut score: usize = 0;
    let mut brick: usize = rng.gen_range(0, BRICKS.len());
    let mut rotation: usize = 0; // 1=90deg, 2=180deg, 3=270deg
    let rotate_cw: bool = false;
    let mut y_brick: isize = 0;
    let mut x_brick: isize = XMAX / 2 - (BRICKMAX / 2) as isize;
    let mut previous_y_brick: isize = y_brick;
    let mut previous_x_brick: isize = x_brick;
    let mut previous_rotation: usize = rotation;

    let game_tick: usize = 50;
    let tick_threshold: usize = 5;
    let mut tick_count: usize = 0;
    let mut move_brick: bool;

    let interval = time::Duration::from_millis(game_tick as u64);
    let delay_delete_row = time::Duration::from_millis(200);

    let mut stdout = stdout();

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    // Draw field border

    for y in 0..YMAX {
        field[(0 + y * XMAX) as usize] = PIXEL_BORDER;
        field[(XMAX - 1 + y * XMAX) as usize] = PIXEL_BORDER;
    }
    for x in 0..XMAX {
        field[(x + (YMAX - 1) * XMAX) as usize] = PIXEL_BORDER;
    }

    stdout
        .execute(SetBackgroundColor(Color::Black))?
        .execute(SetForegroundColor(Color::White))?
        .flush()?;

    // Game loop

    while !is_bucket_full {
        tick_count += 1;
        move_brick = tick_count == tick_threshold;

        // Get user input

        if !is_free_fall && poll(interval)? {
            let event = read()?;

            if event == Event::Key(KeyCode::Esc.into()) {
                let _ = terminal::disable_raw_mode()?;
                exit(0x0);
            }
            if event == Event::Key(KeyCode::Char('p').into()) {
                is_paused = !is_paused;
            }
            if event == Event::Key(KeyCode::Left.into()) {
                if no_collision(&field, brick, rotation, x_brick - 1, y_brick) {
                    x_brick -= 1;
                }
            }
            if event == Event::Key(KeyCode::Right.into()) {
                if no_collision(&field, brick, rotation, x_brick + 1, y_brick) {
                    x_brick += 1;
                }
            }
            if event == Event::Key(KeyCode::Up.into()) {
                let new_rot = if rotate_cw {
                    if rotation == 3 {
                        0
                    } else {
                        rotation + 1
                    }
                } else {
                    if rotation == 0 {
                        3
                    } else {
                        rotation - 1
                    }
                };
                if no_collision(&field, brick, new_rot, x_brick, y_brick) {
                    rotation = new_rot;
                }
            }
            if event == Event::Key(KeyCode::Down.into()) {
                is_free_fall = true;
            }
        }

        if is_paused {
            continue;
        }

        // Move stuff

        if move_brick {
            if no_collision(&field, brick, rotation, x_brick, y_brick + 1) {
                y_brick += 1;
            } else {
                // Add brick to the field

                for yb in 0..BRICKMAX {
                    for xb in 0..BRICKMAX {
                        let brick_idx = rotate(xb, yb, rotation);
                        let field_idx =
                            ((x_brick + xb as isize) + (y_brick + yb as isize) * XMAX) as usize;
                        if BRICKS[brick][brick_idx] == PIXEL_SOLID {
                            field[field_idx] = brick as u8 + 65;
                        }
                    }
                }

                // Check for filled lines and delete them

                for y in 0..(YMAX - 1) {
                    let field_l = (1 + y * XMAX) as usize;
                    let field_r = field_l + XMAX as usize - 2;
                    let mut full_row = true;
                    for x in field_l..field_r {
                        if field[x] == PIXEL_EMPTY {
                            full_row = false;
                        }
                    }
                    if full_row {
                        // Visual effect, draw it right here
                        stdout
                            .execute(cursor::MoveTo((XMARGIN + 1) as u16, (YMARGIN + y) as u16))?
                            .execute(Print("=========="))?
                            .flush()?;

                        thread::sleep(delay_delete_row);
                        // Cut row out of playing field
                        for k in (1..(field_l - 1)).rev() {
                            if k % XMAX as usize != 0 && k % XMAX as usize != (XMAX as usize - 1) {
                                field[(k + (XMAX as usize))] = field[k];
                            }
                            if k < XMAX as usize {
                                field[k] = PIXEL_EMPTY;
                            }
                        }
                    }
                }

                // Select a new brick

                is_free_fall = false;
                move_brick = false;
                brick = rng.gen_range(0, BRICKS.len());

                x_brick = XMAX / 2 - (BRICKMAX / 2) as isize;
                y_brick = 0;
                rotation = 0;

                previous_x_brick = x_brick;
                previous_y_brick = y_brick;
                previous_rotation = rotation;

                // Entire bucket full detection

                if !no_collision(&field, brick, rotation, x_brick, y_brick) {
                    is_bucket_full = true;
                }
            }

            tick_count = 0;
        }

        // Paint field

        // Only draw if there was an actual change
        if field[..] != previous_field[..] {
            for y in 0..YMAX {
                for x in 0..XMAX {
                    let i = (x + y * XMAX) as usize;
                    stdout.execute(cursor::MoveTo((x + XMARGIN) as u16, (y + YMARGIN) as u16))?;
                    if let 65..=72 = field[i] {
                        stdout
                            .execute(SetBackgroundColor(BRICKCOLORS[field[i] as usize - 65]))?
                            .execute(Print(" "))?
                            .execute(SetBackgroundColor(Color::Black))?;
                    } else {
                        stdout.execute(Print(field[i] as char))?;
                    };
                }
            }
        }

        // Remember what the field looks like, to avoid repainting if nothing changes

        for k in 0..FIELD_SIZE as usize {
            previous_field[k] = field[k];
        }

        // Paint falling brick

        if move_brick
            && (previous_x_brick != x_brick
                || previous_y_brick != y_brick
                || previous_rotation != rotation)
        {
            for yb in 0..BRICKMAX as isize {
                for xb in 0..BRICKMAX as isize {
                    let pi = rotate(xb as usize, yb as usize, previous_rotation);
                    if BRICKS[brick][pi] == PIXEL_SOLID {
                        stdout
                            .execute(cursor::MoveTo(
                                (XMARGIN + previous_x_brick + xb) as u16,
                                (YMARGIN + previous_y_brick + yb) as u16,
                            ))?
                            .execute(Print(" "))?;
                    }
                }
            }

            previous_x_brick = x_brick;
            previous_y_brick = y_brick;
            previous_rotation = rotation;

            for yb in 0..BRICKMAX as isize {
                for xb in 0..BRICKMAX as isize {
                    let pi = rotate(xb as usize, yb as usize, rotation);
                    if BRICKS[brick][pi] == PIXEL_SOLID {
                        stdout
                            .execute(cursor::MoveTo(
                                (XMARGIN + x_brick + xb) as u16,
                                (YMARGIN + y_brick + yb) as u16,
                            ))?
                            .execute(SetBackgroundColor(BRICKCOLORS[brick]))?
                            .execute(Print(" "))?
                            .execute(SetBackgroundColor(Color::Black))?;
                    }
                }
            }
        }

        stdout
            .execute(cursor::MoveTo(XSCORE as u16, YSCORE as u16))?
            .execute(Print(format!("SCORE: {}", score)))?
            .flush()?;
    }

    let _ = terminal::disable_raw_mode()?;

    Ok(())
}

// Return the rotated position of one brick pixel
fn rotate(px: usize, py: usize, r: usize) -> usize {
    match r % 4 {
        0 => py * BRICKMAX + px,
        1 => 12 + py - (px * BRICKMAX),
        2 => 15 - (py * BRICKMAX) - px,
        3 => 3 - py + (px * BRICKMAX),
        _ => 0,
    }
}

fn no_collision(field: &Field, brick: usize, r: usize, x_brick: isize, y_brick: isize) -> bool {
    for yb in 0..BRICKMAX {
        for xb in 0..BRICKMAX {
            let brick_idx = rotate(xb, yb, r);
            let field_idx = ((x_brick + xb as isize) + (y_brick + yb as isize) * XMAX) as usize;

            if x_brick + (xb as isize) >= 0 && x_brick + (xb as isize) < XMAX {
                if y_brick + (yb as isize) >= 0 && y_brick + (yb as isize) < YMAX {
                    let brick_pixel = BRICKS[brick][brick_idx];
                    let field_pixel = field[field_idx];
                    if brick_pixel != PIXEL_EMPTY && field_pixel != PIXEL_EMPTY {
                        return false;
                    }
                }
            }
        }
    }
    return true;
}
