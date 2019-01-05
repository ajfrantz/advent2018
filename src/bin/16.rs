use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;

use failure::{format_err, Error};
use regex::Regex;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Opcode {
    Addr,
    Addi,
    Mulr,
    Muli,
    Banr,
    Bani,
    Borr,
    Bori,
    Setr,
    Seti,
    Gtir,
    Gtri,
    Gtrr,
    Eqir,
    Eqri,
    Eqrr,
}

#[derive(Debug, Copy, Clone)]
struct Machine {
    registers: [i32; 4],
}

impl Machine {
    fn reg(&mut self, n: i32) -> &mut i32 {
        &mut self.registers[n as usize]
    }

    fn execute(&mut self, opcode: Opcode, a: i32, b: i32, c: i32) {
        match opcode {
            Opcode::Addr => *self.reg(c) = *self.reg(a) + *self.reg(b),
            Opcode::Addi => *self.reg(c) = *self.reg(a) + b,
            Opcode::Mulr => *self.reg(c) = *self.reg(a) * *self.reg(b),
            Opcode::Muli => *self.reg(c) = *self.reg(a) * b,
            Opcode::Banr => *self.reg(c) = *self.reg(a) & *self.reg(b),
            Opcode::Bani => *self.reg(c) = *self.reg(a) & b,
            Opcode::Borr => *self.reg(c) = *self.reg(a) | *self.reg(b),
            Opcode::Bori => *self.reg(c) = *self.reg(a) | b,
            Opcode::Setr => *self.reg(c) = *self.reg(a),
            Opcode::Seti => *self.reg(c) = a,
            Opcode::Gtir => *self.reg(c) = if a > *self.reg(b) { 1 } else { 0 },
            Opcode::Gtri => *self.reg(c) = if *self.reg(a) > b { 1 } else { 0 },
            Opcode::Gtrr => *self.reg(c) = if *self.reg(a) > *self.reg(b) { 1 } else { 0 },
            Opcode::Eqir => *self.reg(c) = if a == *self.reg(b) { 1 } else { 0 },
            Opcode::Eqri => *self.reg(c) = if *self.reg(a) == b { 1 } else { 0 },
            Opcode::Eqrr => *self.reg(c) = if *self.reg(a) == *self.reg(b) { 1 } else { 0 },
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Example {
    before: [i32; 4],
    instruction: [i32; 4],
    after: [i32; 4],
}

impl Example {
    fn acts_as(&self, opcode: Opcode) -> bool {
        let mut machine = Machine {
            registers: self.before,
        };
        machine.execute(
            opcode,
            self.instruction[1],
            self.instruction[2],
            self.instruction[3],
        );
        machine.registers == self.after
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let mut file = File::open(&args[1])?;
    let mut input = String::new();
    file.read_to_string(&mut input)?;
    input = input.trim().to_string();

    let parts: Vec<&str> = input.split("\n\n\n\n").collect();
    let examples: Vec<&str> = parts[0].split("\n\n").collect();
    let test_program: Vec<&str> = parts[1].split("\n").collect();

    let re = Regex::new(
        &"Before: \\[(\\d), (\\d), (\\d), (\\d)\\]\n\
          (\\d+) (\\d+) (\\d+) (\\d+)\n\
          After:  \\[(\\d), (\\d), (\\d), (\\d)\\]",
    )?;
    let examples: Vec<Example> = examples
        .iter()
        .map(|s| {
            let captures = re.captures(&s).ok_or(format_err!("bad input: {}", s))?;
            Ok(Example {
                before: [
                    captures[1].parse()?,
                    captures[2].parse()?,
                    captures[3].parse()?,
                    captures[4].parse()?,
                ],
                instruction: [
                    captures[5].parse()?,
                    captures[6].parse()?,
                    captures[7].parse()?,
                    captures[8].parse()?,
                ],
                after: [
                    captures[9].parse()?,
                    captures[10].parse()?,
                    captures[11].parse()?,
                    captures[12].parse()?,
                ],
            })
        })
        .map(|res: Result<Example, Error>| res.expect("failed to parse example"))
        .collect();

    let opcodes = vec![
        Opcode::Addr,
        Opcode::Addi,
        Opcode::Mulr,
        Opcode::Muli,
        Opcode::Banr,
        Opcode::Bani,
        Opcode::Borr,
        Opcode::Bori,
        Opcode::Setr,
        Opcode::Seti,
        Opcode::Gtir,
        Opcode::Gtri,
        Opcode::Gtrr,
        Opcode::Eqir,
        Opcode::Eqri,
        Opcode::Eqrr,
    ];

    let first_answer = examples
        .iter()
        .filter(|&example| {
            let mut opcodes = opcodes.clone();
            opcodes.retain(|&op| example.acts_as(op));
            opcodes.len() >= 3
        })
        .count();
    println!("first answer: {}", first_answer);

    // Start with a map where anything is possible...
    let mut opcode_map: HashMap<i32, Vec<Opcode>> = (0..16).map(|n| (n, opcodes.clone())).collect();

    // Eliminate anything that obviously doesn't match the observed behavior.
    for example in examples {
        opcode_map
            .entry(example.instruction[0])
            .and_modify(|ops| ops.retain(|&op| example.acts_as(op)));
    }

    // Then recursively eliminate anything that we've assigned to another value.
    let mut unsolved = 16;
    while unsolved > 0 {
        unsolved = 0;
        for n in 0..16 {
            let ops = &opcode_map[&n];
            if ops.len() > 1 {
                unsolved += 1;
                continue;
            }
            let solved = ops[0];

            for i in 0..16 {
                if i != n {
                    opcode_map
                        .entry(i)
                        .and_modify(|ops| ops.retain(|&op| op != solved));
                }
            }
        }
    }

    // Now execute the test program using the matching values.
    let mut machine = Machine {
        registers: [0, 0, 0, 0],
    };

    let re = Regex::new(&"(\\d+) (\\d+) (\\d+) (\\d+)")?;
    for instruction in test_program {
        let captures = re
            .captures(&instruction)
            .ok_or(format_err!("bad input: {}", instruction))?;
        let opcode = captures[1].parse()?;
        let a = captures[2].parse()?;
        let b = captures[3].parse()?;
        let c = captures[4].parse()?;

        let opcode = opcode_map[&opcode][0];
        machine.execute(opcode, a, b, c);
    }

    println!("second answer: {}", machine.registers[0]);

    Ok(())
}
