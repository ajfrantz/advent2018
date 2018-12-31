extern crate failure;

use std::env;
use std::fs::File;
use std::io::prelude::*;

use failure::Error;
use rayon::prelude::*;

fn as_lower(c: char) -> char {
    let lower = c.to_lowercase().to_string();
    let chars: Vec<char> = lower.chars().collect();
    assert!(chars.len() == 1);
    chars[0]
}

fn reacts(a: char, b: char) -> bool {
    let a_lower = as_lower(a);
    let b_lower = as_lower(b);
    (a_lower == b_lower) && (a != b)
}

fn react_once(units: &Vec<char>) -> (bool, Vec<char>) {
    let mut annihilated: Vec<bool> = Vec::with_capacity(units.len());
    annihilated.resize(units.len(), false);

    let mut reacted = false;
    units
        .windows(2)
        .enumerate()
        .filter(|(_, pair)| reacts(pair[0], pair[1]))
        .for_each(|(idx, _)| {
            if !annihilated[idx] {
                reacted = true;
                annihilated[idx] = true;
                annihilated[idx + 1] = true;
            }
        });

    (
        reacted,
        units
            .iter()
            .enumerate()
            .filter(|&(idx, _)| !annihilated[idx])
            .map(|(_, &c)| c)
            .collect(),
    )
}

fn react_fully(input: &Vec<char>) -> Vec<char> {
    let mut units = input.clone();
    while let (true, result) = react_once(&units) {
        units = result;
    }
    units
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let mut file = File::open(&args[1])?;

    let mut polymer = String::new();
    file.read_to_string(&mut polymer)?;

    let units: Vec<char> = polymer.trim().chars().collect();
    println!("first answer: {}", react_fully(&units).len());

    let mut types: Vec<char> = units.iter().map(|&c| as_lower(c)).collect();
    types.sort();
    types.dedup();

    let (t, shortest) = types
        .par_iter()
        .map(|&t| {
            println!("trying without {}", t);
            let trial_units: Vec<char> = units
                .iter()
                .map(|&c| c)
                .filter(|&c| as_lower(c) != t)
                .collect();
            (t, react_fully(&trial_units).len())
        })
        .min_by_key(|(_, n)| *n)
        .unwrap();

    println!(
        "without {}, the shortest reaction was acheived ({})",
        t, shortest
    );

    Ok(())
}
