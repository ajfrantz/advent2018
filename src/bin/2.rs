extern crate failure;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use failure::Error;

fn matches(a: &str, b: &str) -> String {
    a.chars()
        .zip(b.chars())
        .filter(|(x, y)| x == y)
        .map(|(x, _)| x)
        .collect()
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1])?;

    let mut box_ids: Vec<String> = BufReader::new(file)
        .lines()
        .map(|l| l.expect("file read failed"))
        .collect();

    let mut twos = 0;
    let mut threes = 0;
    for id in box_ids.iter() {
        let mut letter_counts = HashMap::new();
        for letter in id.chars() {
            *letter_counts.entry(letter).or_insert(0) += 1;
        }
        if letter_counts.values().any(|&x| x == 2) {
            twos += 1
        };
        if letter_counts.values().any(|&x| x == 3) {
            threes += 1
        };
    }
    println!("checksum {}", twos * threes);

    box_ids.sort();
    for ids in box_ids.windows(2) {
        let id_len = ids[0].len();
        let matches = matches(&ids[0], &ids[1]);
        let distance = id_len - matches.len();
        if distance == 1 {
            println!("{}", matches);
        }
    }

    Ok(())
}
