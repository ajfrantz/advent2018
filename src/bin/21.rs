use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

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

impl FromStr for Opcode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "addr" => Ok(Opcode::Addr),
            "addi" => Ok(Opcode::Addi),
            "mulr" => Ok(Opcode::Mulr),
            "muli" => Ok(Opcode::Muli),
            "banr" => Ok(Opcode::Banr),
            "bani" => Ok(Opcode::Bani),
            "borr" => Ok(Opcode::Borr),
            "bori" => Ok(Opcode::Bori),
            "setr" => Ok(Opcode::Setr),
            "seti" => Ok(Opcode::Seti),
            "gtir" => Ok(Opcode::Gtir),
            "gtri" => Ok(Opcode::Gtri),
            "gtrr" => Ok(Opcode::Gtrr),
            "eqir" => Ok(Opcode::Eqir),
            "eqri" => Ok(Opcode::Eqri),
            "eqrr" => Ok(Opcode::Eqrr),
            _ => Err(format_err!("unknown opcode {}", s)),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Instruction {
    opcode: Opcode,
    a: i32,
    b: i32,
    c: i32,
}

#[derive(Debug, Clone)]
struct Machine {
    registers: [i32; 6],
    pc: i32,
    instructions: Vec<Instruction>,
}

impl Machine {
    fn new() -> Machine {
        Machine {
            registers: [0, 0, 0, 0, 0, 0],
            pc: 0,
            instructions: Vec::new(),
        }
    }

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

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let file = File::open(&args[1])?;

    let directive = Regex::new(r"#ip (\d+)")?;
    let instruction = Regex::new(r"([a-z]+) (\d+) (\d+) (\d+)")?;

    let mut machine = Machine::new();

    for line in BufReader::new(file).lines().map(|l| l.unwrap()) {
        if let Some(captures) = directive.captures(&line) {
            machine.pc = captures[1].parse()?;
        } else if let Some(captures) = instruction.captures(&line) {
            machine.instructions.push(Instruction {
                opcode: captures[1].parse()?,
                a: captures[2].parse()?,
                b: captures[3].parse()?,
                c: captures[4].parse()?,
            });
        }
    }

    let mut first_check = true;
    let mut seen: HashSet<i32> = HashSet::new();

    loop {
        let ip = *machine.reg(machine.pc) as usize;
        if let Some(instruction) = machine.instructions.get(ip) {
            // This is a tight inner loop effectively doing a division.
            if ip == 17 {
                machine.registers[5] /= 256;
                *machine.reg(machine.pc) = 27;
                continue;
            }

            // This is the exit condition from the loop.
            if ip == 28 {
                let r4 = machine.registers[4];

                if first_check {
                    println!("fastest exit value: {}", r4);
                    first_check = false;
                }

                if !seen.insert(r4) {
                    println!("found repeating values, exiting.");
                    break;
                }

                let r0 = machine.registers[0];
                println!("exit test: r4 ({}) =?= r0 ({})", r4, r0);
            }

            machine.execute(
                instruction.opcode,
                instruction.a,
                instruction.b,
                instruction.c,
            );
            *machine.reg(machine.pc) += 1;
        } else {
            break;
        }
    }

    Ok(())
}
