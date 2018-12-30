extern crate failure;
extern crate regex;

use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use failure::{format_err, Error};
use regex::Regex;

struct Patch {
    id: usize,
    left: usize,
    top: usize,
    width: usize,
    height: usize,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Status {
    Unclaimed,
    Claimed(usize),
    Conflict,
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1])?;

    let re = Regex::new(r"#(\d+) @ (\d+),(\d+): (\d+)x(\d+)")?;
    let patches: Vec<Patch> = BufReader::new(file)
        .lines()
        .map(|l| l.expect("file read failed"))
        .map(|l| {
            let captures = re.captures(&l).ok_or(format_err!("bad command pattern"))?;
            Ok(Patch {
                id: captures[1].parse()?,
                left: captures[2].parse()?,
                top: captures[3].parse()?,
                width: captures[4].parse()?,
                height: captures[5].parse()?,
            })
        })
        .map(|res: Result<Patch, Error>| res.expect("failed to parse patch"))
        .collect();

    let mut fabric = vec![vec![Status::Unclaimed; 1000]; 1000];
    let mut conflicting = HashSet::new();
    for patch in patches.iter() {
        for x in patch.left..patch.left + patch.width {
            for y in patch.top..patch.top + patch.height {
                fabric[x][y] = match fabric[x][y] {
                    Status::Unclaimed => Status::Claimed(patch.id),
                    Status::Claimed(first) => {
                        conflicting.insert(first);
                        conflicting.insert(patch.id);
                        Status::Conflict
                    }
                    Status::Conflict => {
                        conflicting.insert(patch.id);
                        Status::Conflict
                    }
                }
            }
        }
    }

    let in_conflict: usize = fabric
        .iter()
        .map(|row| row.iter().filter(|&&s| s == Status::Conflict).count())
        .sum();
    println!("cells in conflict: {}", in_conflict);

    println!(
        "clean claims: {:?}",
        patches
            .iter()
            .map(|patch| patch.id)
            .filter(|id| !conflicting.contains(id))
            .collect::<Vec<usize>>()
    );

    Ok(())
}
