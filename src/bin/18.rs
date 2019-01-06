use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use failure::Error;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Acre {
    Open,
    Trees,
    Lumberyard,
}

fn adjacent(acre: Acre, (x, y): (usize, usize), landscape: &Vec<Vec<Acre>>) -> usize {
    let neighbors = [
        (x - 1, y - 1),
        (x, y - 1),
        (x + 1, y - 1),
        (x - 1, y),
        (x + 1, y),
        (x - 1, y + 1),
        (x, y + 1),
        (x + 1, y + 1),
    ];
    neighbors
        .iter()
        .filter(|&&(x, y)| landscape.get(y).and_then(|r| r.get(x)) == Some(&acre))
        .count()
}

fn tick(landscape: &Vec<Vec<Acre>>) -> Vec<Vec<Acre>> {
    let mut next = landscape.clone();
    for (y, row) in landscape.iter().enumerate() {
        for (x, acre) in row.iter().enumerate() {
            next[y][x] = match acre {
                Acre::Open => {
                    if adjacent(Acre::Trees, (x, y), landscape) >= 3 {
                        Acre::Trees
                    } else {
                        Acre::Open
                    }
                }
                Acre::Trees => {
                    if adjacent(Acre::Lumberyard, (x, y), landscape) >= 3 {
                        Acre::Lumberyard
                    } else {
                        Acre::Trees
                    }
                }
                Acre::Lumberyard => {
                    if adjacent(Acre::Lumberyard, (x, y), landscape) >= 1
                        && adjacent(Acre::Trees, (x, y), landscape) >= 1
                    {
                        Acre::Lumberyard
                    } else {
                        Acre::Open
                    }
                }
            }
        }
    }
    next
}

fn print_score(landscape: &Vec<Vec<Acre>>) {
    let wooded = landscape
        .iter()
        .flat_map(|r| r.iter())
        .filter(|&&a| a == Acre::Trees)
        .count();
    let lumberyards = landscape
        .iter()
        .flat_map(|r| r.iter())
        .filter(|&&a| a == Acre::Lumberyard)
        .count();
    println!("{} * {} = {}", wooded, lumberyards, wooded * lumberyards);
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1])?;

    let mut landscape: Vec<Vec<Acre>> = Vec::new();
    for line in BufReader::new(file).lines().map(|l| l.unwrap()) {
        landscape.push(
            line.chars()
                .map(|c| match c {
                    '.' => Acre::Open,
                    '|' => Acre::Trees,
                    '#' => Acre::Lumberyard,
                    _ => panic!("bad input: {}", c),
                })
                .collect(),
        );
    }

    let mut cache = HashMap::new();

    for elapsed in 1..=10 {
        landscape = tick(&landscape);
        cache.insert(landscape.clone(), elapsed);
    }
    print_score(&landscape);

    for elapsed in 11..=1000000000 {
        landscape = tick(&landscape);

        if let Some(previous) = cache.get(&landscape) {
            println!("Found repeated pattern after {} minutes.", elapsed);
            println!("Previously seen after {} minutes.", previous);
            let remaining_minutes = 1000000000 - elapsed;
            let period = elapsed - previous;
            let cycles = remaining_minutes / period;
            println!("Repetition ({} min): skip {} cycles.", period, cycles);
            let completion = remaining_minutes % period;
            println!("{} steps remain to find the end.", completion);
            for _ in 0..completion {
                landscape = tick(&landscape);
            }
            break;
        }

        cache.insert(landscape.clone(), elapsed);
    }
    print_score(&landscape);

    Ok(())
}
