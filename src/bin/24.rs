#[macro_use]
extern crate lazy_static;

use std::env;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

use failure::{format_err, Error};
use regex::Regex;
use slotmap::{new_key_type, SlotMap};

new_key_type! {
    struct ArmyKey;
    struct GroupKey;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Army {
    ImmuneSystem,
    Infection,
}

#[derive(Debug, Copy, Clone)]
enum AttackType {
    Fire,
    Cold,
    Slashing,
    Bludgeoning,
    Radiation,
}

impl FromStr for AttackType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "fire" => AttackType::Fire,
            "cold" => AttackType::Cold,
            "slashing" => AttackType::Slashing,
            "bludgeoning" => AttackType::Bludgeoning,
            "radiation" => AttackType::Radiation,
            _ => return Err(format_err!("bad attack type: {}", s)),
        })
    }
}

#[derive(Debug, Copy, Clone)]
struct Defenses {
    fire: usize,
    cold: usize,
    slashing: usize,
    bludgeoning: usize,
    radiation: usize,
}

impl Default for Defenses {
    fn default() -> Defenses {
        Defenses {
            fire: 1,
            cold: 1,
            slashing: 1,
            bludgeoning: 1,
            radiation: 1,
        }
    }
}

impl Index<AttackType> for Defenses {
    type Output = usize;

    fn index(&self, attack_type: AttackType) -> &usize {
        match attack_type {
            AttackType::Fire => &self.fire,
            AttackType::Cold => &self.cold,
            AttackType::Slashing => &self.slashing,
            AttackType::Bludgeoning => &self.bludgeoning,
            AttackType::Radiation => &self.radiation,
        }
    }
}

impl IndexMut<AttackType> for Defenses {
    fn index_mut(&mut self, attack_type: AttackType) -> &mut usize {
        match attack_type {
            AttackType::Fire => &mut self.fire,
            AttackType::Cold => &mut self.cold,
            AttackType::Slashing => &mut self.slashing,
            AttackType::Bludgeoning => &mut self.bludgeoning,
            AttackType::Radiation => &mut self.radiation,
        }
    }
}

impl Defenses {
    fn with_weaknesses(mut self, types: &Vec<AttackType>) -> Defenses {
        for &weakness in types.iter() {
            self[weakness] = 2;
        }
        self
    }

    fn with_immunities(mut self, types: &Vec<AttackType>) -> Defenses {
        for &immunity in types.iter() {
            self[immunity] = 0;
        }
        self
    }
}

impl FromStr for Defenses {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?:(?:(?:weak to (?P<weak>[^;]+))|(?:immune to (?P<immune>[^;]+)))(?:; )?){0,2}"
            )
            .unwrap();
        }

        let mut defenses: Defenses = Default::default();

        let captures = RE
            .captures(&s)
            .ok_or_else(|| format_err!("defenses match failed: {}", s))?;

        if let Some(weaknesses) = captures.name("weak") {
            let w = weaknesses
                .as_str()
                .split(", ")
                .map(|s| s.parse())
                .collect::<Result<Vec<AttackType>, _>>()?;
            defenses = defenses.with_weaknesses(&w);
        }

        if let Some(immunities) = captures.name("immune") {
            let i = immunities
                .as_str()
                .split(", ")
                .map(|s| s.parse())
                .collect::<Result<Vec<AttackType>, _>>()?;
            defenses = defenses.with_immunities(&i);
        }

        Ok(defenses)
    }
}

#[derive(Debug, Copy, Clone)]
struct Group {
    army: ArmyKey,
    units: usize,
    hit_points: usize,
    attack_damage: usize,
    attack_type: AttackType,
    initiative: usize,
    defenses: Defenses,
}

impl Group {
    fn from_str(s: &str, army: ArmyKey) -> Result<Group, Error> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(\d+) units each with (\d+) hit points (?:\(([^\)]+)\) )?with an attack that does (\d+) (\w+) damage at initiative (\d+)").unwrap();
        }

        let captures = RE
            .captures(&s)
            .ok_or_else(|| format_err!("group match failed: {}", s))?;

        Ok(Group {
            army,
            units: captures[1].parse()?,
            hit_points: captures[2].parse()?,
            attack_damage: captures[4].parse()?,
            attack_type: captures[5].parse()?,
            initiative: captures[6].parse()?,
            defenses: captures
                .get(3)
                .map(|s| s.as_str().parse())
                .unwrap_or_else(|| Ok(Default::default()))?,
        })
    }

    fn effective_power(&self) -> usize {
        self.units * self.attack_damage
    }

    fn damage_against(&self, target: &Group) -> usize {
        let modifier = target.defenses[self.attack_type];
        self.effective_power() * modifier
    }

    fn pick_target(&self, groups: &SlotMap<GroupKey, Group>) -> Option<GroupKey> {
        let mut candidates: Vec<GroupKey> = groups
            .keys()
            .filter(|&k| groups[k].army != self.army)
            .collect();

        if let Some(most_damage) = candidates
            .iter()
            .map(|&k| {
                let damage = self.damage_against(&groups[k]);
                damage
            })
            .max()
        {
            // If it cannot deal any defending groups damage, it does not choose a target.
            if most_damage == 0 {
                return None;
            }
            candidates.retain(|&k| self.damage_against(&groups[k]) == most_damage);
        }

        if let Some(largest_power) = candidates
            .iter()
            .map(|&k| groups[k].effective_power())
            .max()
        {
            candidates.retain(|&k| groups[k].effective_power() == largest_power);
        }

        candidates.sort_unstable_by_key(|&k| groups[k].initiative as i64 * -1);
        candidates.first().map(|&k| k)
    }

    fn take_damage(&mut self, attacker: &Group) {
        let damage = attacker.damage_against(self);
        let units_lost = std::cmp::min(self.units, damage / self.hit_points);
        self.units -= units_lost;
    }
}

#[derive(Debug, Clone)]
struct Game {
    armies: SlotMap<ArmyKey, Army>,
    groups: SlotMap<GroupKey, Group>,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Immune System:")?;
        for (k, group) in self
            .groups
            .iter()
            .filter(|(_, g)| self.armies[g.army] == Army::ImmuneSystem)
        {
            writeln!(f, "Group {:?} contains {} units", k, group.units)?;
        }

        writeln!(f, "Infection:")?;
        for (k, group) in self
            .groups
            .iter()
            .filter(|(_, g)| self.armies[g.army] == Army::Infection)
        {
            writeln!(f, "Group {:?} contains {} units", k, group.units)?;
        }

        Ok(())
    }
}

impl FromStr for Game {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut armies = SlotMap::with_key();
        let immune_system = armies.insert(Army::ImmuneSystem);
        let infection = armies.insert(Army::Infection);

        let mut groups = SlotMap::with_key();

        let mut input = s.split("\n\n");
        let mut lines = input
            .next()
            .ok_or_else(|| format_err!("missing immune team"))?
            .split("\n");
        lines.next(); // skip name
        for line in lines {
            groups.insert(Group::from_str(line, immune_system)?);
        }

        let mut lines = input
            .next()
            .ok_or_else(|| format_err!("missing infection team"))?
            .split("\n");
        lines.next(); // skip name
        for line in lines {
            groups.insert(Group::from_str(line, infection)?);
        }

        Ok(Game { armies, groups })
    }
}

struct Attack {
    attacker: GroupKey,
    defender: GroupKey,
}

impl Game {
    fn select_targets(&self) -> Vec<Attack> {
        let mut attackers: Vec<GroupKey> = self.groups.keys().collect();
        attackers.sort_unstable_by_key(|&k| {
            (
                self.groups[k].effective_power() as i64 * -1, // decreasing effective power
                self.groups[k].initiative as i64 * -1,        // tie break decreasing initiative
            )
        });

        let mut attacks = Vec::with_capacity(self.groups.len());
        let mut targets = self.groups.clone();
        for attacker in attackers {
            let group = self.groups[attacker];
            if let Some(defender) = group.pick_target(&targets) {
                attacks.push(Attack { attacker, defender });
                targets.remove(defender);
            }
        }

        attacks
    }

    fn resolve_attacks(&mut self, mut attacks: Vec<Attack>) -> usize {
        let mut killed = 0;

        attacks.sort_unstable_by_key(|a| self.groups[a.attacker].initiative as i64 * -1);
        for attack in attacks {
            let attacker = self.groups[attack.attacker];

            // Require the group to still be alive to allow the attack.
            if attacker.units == 0 {
                continue;
            }

            let defender = self.groups.get_mut(attack.defender).unwrap();
            let before = defender.units;
            defender.take_damage(&attacker);
            let after = defender.units;
            killed += before - after;
            println!(
                "{:?} attacks {:?}, killing {} units",
                attack.attacker,
                attack.defender,
                before - after
            );
        }

        killed
    }

    fn fight(&mut self) -> bool {
        let attacks = self.select_targets();
        let killed = self.resolve_attacks(attacks);
        self.groups.retain(|_, group| group.units > 0);
        killed != 0
    }

    fn is_resolved(&self) -> bool {
        for army in self.armies.keys() {
            if self.groups.values().filter(|g| g.army == army).count() == 0 {
                return true;
            }
        }
        false
    }

    fn answer(&self) -> usize {
        self.groups.values().map(|g| g.units).sum()
    }

    fn reindeer_lives(&self) -> bool {
        let key = self.groups.keys().next().unwrap();
        self.armies[self.groups[key].army] == Army::ImmuneSystem
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let mut file = File::open(&args[1])?;
    let mut input = String::new();
    file.read_to_string(&mut input)?;
    input = input.trim().to_string();

    let game: Game = input.parse()?;

    'outer: for boost in 1.. {
        let mut boosted = game.clone();
        for group in boosted.groups.values_mut() {
            if boosted.armies[group.army] == Army::ImmuneSystem {
                group.attack_damage += boost;
            }
        }

        while !boosted.is_resolved() {
            println!("\n\n{}", boosted);
            if !boosted.fight() {
                continue 'outer;
            }
        }

        println!("\n\n{}", boosted);

        if boosted.reindeer_lives() {
            println!("answer: {}", boosted.answer());
            break;
        }
    }

    Ok(())
}
