use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use failure::{format_err, Error};
use regex::Regex;

#[derive(Debug, Copy, Clone)]
struct Point {
    position: (i32, i32),
    velocity: (i32, i32),
}

struct Rect {
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
}

impl Rect {
    fn new() -> Rect {
        Rect {
            left: std::i32::MAX,
            right: std::i32::MIN,
            top: std::i32::MAX,
            bottom: std::i32::MIN,
        }
    }

    fn width(&self) -> i32 {
        self.right - self.left + 1
    }

    fn height(&self) -> i32 {
        self.bottom - self.top + 1
    }

    fn area(&self) -> i64 {
        self.width() as i64 * self.height() as i64
    }
}

fn propagate(input: &Vec<Point>, steps: i32) -> Vec<Point> {
    input
        .iter()
        .map(|Point { position, velocity }| Point {
            position: (
                position.0 + steps * velocity.0,
                position.1 + steps * velocity.1,
            ),
            velocity: *velocity,
        })
        .collect()
}

fn bounds(pattern: &Vec<Point>) -> Rect {
    pattern.iter().fold(Rect::new(), |acc, p| Rect {
        left: std::cmp::min(acc.left, p.position.0),
        right: std::cmp::max(acc.right, p.position.0),
        top: std::cmp::min(acc.top, p.position.1),
        bottom: std::cmp::max(acc.bottom, p.position.1),
    })
}

fn draw(pattern: &Vec<Point>) {
    let rect = bounds(pattern);
    let mut grid = vec![vec![' '; rect.width() as usize]; rect.height() as usize];
    pattern.iter().for_each(|p| {
        let x = (p.position.0 - rect.left) as usize;
        let y = (p.position.1 - rect.top) as usize;
        grid[y][x] = '#';
    });
    for row in grid.iter() {
        for c in row.iter() {
            print!("{}", c);
        }
        println!("");
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1])?;

    let pair = r"<\s*([^,]+),\s+([^>]+)>";
    let re = Regex::new(&format!("position={0} velocity={0}", pair))?;

    let points: Vec<Point> = BufReader::new(file)
        .lines()
        .map(|l| l.expect("file read failed"))
        .map(|l| {
            let captures = re.captures(&l).ok_or(format_err!("bad input: {}", l))?;
            Ok(Point {
                position: (captures[1].parse()?, captures[2].parse()?),
                velocity: (captures[3].parse()?, captures[4].parse()?),
            })
        })
        .map(|res: Result<Point, Error>| res.expect("failed to parse point"))
        .collect();

    let mut score = bounds(&points).area();
    for n in 1.. {
        let trial = bounds(&propagate(&points, n)).area();
        if trial > score {
            println!("found pattern at {} seconds", n - 1);
            draw(&propagate(&points, n - 1));
            break;
        }
        score = trial;
    }

    Ok(())
}
