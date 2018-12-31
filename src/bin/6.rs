extern crate failure;
extern crate regex;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use failure::Error;

#[derive(Copy, Clone, Debug, PartialEq)]
struct Coordinate {
    x: i32,
    y: i32,
}

#[derive(Copy, Clone, Debug)]
struct Cell {
    closest: usize,
    distance: i32,
}

fn distance((x1, y1): (i32, i32), (x2, y2): (i32, i32)) -> i32 {
    (x1 - x2).abs() + (y1 - y2).abs()
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1])?;

    let locations: Vec<Coordinate> = BufReader::new(file)
        .lines()
        .map(|l| l.expect("file read failed"))
        .map(|l| {
            l.split(", ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .map(|parts| Coordinate {
            x: parts[0].parse().unwrap(),
            y: parts[1].parse().unwrap(),
        })
        .collect();

    // Let's make a square grid large enough to hold all the locations.
    let max_x = locations.iter().map(|l| l.x).max().unwrap();
    let max_y = locations.iter().map(|l| l.y).max().unwrap();
    let side = max_x.max(max_y) as usize;

    let mut grid: Vec<Vec<Option<Cell>>> = vec![vec![None; side]; side];
    let mut areas = HashMap::new();

    // Naively blast over the grid and brute force the nearest point.
    for x in 0..side {
        for y in 0..side {
            let cell = (x as i32, y as i32);
            let distances: Vec<_> = locations
                .iter()
                .enumerate()
                .map(|(index, coord)| (index, distance(cell, (coord.x, coord.y))))
                .collect();
            let (min_index, min_distance) = distances
                .iter()
                .min_by_key(|(_, dist)| dist)
                .unwrap()
                .clone();
            if distances
                .iter()
                .filter(|(_, dist)| *dist == min_distance)
                .count()
                == 1
            {
                grid[x][y] = Some(Cell {
                    closest: min_index,
                    distance: min_distance,
                });

                *areas.entry(min_index).or_insert(0) += 1;
            }
        }
    }

    // Eliminate zones touching the edges.
    for x in 0..side {
        if let Some(Cell { closest, .. }) = grid[x][0] {
            areas.remove(&closest);
        }
        if let Some(Cell { closest, .. }) = grid[x][side - 1] {
            areas.remove(&closest);
        }
    }
    for y in 0..side {
        if let Some(Cell { closest, .. }) = grid[0][y] {
            areas.remove(&closest);
        }
        if let Some(Cell { closest, .. }) = grid[side - 1][y] {
            areas.remove(&closest);
        }
    }

    let answer = areas.iter().max_by_key(|(_, &area)| area).unwrap();

    println!("first answer: {}", answer.1);

    // Do it again, but with total distance this time.
    let mut area = 0;
    for x in 0..side {
        for y in 0..side {
            let cell = (x as i32, y as i32);
            let total_distance: i32 = locations
                .iter()
                .map(|coord| distance(cell, (coord.x, coord.y)))
                .sum();
            if total_distance < 10000 {
                area += 1;
            }
        }
    }

    println!("second answer: {}", area);

    Ok(())
}
