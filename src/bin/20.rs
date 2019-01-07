use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs::File;
use std::io::prelude::*;

use failure::{format_err, Error};

#[derive(Debug, Clone)]
enum Element {
    Literal(char),
    Alternate(Vec<Vec<Element>>),
}

struct Parser<'a> {
    input: &'a str,
}

impl<'a> Parser<'a> {
    fn tail(&self) -> Result<&'a str, Error> {
        self.input
            .chars()
            .nth(0)
            .and_then(|c| self.input.get(c.len_utf8()..))
            .ok_or_else(|| format_err!("cannot skip char"))
    }

    fn accept(&mut self, c: char) -> Result<bool, Error> {
        if self.input.starts_with(c) {
            self.input = self.tail()?;
            return Ok(true);
        }
        Ok(false)
    }

    fn expect(&mut self, c: char) -> Result<(), Error> {
        if self.accept(c)? {
            return Ok(());
        }
        Err(format_err!("expected {}, got {}", c, &self.input[..]))
    }

    fn alternate(&mut self) -> Result<Option<Element>, Error> {
        if self.accept('(')? {
            let mut alternatives = Vec::new();
            loop {
                alternatives.push(self.elements()?);
                if !self.accept('|')? {
                    break;
                }
            }
            self.expect(')')?;
            return Ok(Some(Element::Alternate(alternatives)));
        }

        Ok(None)
    }

    fn direction(&mut self) -> Result<Option<Element>, Error> {
        for &c in &['N', 'S', 'E', 'W'] {
            if self.accept(c)? {
                return Ok(Some(Element::Literal(c)));
            }
        }
        Ok(None)
    }

    fn elements(&mut self) -> Result<Vec<Element>, Error> {
        let mut result = Vec::new();

        loop {
            if let Some(e) = self.direction()? {
                result.push(e);
            } else if let Some(e) = self.alternate()? {
                result.push(e);
            } else {
                break;
            }
        }

        Ok(result)
    }

    fn parse(&mut self) -> Result<Vec<Element>, Error> {
        self.expect('^')?;
        let root = self.elements()?;
        self.expect('$')?;

        Ok(root)
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct Room {
    north: bool,
    south: bool,
    east: bool,
    west: bool,
}

/// This is pretty wildly inefficient, but we only draw once at the end...
fn draw(map: &HashMap<(i32, i32), Room>) {
    if map.is_empty() {
        return;
    }
    let x_min = map.keys().map(|&(x, _)| x).min().unwrap();
    let x_max = map.keys().map(|&(x, _)| x).max().unwrap();
    let cols = (2 * (x_max - x_min + 1) + 1) as usize;

    let y_min = map.keys().map(|&(_, y)| y).min().unwrap();
    let y_max = map.keys().map(|&(_, y)| y).max().unwrap();
    let rows = (2 * (y_max - y_min + 1) + 1) as usize;

    let mut grid = vec![vec![' '; cols]; rows];

    for (&(x, y), room) in map.iter() {
        let start = (x == 0) && (y == 0);

        // Transform coordinates from arbitrary space to grid space.
        let x = (2 * (x - x_min) + 1) as usize;
        let y = (2 * (y - y_min) + 1) as usize;

        // Draw the room.
        grid[y][x] = if start { 'X' } else { '.' };
        grid[y - 1][x - 1] = '#';
        grid[y + 1][x - 1] = '#';
        grid[y - 1][x + 1] = '#';
        grid[y + 1][x + 1] = '#';
        grid[y - 1][x] = if room.north { '-' } else { '#' };
        grid[y + 1][x] = if room.south { '-' } else { '#' };
        grid[y][x + 1] = if room.east { '|' } else { '#' };
        grid[y][x - 1] = if room.west { '|' } else { '#' };
    }

    print!("\n");
    for row in grid {
        println!("{}", row.into_iter().collect::<String>());
    }
    print!("\n");
}

fn walk_paths(
    mut x: i32,
    mut y: i32,
    mut regex: &[Element],
    mut map: &mut HashMap<(i32, i32), Room>,
) {
    while let Some(element) = regex.first() {
        match element {
            Element::Literal('N') => {
                let (dx, dy) = (0, -1);
                map.entry((x, y)).or_default().north = true;
                map.entry((x + dx, y + dy)).or_default().south = true;
                x += dx;
                y += dy;
            }
            Element::Literal('S') => {
                let (dx, dy) = (0, 1);
                map.entry((x, y)).or_default().south = true;
                map.entry((x + dx, y + dy)).or_default().north = true;
                x += dx;
                y += dy;
            }
            Element::Literal('E') => {
                let (dx, dy) = (1, 0);
                map.entry((x, y)).or_default().east = true;
                map.entry((x + dx, y + dy)).or_default().west = true;
                x += dx;
                y += dy;
            }
            Element::Literal('W') => {
                let (dx, dy) = (-1, 0);
                map.entry((x, y)).or_default().west = true;
                map.entry((x + dx, y + dy)).or_default().east = true;
                x += dx;
                y += dy;
            }
            Element::Alternate(alternatives) => {
                for alternative in alternatives {
                    walk_paths(x, y, &alternative[..], &mut map);
                }
            }
            _ => panic!("unimplemented: {:?}", element),
        }

        regex = regex.get(1..).unwrap();
    }
}

fn answers(map: &HashMap<(i32, i32), Room>) -> (usize, usize) {
    let mut frontier = VecDeque::new();
    frontier.push_back((0, 0));

    let mut distances = HashMap::new();
    distances.insert((0, 0), 0);

    while let Some(current) = frontier.pop_front() {
        let distance = distances[&current] + 1;
        let room = map[&current];

        if room.north {
            let north = (current.0, current.1 - 1);
            distances.entry(north).or_insert_with(|| {
                frontier.push_back(north);
                distance
            });
        }
        if room.south {
            let south = (current.0, current.1 + 1);
            distances.entry(south).or_insert_with(|| {
                frontier.push_back(south);
                distance
            });
        }
        if room.east {
            let east = (current.0 + 1, current.1);
            distances.entry(east).or_insert_with(|| {
                frontier.push_back(east);
                distance
            });
        }
        if room.west {
            let west = (current.0 - 1, current.1);
            distances.entry(west).or_insert_with(|| {
                frontier.push_back(west);
                distance
            });
            frontier.push_back(west);
        }
    }

    let first = *distances.values().max().unwrap();
    let second = distances.values().filter(|&&v| v >= 1000).count();
    (first, second)
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let mut file = File::open(&args[1])?;
    let mut input = String::new();
    file.read_to_string(&mut input)?;

    let mut parser = Parser { input: &input };
    let regex = parser.parse()?;

    let mut map: HashMap<(i32, i32), Room> = HashMap::new();
    walk_paths(0, 0, &regex, &mut map);

    let (first, second) = answers(&map);
    println!("Furthest room requires passing {} doors", first);
    println!("{} rooms require passing through 1000+ doors", second);
    draw(&map);

    Ok(())
}
