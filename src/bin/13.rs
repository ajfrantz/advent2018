use std::env;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

use failure::Error;

#[derive(Debug, Copy, Clone)]
enum TrackType {
    Vertical,
    Horizontal,
    Clockwise,
    CounterClockwise,
    Junction,
}

impl TrackType {
    fn from(c: char) -> Option<TrackType> {
        match c {
            '|' | '^' | 'v' => Some(TrackType::Vertical),
            '-' | '<' | '>' => Some(TrackType::Horizontal),
            '/' => Some(TrackType::Clockwise),
            '\\' => Some(TrackType::CounterClockwise),
            '+' => Some(TrackType::Junction),
            _ => None,
        }
    }
}

impl fmt::Display for TrackType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&match self {
            TrackType::Vertical => "|",
            TrackType::Horizontal => "-",
            TrackType::Clockwise => "/",
            TrackType::CounterClockwise => "\\",
            TrackType::Junction => "+",
        })
    }
}

#[derive(Debug, Clone)]
struct Track {
    cells: Vec<Vec<Option<TrackType>>>,
}

impl Track {
    fn new(input: &str) -> Track {
        let mut cells = Vec::new();

        for line in input.split("\n") {
            let mut row = Vec::new();
            for c in line.chars() {
                row.push(TrackType::from(c));
            }
            cells.push(row);
        }

        Track { cells }
    }
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone)]
enum Rotation {
    Left,
    Straight,
    Right,
}

impl Rotation {
    fn next(&self) -> Rotation {
        match self {
            Rotation::Left => Rotation::Straight,
            Rotation::Straight => Rotation::Right,
            Rotation::Right => Rotation::Left,
        }
    }
}

impl Direction {
    fn offset(&self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    fn rotate(&self, rotation: Rotation) -> Direction {
        match (self, rotation) {
            (Direction::Up, Rotation::Left) => Direction::Left,
            (Direction::Up, Rotation::Straight) => Direction::Up,
            (Direction::Up, Rotation::Right) => Direction::Right,
            (Direction::Down, Rotation::Left) => Direction::Right,
            (Direction::Down, Rotation::Straight) => Direction::Down,
            (Direction::Down, Rotation::Right) => Direction::Left,
            (Direction::Left, Rotation::Left) => Direction::Down,
            (Direction::Left, Rotation::Straight) => Direction::Left,
            (Direction::Left, Rotation::Right) => Direction::Up,
            (Direction::Right, Rotation::Left) => Direction::Up,
            (Direction::Right, Rotation::Straight) => Direction::Right,
            (Direction::Right, Rotation::Right) => Direction::Down,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Cart {
    x: usize,
    y: usize,
    direction: Direction,
    next_turn: Rotation,
}

impl fmt::Display for Cart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&match self.direction {
            Direction::Up => "^",
            Direction::Down => "v",
            Direction::Left => "<",
            Direction::Right => ">",
        })
    }
}

impl Cart {
    fn traverse(&self, track: &Track) -> Cart {
        let offset = self.direction.offset();
        let x = (self.x as i32 + offset.0) as usize;
        let y = (self.y as i32 + offset.1) as usize;
        let target = track.cells[y][x].unwrap();
        match target {
            TrackType::Vertical | TrackType::Horizontal => Cart { x, y, ..*self },
            TrackType::Clockwise => Cart {
                x,
                y,
                direction: match self.direction {
                    Direction::Up => Direction::Right,
                    Direction::Down => Direction::Left,
                    Direction::Left => Direction::Down,
                    Direction::Right => Direction::Up,
                },
                ..*self
            },
            TrackType::CounterClockwise => Cart {
                x,
                y,
                direction: match self.direction {
                    Direction::Up => Direction::Left,
                    Direction::Down => Direction::Right,
                    Direction::Left => Direction::Up,
                    Direction::Right => Direction::Down,
                },
                ..*self
            },
            TrackType::Junction => Cart {
                x,
                y,
                direction: self.direction.rotate(self.next_turn),
                next_turn: self.next_turn.next(),
            },
        }
    }
}

#[derive(Debug, Clone)]
struct State {
    track: Track,
    carts: Vec<Cart>,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (y, row) in self.track.cells.iter().enumerate() {
            for (x, track) in row.iter().enumerate() {
                if let Some(track) = track {
                    if let Some(cart) = self.carts.iter().find(|c| c.x == x && c.y == y) {
                        write!(f, "{}", cart)?
                    } else {
                        write!(f, "{}", track)?
                    }
                } else {
                    write!(f, " ")?
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl State {
    fn new(input: &str) -> State {
        let mut carts = Vec::new();
        for (y, line) in input.split("\n").enumerate() {
            for (x, c) in line.chars().enumerate() {
                let direction = match c {
                    '^' => Direction::Up,
                    'v' => Direction::Down,
                    '<' => Direction::Left,
                    '>' => Direction::Right,
                    _ => continue,
                };
                carts.push(Cart {
                    x,
                    y,
                    direction,
                    next_turn: Rotation::Left,
                });
            }
        }

        State {
            track: Track::new(&input),
            carts,
        }
    }

    fn tick(mut self) -> State {
        // Enforce the movement order dictated by the problem.
        self.carts
            .sort_unstable_by_key(|c| (-(c.y as i32), -(c.x as i32)));

        // O(n^2) but for a small number of carts...
        let mut carts = Vec::new();
        while let Some(cart) = self.carts.pop() {
            let cart = cart.traverse(&self.track);

            let before = carts.len() + self.carts.len();
            self.carts.retain(|c: &Cart| c.x != cart.x || c.y != cart.y);
            carts.retain(|c: &Cart| c.x != cart.x || c.y != cart.y);
            let after = carts.len() + self.carts.len();

            if after < before {
                println!("collision at {},{}", cart.x, cart.y);
            } else {
                carts.push(cart);
            }
        }

        State { carts, ..self }
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let mut file = File::open(&args[1])?;
    let mut input = String::new();
    file.read_to_string(&mut input)?;

    let mut state = State::new(&input);
    println!("{}", state);

    while state.carts.len() > 1 {
        state = state.tick();
    }

    println!("{}", state);

    if let Some(survivor) = state.carts.first() {
        println!("The sole cart is at {},{}", survivor.x, survivor.y);
    }

    Ok(())
}
