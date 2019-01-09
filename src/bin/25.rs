use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use failure::Error;

#[derive(Debug, Copy, Clone)]
struct Coordinate {
    w: i32,
    x: i32,
    y: i32,
    z: i32,
}

impl Coordinate {
    fn dist(&self, other: &Coordinate) -> i32 {
        let dw = (self.w - other.w).abs();
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        let dz = (self.z - other.z).abs();
        dw + dx + dy + dz
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1])?;

    let mut points: Vec<Coordinate> = Vec::new();

    for line in BufReader::new(file).lines().map(|l| l.unwrap()) {
        let coords = line
            .trim()
            .split(',')
            .map(|s| s.parse())
            .collect::<Result<Vec<i32>, _>>()?;

        points.push(Coordinate {
            w: coords[0],
            x: coords[1],
            y: coords[2],
            z: coords[3],
        });
    }

    let mut constellations: Vec<Vec<Coordinate>> = Vec::new();
    for point in points {
        let mut in_range: Vec<usize> = constellations
            .iter()
            .enumerate()
            .filter(|(_, members)| members.iter().any(|member| member.dist(&point) <= 3))
            .map(|(i, _)| i)
            .collect();

        if let Some(&to_join) = in_range.first() {
            // Join the first constellation (with the lowest index).
            constellations[to_join].push(point);

            // Merge any other constellations we could have joined.
            // We do this high index -> low index, so the indices remain valid.
            let mut merge_points = Vec::new();
            while in_range.len() > 1 {
                let to_merge = in_range.pop().unwrap();
                merge_points.append(&mut constellations[to_merge]);
                constellations.remove(to_merge);
            }
            constellations[to_join].append(&mut merge_points);
        } else {
            constellations.push(vec![point]);
        }
    }

    println!("{}", constellations.len());

    Ok(())
}
