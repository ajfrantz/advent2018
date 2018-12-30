extern crate failure;
extern crate regex;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use failure::{bail, ensure, format_err, Error};
use regex::Regex;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Event {
    ShiftChange { new_guard: usize },
    FallsAsleep { minute: usize },
    WakesUp { minute: usize },
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Status {
    Awake(usize),
    Sleeping(usize, usize),
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1])?;

    let mut entries: Vec<String> = BufReader::new(file)
        .lines()
        .map(|l| l.expect("file read failed"))
        .collect();
    entries.sort();

    let date = r"(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})";
    let time = r"(?P<hour>\d{2}):(?P<minute>\d{2})";
    let shift_start = r"(Guard #(?P<id>\d+) begins shift)";
    let asleep = r"(?P<sleep>falls asleep)";
    let awake = r"(?P<wake>wakes up)";
    let full_pattern = format!(
        r"\[{} {}\] ({}|{}|{})",
        date, time, shift_start, asleep, awake
    );
    let event = Regex::new(&full_pattern)?;

    let events: Vec<Event> = entries
        .iter()
        .map(|entry| {
            let captures = event
                .captures(&entry)
                .ok_or(format_err!("event pattern failed: {}", entry))?;
            if let Some(id) = captures.name("id") {
                Ok(Event::ShiftChange {
                    new_guard: id.as_str().parse::<usize>()?,
                })
            } else {
                let hour = captures
                    .name("hour")
                    .ok_or(format_err!("no hour"))?
                    .as_str();
                ensure!(hour == "00", "event outside midnight hour");

                let minute = captures
                    .name("minute")
                    .ok_or(format_err!("no minute"))?
                    .as_str()
                    .parse::<usize>()?;

                if let Some(_) = captures.name("sleep") {
                    Ok(Event::FallsAsleep { minute })
                } else if let Some(_) = captures.name("wake") {
                    Ok(Event::WakesUp { minute })
                } else {
                    bail!("pattern match fail");
                }
            }
        })
        .map(|res: Result<Event, Error>| res.expect("failed to parse event"))
        .collect();

    let mut status = Status::Awake(0);
    let mut time_slept = HashMap::new();
    let mut minutes_slept = HashMap::new();
    for event in events {
        status = match (status, event) {
            (Status::Awake(_), Event::ShiftChange { new_guard }) => Status::Awake(new_guard),
            (Status::Awake(id), Event::FallsAsleep { minute }) => Status::Sleeping(id, minute),
            (Status::Sleeping(id, start), Event::WakesUp { minute }) => {
                ensure!(id != 0, "bad guard id");
                ensure!(minute > start, "inconsistent time");
                *time_slept.entry(id).or_insert(0) += minute - start;
                for m in start..minute {
                    *minutes_slept
                        .entry(id)
                        .or_insert(HashMap::new())
                        .entry(m)
                        .or_insert(0) += 1;
                }
                Status::Awake(id)
            }
            _ => bail!("bad transition (in state {:?}, got {:?})", status, event),
        }
    }

    let guard = time_slept
        .iter()
        .max_by_key(|(_id, &slept)| slept)
        .map(|(id, _slept)| id)
        .unwrap();
    println!("sleepingest guard: {}", guard);

    let minute = minutes_slept
        .get(guard)
        .unwrap()
        .iter()
        .max_by_key(|(_minute, slept)| *slept)
        .map(|(minute, _slept)| minute)
        .unwrap();
    println!("guard {}'s sleepingest minute: {}", guard, minute);
    println!("first answer: {}", guard * minute);

    let (guard, minute, occurrences) = minutes_slept
        .iter()
        .map(|(guard, minutes)| {
            let (minute, occurrences) = minutes.iter().max_by_key(|(_, &s)| s).unwrap();
            (guard, minute, occurrences)
        })
        .max_by_key(|(_, _, &o)| o)
        .unwrap();

    println!(
        "guard {}'s slept most ({} times) on minute: {}",
        guard, occurrences, minute
    );
    println!("second answer: {}", guard * minute);

    Ok(())
}
