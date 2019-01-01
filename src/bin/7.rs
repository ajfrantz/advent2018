use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use failure::Error;
use regex::Regex;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Name(char);

// Explicit (Partial)Ord impl so BinaryHeap is a min-heap.
impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Name) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Name {
    fn cmp(&self, other: &Name) -> Ordering {
        other.0.cmp(&self.0)
    }
}

impl Name {
    fn work(&self) -> i32 {
        61 + (self.0 as i32 - 'A' as i32)
    }
}

#[derive(Debug, Clone)]
struct Step {
    pending: usize,
    feeds: Vec<Name>,
}

fn get_step(steps: &mut HashMap<Name, Step>, name: Name) -> &mut Step {
    steps.entry(name).or_insert(Step {
        pending: 0,
        feeds: Vec::new(),
    })
}

fn build_ready_list(steps: &HashMap<Name, Step>) -> BinaryHeap<Name> {
    let mut ready = BinaryHeap::new();
    for (&name, info) in steps.iter() {
        if info.pending == 0 {
            ready.push(name);
        }
    }
    ready
}

fn first_answer(mut steps: HashMap<Name, Step>) {
    let mut ready = build_ready_list(&steps);

    // Run until we're out of steps.  Note that this assumes the puzzle input is reasonable.
    let mut answer = String::new();
    while let Some(step) = ready.pop() {
        answer.push(step.0);
        let unblocks = steps.get(&step).unwrap().feeds.clone();
        for &name in unblocks.iter() {
            let pending = &mut get_step(&mut steps, name).pending;
            *pending -= 1;
            if *pending == 0 {
                ready.push(name);
            }
        }
    }

    println!("first answer: {}", answer);
}

fn second_answer(mut steps: HashMap<Name, Step>) {
    let mut ready = build_ready_list(&steps);
    let mut work = Vec::new();

    let mut time = 0;
    while !ready.is_empty() || !work.is_empty() {
        // Assign work to idle workers.
        while !ready.is_empty() && work.len() < 5 {
            let task = ready.pop().unwrap();
            work.push((time + task.work(), task));
        }

        // Step until the next completed work.
        work.sort_unstable_by_key(|(t, _)| -t);
        if let Some((t_complete, _)) = work.last() {
            time = *t_complete;
        }

        // Figure out what's finished now.
        while let Some((t_complete, name)) = work.last() {
            if time >= *t_complete {
                let unblocks = steps.get(name).unwrap().feeds.clone();
                for &name in unblocks.iter() {
                    let pending = &mut get_step(&mut steps, name).pending;
                    *pending -= 1;
                    if *pending == 0 {
                        ready.push(name);
                    }
                }
                work.pop();
            } else {
                // Not being able to combine while-let and a boolean check is lame.
                break;
            }
        }
    }

    println!("second answer: {}", time);
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1])?;

    let re = Regex::new(r"Step ([A-Z]) must be finished before step ([A-Z]) can begin.")?;

    // Build the dependency graph.
    let mut steps = HashMap::new();
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let captures = re.captures(&line).unwrap();
        let input = Name(captures[1].chars().next().unwrap());
        let output = Name(captures[2].chars().next().unwrap());

        get_step(&mut steps, input).feeds.push(output);
        get_step(&mut steps, output).pending += 1;
    }

    first_answer(steps.clone());
    second_answer(steps);

    Ok(())
}
