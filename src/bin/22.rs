#[macro_use]
extern crate cached;
extern crate pathfinding;

use failure::Error;
use pathfinding::prelude::{absdiff, astar};

// Should be large enough to find the best path...
const GRID_SIZE: usize = 2048;

// Example input.
// const DEPTH: usize = 510;
// const TARGET: (usize, usize) = (10, 10);

// Puzzle input.
const DEPTH: usize = 11109;
const TARGET: (usize, usize) = (9, 731);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum RegionType {
    Rocky,
    Narrow,
    Wet,
}

cached! {
    GEO_INDEX;
    fn geologic_index(x: usize, y: usize) -> usize = {
        match (x, y) {
            (0, 0) => 0,
            TARGET => 0,
            (x, 0) => x * 16807,
            (0, y) => y * 48271,
            _ => erosion_level(x - 1, y) * erosion_level(x, y - 1),
        }
    }
}

fn erosion_level(x: usize, y: usize) -> usize {
    (geologic_index(x, y) + DEPTH) % 20183
}

fn region_type(x: usize, y: usize) -> RegionType {
    match erosion_level(x, y) % 3 {
        0 => RegionType::Rocky,
        1 => RegionType::Wet,
        2 => RegionType::Narrow,
        _ => unreachable!(),
    }
}

fn draw(grid: &Vec<Vec<RegionType>>) {
    for y in 0..16 {
        for x in 0..16 {
            print!(
                "{}",
                match grid[y][x] {
                    RegionType::Rocky => '.',
                    RegionType::Wet => '=',
                    RegionType::Narrow => '|',
                }
            );
        }
        print!("\n");
    }
}

fn risk_level(
    grid: &Vec<Vec<RegionType>>,
    top_left: (usize, usize),
    bottom_right: (usize, usize),
) -> usize {
    let mut risk = 0;
    for y in top_left.1..=bottom_right.1 {
        for x in top_left.0..=bottom_right.0 {
            risk += match grid[y][x] {
                RegionType::Rocky => 0,
                RegionType::Wet => 1,
                RegionType::Narrow => 2,
            }
        }
    }
    risk
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Tool {
    Torch,
    ClimbingGear,
    Neither,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct State {
    x: usize,
    y: usize,
    tool: Tool,
}

impl State {
    fn heuristic(&self, other: &State) -> usize {
        absdiff(self.x, other.x)
            + absdiff(self.y, other.y)
            + if self.tool != other.tool { 7 } else { 0 }
    }

    fn successors(&self, grid: &Vec<Vec<RegionType>>) -> Vec<(State, usize)> {
        let mut successors = Vec::new();

        // We can stay in place and change tool.
        for &tool in &[Tool::Torch, Tool::ClimbingGear, Tool::Neither] {
            if tool != self.tool {
                successors.push((State { tool, ..*self }, 7));
            }
        }

        // Or we can keep our current tool, but try to move.
        let x = self.x;
        let y = self.y;
        for target in &[(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if target.0 > GRID_SIZE || target.1 > GRID_SIZE {
                continue;
            }
            match (grid[target.1][target.0], self.tool) {
                (RegionType::Rocky, Tool::Torch)
                | (RegionType::Rocky, Tool::ClimbingGear)
                | (RegionType::Wet, Tool::ClimbingGear)
                | (RegionType::Wet, Tool::Neither)
                | (RegionType::Narrow, Tool::Torch)
                | (RegionType::Narrow, Tool::Neither) => {
                    successors.push((
                        State {
                            x: target.0,
                            y: target.1,
                            ..*self
                        },
                        1,
                    ));
                }
                _ => (),
            }
        }

        successors
    }
}

fn solve(grid: &Vec<Vec<RegionType>>) -> usize {
    let start = State {
        x: 0,
        y: 0,
        tool: Tool::Torch,
    };
    let goal = State {
        x: TARGET.0,
        y: TARGET.1,
        tool: Tool::Torch,
    };

    let result = astar(
        &start,
        |n| n.successors(&grid),
        |n| n.heuristic(&goal),
        |n| *n == goal,
    );

    if let Some((_, total_cost)) = result {
        return total_cost;
    }

    panic!("a-star failed");
}

fn main() -> Result<(), Error> {
    let mut grid = vec![vec![RegionType::Rocky; GRID_SIZE]; GRID_SIZE];

    for y in 0..2048 {
        for x in 0..2048 {
            grid[y][x] = region_type(x, y);
        }
    }
    //draw(&grid);

    println!("risk level: {}", risk_level(&grid, (0, 0), TARGET));
    println!("shortest path: {}", solve(&grid));

    Ok(())
}
