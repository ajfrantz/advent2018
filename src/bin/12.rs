use std::collections::HashMap;

use failure::Error;

fn tick(input: &str) -> char {
    match input {
        "..#.." => '.',
        "..#.#" => '.',
        "#.#.." => '.',
        ".#..#" => '.',
        "#...." => '.',
        "....#" => '.',
        ".#.#." => '#',
        "#.###" => '.',
        "####." => '.',
        "....." => '.',
        ".#..." => '#',
        "#####" => '#',
        ".####" => '.',
        "#..#." => '#',
        "#...#" => '#',
        ".###." => '.',
        "###.#" => '#',
        "...##" => '#',
        "#.##." => '#',
        ".#.##" => '#',
        "##.#." => '#',
        "...#." => '.',
        "..###" => '#',
        "###.." => '#',
        "##..." => '.',
        "..##." => '.',
        ".##.#" => '.',
        "##.##" => '.',
        ".##.." => '.',
        "##..#" => '#',
        "#.#.#" => '.',
        "#..##" => '#',
        _ => panic!("got input {} but have no matching rule", input),
    }
}

fn render(state: &HashMap<i32, char>, left: i32, right: i32) {
    println!(
        "[{:02}] {}",
        left,
        (left..=right)
            .map(|p| state.get(&p).or(Some(&'.')).unwrap())
            .collect::<String>()
    );
}

fn step(state: &mut HashMap<i32, char>, left: &mut i32, right: &mut i32) {
    *left -= 5;
    *right += 5;

    let mut next_state = HashMap::new();
    for pos in *left..=*right {
        let pattern: String = (pos - 2..=pos + 2)
            .map(|p| state.get(&p).or(Some(&'.')).unwrap())
            .collect();
        next_state.insert(pos, tick(&pattern));
    }

    *state = next_state;
    while let Some(&'.') = state.get(&left).or(Some(&'.')) {
        *left += 1;
    }
    while let Some(&'.') = state.get(&right).or(Some(&'.')) {
        *right -= 1;
    }
}

fn main() -> Result<(), Error> {
    let initial_state = "##.#..#.#..#.####.#########.#...#.#.#......##.#.#...##.....#...#...#.##.#...##...#.####.##..#.#..#.";

    let mut state = HashMap::new();
    initial_state.chars().enumerate().for_each(|(pos, c)| {
        state.insert(pos as i32, c);
    });

    let mut left = 0i32;
    let mut right = initial_state.len() as i32;

    for gen in 0..20 {
        print!("{:03}: ", gen);
        render(&state, left, right);
        step(&mut state, &mut left, &mut right);
    }

    let first_answer: i32 = (left..=right)
        .filter_map(|p| {
            if let Some(&'#') = state.get(&p) {
                return Some(p);
            }
            None
        })
        .sum();
    println!("first answer: {}", first_answer);

    for gen in 20..112 {
        print!("{:03}: ", gen);
        render(&state, left, right);
        step(&mut state, &mut left, &mut right);
    }

    // At this point the pattern is fixed, but it slides right 1 pot/step.
    let offset = 50000000000i64 - 112;
    let second_answer: i64 = (left..=right)
        .filter_map(|p| {
            if let Some(&'#') = state.get(&p) {
                return Some(p as i64 + offset);
            }
            None
        })
        .sum();
    println!("second answer: {}", second_answer);

    Ok(())
}
