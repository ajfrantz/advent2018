use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use failure::Error;
use regex::Regex;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Coordinate {
    x: i64,
    y: i64,
    z: i64,
}

impl Coordinate {
    fn dist(&self, other: &Coordinate) -> i64 {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        let dz = (self.z - other.z).abs();
        (dx + dy + dz)
    }
}

#[derive(Debug, Copy, Clone)]
struct Nanobot {
    pos: Coordinate,
    r: i64,
}

impl Nanobot {
    fn in_range(&self, pos: &Coordinate) -> bool {
        self.pos.dist(pos) <= self.r
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct AABB {
    min: Coordinate,
    max: Coordinate,
}

fn clamp<T: Ord>(input: T, min: T, max: T) -> T {
    std::cmp::min(std::cmp::max(input, min), max)
}

impl AABB {
    fn span(&self) -> i64 {
        self.min.dist(&self.max)
    }

    fn split_x(&self) -> (AABB, AABB) {
        let x_len = self.max.x - self.min.x;
        let x_mid = self.min.x + x_len / 2;

        (
            AABB {
                min: self.min,
                max: Coordinate {
                    x: x_mid,
                    ..self.max
                },
            },
            AABB {
                min: Coordinate {
                    x: std::cmp::min(x_mid + 1, self.max.x),
                    ..self.min
                },
                max: self.max,
            },
        )
    }

    fn split_y(&self) -> (AABB, AABB) {
        let y_len = self.max.y - self.min.y;
        let y_mid = self.min.y + y_len / 2;

        (
            AABB {
                min: self.min,
                max: Coordinate {
                    y: y_mid,
                    ..self.max
                },
            },
            AABB {
                min: Coordinate {
                    y: std::cmp::min(y_mid + 1, self.max.y),
                    ..self.min
                },
                max: self.max,
            },
        )
    }

    fn split_z(&self) -> (AABB, AABB) {
        let z_len = self.max.z - self.min.z;
        let z_mid = self.min.z + z_len / 2;

        (
            AABB {
                min: self.min,
                max: Coordinate {
                    z: z_mid,
                    ..self.max
                },
            },
            AABB {
                min: Coordinate {
                    z: std::cmp::min(z_mid + 1, self.max.z),
                    ..self.min
                },
                max: self.max,
            },
        )
    }

    fn split(&self) -> (AABB, AABB) {
        let x_len = self.max.x - self.min.x;
        let y_len = self.max.y - self.min.y;
        let z_len = self.max.z - self.min.z;

        if x_len > y_len {
            if x_len > z_len {
                self.split_x()
            } else {
                self.split_z()
            }
        } else if y_len > z_len {
            self.split_y()
        } else {
            self.split_z()
        }
    }

    fn clamp(&self, pos: &Coordinate) -> Coordinate {
        Coordinate {
            x: clamp(pos.x, self.min.x, self.max.x),
            y: clamp(pos.y, self.min.y, self.max.y),
            z: clamp(pos.z, self.min.z, self.max.z),
        }
    }

    fn in_range(&self, bot: &Nanobot) -> bool {
        let closest_point = self.clamp(&bot.pos);
        closest_point.dist(&bot.pos) <= bot.r
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Search {
    value: usize,
    volume: AABB,
}

impl Ord for Search {
    fn cmp(&self, other: &Search) -> Ordering {
        // Order first by value, then by distance from 0,0,0 per puzzle requirement.
        let origin = Coordinate { x: 0, y: 0, z: 0 };
        self.value.cmp(&other.value).then_with(|| {
            let self_dist = origin.dist(&self.volume.min);
            let other_dist = origin.dist(&other.volume.min);
            other_dist.cmp(&self_dist)
        })
    }
}

impl PartialOrd for Search {
    fn partial_cmp(&self, other: &Search) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1])?;

    let re = Regex::new(r"pos=<(-?\d+),(-?\d+),(-?\d+)>, r=(\d+)")?;

    let nanobots: Vec<Nanobot> = BufReader::new(file)
        .lines()
        .map(|l| l.unwrap())
        .map(|l| {
            let captures = re.captures(&l).unwrap();
            Nanobot {
                pos: Coordinate {
                    x: captures[1].parse().unwrap(),
                    y: captures[2].parse().unwrap(),
                    z: captures[3].parse().unwrap(),
                },
                r: captures[4].parse().unwrap(),
            }
        })
        .collect();

    let strongest = nanobots.iter().max_by_key(|bot| bot.r).unwrap();
    let in_range = nanobots
        .iter()
        .filter(|bot| strongest.in_range(&bot.pos))
        .count();
    println!("first answer: {}", in_range);

    let x_min = nanobots.iter().min_by_key(|bot| bot.pos.x).unwrap().pos.x;
    let x_max = nanobots.iter().max_by_key(|bot| bot.pos.x).unwrap().pos.x;
    let y_min = nanobots.iter().min_by_key(|bot| bot.pos.y).unwrap().pos.y;
    let y_max = nanobots.iter().max_by_key(|bot| bot.pos.y).unwrap().pos.y;
    let z_min = nanobots.iter().min_by_key(|bot| bot.pos.z).unwrap().pos.z;
    let z_max = nanobots.iter().max_by_key(|bot| bot.pos.z).unwrap().pos.z;
    let volume = AABB {
        min: Coordinate {
            x: x_min,
            y: y_min,
            z: z_min,
        },
        max: Coordinate {
            x: x_max,
            y: y_max,
            z: z_max,
        },
    };
    let in_range = nanobots.iter().filter(|bot| volume.in_range(bot)).count();

    let mut heap = BinaryHeap::new();
    heap.push(Search {
        value: in_range,
        volume,
    });

    while let Some(search) = heap.pop() {
        if search.volume.span() == 0 {
            println!("solution: {:?}", search);

            let volume = search.volume;
            let in_range = nanobots.iter().filter(|bot| volume.in_range(bot)).count();
            let answer = volume.min.x + volume.min.y + volume.min.z;
            println!(
                "{:?} is in range of {} bots [answer: {}]",
                volume.min, in_range, answer
            );
            return Ok(());
        }

        let (a, b) = search.volume.split();
        let in_range_of_a = nanobots.iter().filter(|bot| a.in_range(bot)).count();
        let in_range_of_b = nanobots.iter().filter(|bot| b.in_range(bot)).count();

        heap.push(Search {
            value: in_range_of_a,
            volume: a,
        });
        heap.push(Search {
            value: in_range_of_b,
            volume: b,
        });
    }

    panic!("search failed to converge");
}
