extern crate failure;

use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use failure::Error;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1])?;

    let changes: Vec<i32> = BufReader::new(file)
        .lines()
        .map(|l| l.expect("file read failed"))
        .map(|l| l.parse::<i32>().expect("not-a-number"))
        .collect();

    println!("{}", changes.iter().sum::<i32>());

    let mut seen = HashSet::new();
    seen.insert(0i32);

    let mut frequency = 0;
    for change in changes.iter().cycle() {
        frequency += change;
        if seen.contains(&frequency) {
            println!("first duplicate: {}", frequency);
            break;
        }
        seen.insert(frequency);
    }

    Ok(())
}
