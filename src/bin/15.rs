extern crate slotmap;

use std::collections::VecDeque;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

use failure::Error;
use slotmap::{new_key_type, SlotMap};

new_key_type! {
    struct UnitKey;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Team {
    Goblin,
    Elf,
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Team::Goblin => write!(f, "G")?,
            Team::Elf => write!(f, "E")?,
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Unit {
    team: Team,
    hp: i32,
    position: usize,
}

impl Unit {
    pub fn new(team: Team, position: usize) -> Unit {
        Unit {
            team,
            hp: 200,
            position,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Square {
    Wall,
    Open,
    Occupied(UnitKey),
}

#[derive(Debug, Clone)]
struct Game {
    width: usize,
    height: usize,
    map: Vec<Square>,
    units: SlotMap<UnitKey, Unit>,
    elf_power: i32,
    elf_losses: i32,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut units = Vec::new();
        for (i, square) in self.map.iter().enumerate() {
            match square {
                Square::Wall => write!(f, "#")?,
                Square::Open => write!(f, ".")?,
                Square::Occupied(unit) => {
                    let unit = self.units[*unit];
                    units.push((unit.team, unit.hp));
                    write!(f, "{}", unit.team)?;
                }
            }

            if ((i + 1) % self.width) == 0 {
                for unit in units.iter() {
                    write!(f, "  {}({})", unit.0, unit.1)?;
                }
                units.clear();
                write!(f, "\n")?;
            }
        }
        Ok(())
    }
}

impl Game {
    fn new(input: &str, elf_power: i32) -> Game {
        let mut map = Vec::new();
        let mut units = SlotMap::with_key();
        let mut width = None;

        for line in input.split("\n") {
            for c in line.chars() {
                map.push(match c {
                    '#' => Square::Wall,
                    '.' => Square::Open,
                    'G' => Square::Occupied(units.insert(Unit::new(Team::Goblin, map.len()))),
                    'E' => Square::Occupied(units.insert(Unit::new(Team::Elf, map.len()))),
                    _ => panic!("bad input: {}", c),
                });
            }

            if let Some(w) = width {
                assert!(map.len() % w == 0);
            } else {
                width = Some(map.len());
            }
        }

        let width = width.unwrap();
        Game {
            width,
            height: map.len() / width,
            map,
            units,
            elf_power,
            elf_losses: 0,
        }
    }

    /// Returns the keys for units in the order they should move.
    /// Note that by the time a unit's key comes up, it may already be dead.
    fn turn_order(&self) -> Vec<UnitKey> {
        let mut keys: Vec<UnitKey> = self.units.keys().collect();
        keys.sort_unstable_by_key(|&k| self.units[k].position);
        keys
    }

    /// Returns a list of targets for the given team, in "reading order."
    fn targets(&self, for_team: Team) -> Vec<usize> {
        let mut targets: Vec<usize> = self
            .units
            .values()
            .filter(|&u| u.team != for_team)
            .map(|u| u.position)
            .collect();
        targets.sort_unstable();
        targets
    }

    fn neighbors(&self, target: usize) -> Vec<usize> {
        let offsets = [-(self.width as i32), -1, 1, self.width as i32];
        offsets
            .into_iter()
            .map(|&offset| (target as i32 + offset) as usize)
            .filter(|&p| p < self.map.len())
            .collect()
    }

    /// Filters a target list to those hittable from the given location.
    /// Returns list in prioritized order (by lower health, reading order).
    fn hittable(&self, from: usize, targets: &Vec<usize>) -> Vec<usize> {
        let neighbors = self.neighbors(from);
        let mut targets: Vec<usize> = targets
            .iter()
            .map(|&p| p)
            .filter(|&target| neighbors.iter().find(|&&p| p == target) != None)
            .collect();

        // We require a stable sort here, since the original target list is given in reading order
        // and that is the tie breaking criterion.
        targets.sort_by_key(|&p| {
            if let Square::Occupied(key) = self.map[p] {
                self.units[key].hp
            } else {
                panic!("targeting missing unit");
            }
        });

        targets
    }

    fn open_neighbors(&self, target: usize) -> Vec<usize> {
        let mut neighbors = self.neighbors(target);
        neighbors.retain(|&p| self.map[p] == Square::Open);
        neighbors
    }

    /// Find shortest-path distance from start to all (reachable) cells.
    /// Unreachable cells get a costs of None.
    fn distances(&self, from: usize) -> Vec<Option<usize>> {
        let mut frontier = VecDeque::new();
        frontier.push_back(from);

        let mut distances = Vec::with_capacity(self.map.len());
        distances.resize(self.map.len(), None);
        distances[from] = Some(0);

        while let Some(current) = frontier.pop_front() {
            let distance = Some(distances[current].unwrap() + 1);

            for neighbor in self.open_neighbors(current) {
                if let None = distances[neighbor] {
                    frontier.push_back(neighbor);
                    distances[neighbor] = distance;
                }
            }
        }

        distances
    }

    /// Get the (reading-order-preferred) shortest-distance next step to the goal cell.
    /// If the goal is unreachable, returns None.
    fn next_step(&self, from: usize, to: usize) -> Option<usize> {
        let mut frontier = VecDeque::new();
        frontier.push_back(to);

        let mut costs = Vec::with_capacity(self.map.len());
        costs.resize(self.map.len(), None);
        costs[to] = Some(0);

        while let Some(current) = frontier.pop_front() {
            let distance = Some(costs[current].unwrap() + 1);

            for neighbor in self.open_neighbors(current) {
                if let None = costs[neighbor] {
                    frontier.push_back(neighbor);
                    costs[neighbor] = distance;
                }
            }
        }

        self.open_neighbors(from)
            .iter()
            .filter(|&&p| costs[p] != None)
            .min_by_key(|&&p| costs[p])
            .map(|&p| p)
    }

    fn try_attack(&mut self, attacker: &Unit, targets: &Vec<usize>) -> bool {
        if let Some(target) = self
            .hittable(attacker.position, targets)
            .first()
            .map(|&t| t)
        {
            if let Square::Occupied(unit) = self.map[target] {
                let attack_power = match attacker {
                    Unit {
                        team: Team::Elf, ..
                    } => self.elf_power,
                    _ => 3,
                };
                self.units[unit].hp -= attack_power;
                if self.units[unit].hp <= 0 {
                    self.units.remove(unit);
                    self.map[target] = Square::Open;
                    if attacker.team == Team::Goblin {
                        self.elf_losses += 1;
                    }
                }
                return true;
            } else {
                panic!("attacking empty square");
            }
        }
        false
    }

    /// Tries to move from @p from to attack a target in @p targets.
    /// Returns Some(new position) if successful, else None.
    fn try_move(&mut self, from: usize, targets: &Vec<usize>) -> bool {
        let mut adjacents: Vec<usize> = targets
            .iter()
            .flat_map(|&target| self.open_neighbors(target))
            .collect();
        adjacents.sort_unstable();
        adjacents.dedup();

        let distances = self.distances(from);
        if let Some((dest, _)) = adjacents
            .iter()
            .filter_map(|&p| match distances[p] {
                Some(d) => Some((p, d)),
                _ => None,
            })
            .min_by_key(|&(_, distance)| distance)
        {
            let step = self.next_step(from, dest).unwrap();
            self.move_unit(from, step);
            return true;
        }

        false
    }

    fn move_unit(&mut self, from: usize, to: usize) {
        if let Square::Occupied(unit) = self.map[from] {
            self.map[to] = self.map[from];
            self.units[unit].position = to;

            self.map[from] = Square::Open;
        }
    }

    fn do_round(&mut self) -> bool {
        for unit_key in self.turn_order().iter() {
            if let Some(&unit) = self.units.get(*unit_key) {
                let targets = self.targets(unit.team);
                if targets.is_empty() {
                    return true;
                }

                if self.try_attack(&unit, &targets) {
                    continue;
                }

                if self.try_move(unit.position, &targets) {
                    let unit = self.units[*unit_key];
                    self.try_attack(&unit, &targets);
                }
            }
        }

        false
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let mut file = File::open(&args[1])?;
    let mut input = String::new();
    file.read_to_string(&mut input)?;

    for elf_power in 3.. {
        let mut game = Game::new(&input, elf_power);

        let mut round = 1;
        while !game.do_round() && game.elf_losses == 0 {
            round += 1;
        }

        if game.elf_losses == 0 {
            println!("Elves needed {} attack power.", elf_power);

            let last_full = round - 1;
            let hit_points: i32 = game.units.values().map(|&u| u.hp).sum();
            println!(
                "Outcome is {} * {} = {}",
                last_full,
                hit_points,
                last_full * hit_points
            );
            break;
        }
    }

    Ok(())
}
