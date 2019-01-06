use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use failure::Error;
use regex::Regex;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Cell {
    Sand,
    Clay,
    Reachable(bool, bool),
}

impl Cell {
    fn is_stable(self) -> bool {
        self == Cell::Clay || self == Cell::Reachable(true, true)
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1])?;

    let y_range_re = Regex::new(r"x=(\d+), y=(\d+)..(\d+)")?;
    let x_range_re = Regex::new(r"y=(\d+), x=(\d+)..(\d+)")?;

    let mut grid = vec![vec![Cell::Sand; 2048]; 2048];
    let mut y_min = 2048;
    let mut y_max = 0;

    for line in BufReader::new(file).lines().map(|l| l.unwrap()) {
        if let Some(captures) = y_range_re.captures(&line) {
            let x: usize = captures[1].parse()?;
            let y_start: usize = captures[2].parse()?;
            let y_end: usize = captures[3].parse()?;
            for y in y_start..=y_end {
                grid[y][x] = Cell::Clay;
                y_min = y_min.min(y);
                y_max = y_max.max(y);
            }
        } else if let Some(captures) = x_range_re.captures(&line) {
            let y: usize = captures[1].parse()?;
            let x_start: usize = captures[2].parse()?;
            let x_end: usize = captures[3].parse()?;
            for x in x_start..=x_end {
                grid[y][x] = Cell::Clay;
                y_min = y_min.min(y);
                y_max = y_max.max(y);
            }
        } else {
            panic!("bad input line: {}", line);
        }
    }

    // Color the grid with reachable cells.
    let mut active = VecDeque::new();
    grid[1][500] = Cell::Reachable(false, false);
    active.push_back((500, 1));

    while let Some((x, y)) = active.pop_front() {
        // Don't go outside the designated area.
        if y > y_max {
            continue;
        }

        let below = grid[y + 1][x];
        if below == Cell::Sand {
            // Since water could reach *this* spot, it can always go down from here.
            grid[y + 1][x] = Cell::Reachable(false, false);
            active.push_back((x, y + 1));
        } else if below.is_stable() {
            // Reachability propagates left-right if the space below it is stable.
            if grid[y][x - 1] == Cell::Sand {
                grid[y][x - 1] = Cell::Reachable(false, false);
                active.push_back((x - 1, y));
            }
            if grid[y][x + 1] == Cell::Sand {
                grid[y][x + 1] = Cell::Reachable(false, false);
                active.push_back((x + 1, y));
            }

            // Before fully retiring this cell, also evaluate stability.
            match (grid[y][x - 1], grid[y][x]) {
                (_, Cell::Reachable(true, _)) => (),
                (Cell::Reachable(true, _), Cell::Reachable(false, r))
                | (Cell::Clay, Cell::Reachable(false, r)) => {
                    // This cell is confirmed bounded left.
                    // Affects reachable cells above (reachable) or left ( stable).
                    grid[y][x] = Cell::Reachable(true, r);
                    if let Cell::Reachable(_, _) = grid[y - 1][x] {
                        active.push_back((x, y - 1));
                    }
                    if let Cell::Reachable(_, _) = grid[y][x + 1] {
                        active.push_back((x + 1, y));
                    }
                }
                _ => (),
            }

            match (grid[y][x], grid[y][x + 1]) {
                (Cell::Reachable(_, true), _) => (),
                (Cell::Reachable(l, false), Cell::Reachable(_, true))
                | (Cell::Reachable(l, false), Cell::Clay) => {
                    // This cell is confirmed bounded right.
                    // Affects reachable cells above (reachable) or left ( stable).
                    grid[y][x] = Cell::Reachable(l, true);
                    if let Cell::Reachable(_, _) = grid[y - 1][x] {
                        active.push_back((x, y - 1));
                    }
                    if let Cell::Reachable(_, _) = grid[y][x - 1] {
                        active.push_back((x - 1, y));
                    }
                }
                _ => (),
            }
        }
    }

    let answer = (y_min..=y_max)
        .flat_map(|y| grid[y].iter())
        .filter(|c| {
            if let Cell::Reachable(_, _) = c {
                true
            } else {
                false
            }
        })
        .count();
    println!("first answer: {}", answer);

    let answer = (y_min..=y_max)
        .flat_map(|y| grid[y].iter())
        .filter(|c| {
            if let Cell::Reachable(true, true) = c {
                true
            } else {
                false
            }
        })
        .count();
    println!("second answer: {}", answer);

    Ok(())
}
