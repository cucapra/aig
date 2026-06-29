use std::collections::HashMap;
use std::io::{self, BufRead, Error};

mod ascii_parser;
mod binary_parser;

use crate::graph::{AigGraph, NodeId};
use ascii_parser::parse_ascii_aiger_into_graph;
use binary_parser::parse_binary_aiger_into_graph;

#[derive(Debug)]
pub struct AigerHeader {
    pub is_ascii: bool,
    pub m: usize,
    pub i: usize,
    pub l: usize,
    pub o: usize,
    pub a: usize,
}

pub fn run_parser_with_options(
    reader: &mut impl BufRead,
    pre_optimize: bool,
) -> io::Result<AigGraph> {
    let header: AigerHeader = verify_aiger_header(reader)?;

    let graph: AigGraph = if header.is_ascii {
        parse_ascii_aiger_into_graph(header, reader, pre_optimize)?
    } else {
        parse_binary_aiger_into_graph(header, reader, pre_optimize)?
    };

    Ok(graph)
}

pub fn verify_aiger_header(reader: &mut impl BufRead) -> Result<AigerHeader, Error> {
    let mut line: String = String::new();
    reader.read_line(&mut line)?;

    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() != 6 {
        panic!("Header must have format: aag/aig M I L O A");
    }

    let is_ascii: bool = match parts[0] {
        "aag" => true,
        "aig" => false,
        _ => panic!("Invalid tag, must be either 'aag' or 'aig'"),
    };

    let m: usize = parts[1].parse().unwrap();
    let i: usize = parts[2].parse().unwrap();
    let l: usize = parts[3].parse().unwrap();
    let o: usize = parts[4].parse().unwrap();
    let a: usize = parts[5].parse().unwrap();
    let expected_m: usize = i + l + a;

    if m < expected_m {
        panic!(
            "ASCII AIGER requires M >= I + L + A, Binary requires M = I + L + A, got M={} and I+L+A={}",
            m, expected_m
        )
    }

    if m != expected_m && !is_ascii {
        panic!(
            "Binary AIGER requires M = I + L + A, got M={} and I+L+A={}",
            m, expected_m
        );
    }

    Ok(AigerHeader {
        is_ascii,
        m,
        i,
        l,
        o,
        a,
    })
}

pub fn read_one_number_line(reader: &mut impl BufRead) -> Result<usize, Error> {
    let mut line: String = String::new();

    if (reader.read_line(&mut line)?) == 0 {
        panic!("no data read from number line")
    }

    let trimmed: usize = line.trim().parse().unwrap();

    Ok(trimmed)
}

/// A mapping from AIGER literal indices to our internal `NodeId`s.
#[derive(Default)]
struct Literals(HashMap<usize, NodeId>);

impl Literals {
    fn new() -> Self {
        let mut map = HashMap::new();
        map.insert(0, NodeId::FALSE);
        map.insert(1, NodeId::TRUE);
        Self(map)
    }

    /// Record that a given AIGER literal corresponds to a given fresh `NodeID`.
    fn add(&mut self, literal: usize, id: NodeId) {
        debug_assert!(!self.0.contains_key(&literal));
        self.0.insert(literal, id);
    }

    /// Get the `NodeID` corresponding to a given AIGER literal.
    ///
    /// Panic if the literal is not present.
    fn get(&self, literal: usize) -> NodeId {
        // First, try the literal directly.
        match self.0.get(&literal) {
            Some(&id) => id,
            // If that fails, it might be because the literal is inverted. Try
            // regularizing the literal and look again.
            None => {
                let regular_lit: usize = literal & !1;
                let is_inverted: bool = (literal & 1) == 1;
                match self.0.get(&regular_lit) {
                    Some(&regular_node) => {
                        if is_inverted {
                            regular_node.invert()
                        } else {
                            regular_node
                        }
                    }
                    None => panic!("Unknown aiger literal: {}", literal),
                }
            }
        }
    }
}
