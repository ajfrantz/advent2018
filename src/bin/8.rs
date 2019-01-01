use std::env;
use std::fs::File;
use std::io::prelude::*;

use failure::{ensure, Error, Fail};

#[derive(Debug, Clone)]
struct Node {
    children: Vec<Node>,
    metadata: Vec<usize>,
}

#[derive(Fail, Debug)]
#[fail(display = "Ran out of input parsing node")]
struct EofError;

fn parse_node(input: &[usize]) -> Result<(Node, &[usize]), Error> {
    let (&n_children, input) = input.split_first().ok_or(EofError)?;
    let (&n_metadata, mut input) = input.split_first().ok_or(EofError)?;

    let mut children = Vec::new();
    for _ in 0..n_children {
        let (child, remain) = parse_node(input)?;
        children.push(child);
        input = remain;
    }

    let mut metadata = Vec::new();
    for _ in 0..n_metadata {
        let (&datum, remain) = input.split_first().ok_or(EofError)?;
        metadata.push(datum);
        input = remain;
    }

    Ok((Node { children, metadata }, input))
}

fn checksum(node: &Node) -> usize {
    node.metadata.iter().sum::<usize>() + node.children.iter().map(|n| checksum(&n)).sum::<usize>()
}

fn value(node: &Node) -> usize {
    // If a node has no child nodes, its value is the sum of its metadata entries.
    if node.children.is_empty() {
        return node.metadata.iter().sum::<usize>();
    }

    // Otherwise, the value is the sum of the referenced child nodes' values.
    node.metadata
        .iter()
        .filter(|&&datum| datum != 0)
        .map(|datum| node.children.get(datum - 1).map_or(0, |child| value(child)))
        .sum::<usize>()
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let mut file = File::open(&args[1])?;

    let mut input = String::new();
    file.read_to_string(&mut input)?;
    let input: Vec<usize> = input
        .trim()
        .split(' ')
        .map(|s| s.parse().expect("non-integer input"))
        .collect();

    let (root, leftover) = parse_node(&input)?;
    ensure!(leftover.is_empty(), "had leftover data at root");
    println!("first answer: {}", checksum(&root));
    println!("second answer: {}", value(&root));

    Ok(())
}
