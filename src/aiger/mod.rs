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
    pub max_var: usize,
    pub num_inputs: usize,
    pub num_latches: usize,
    pub num_outputs: usize,
    pub num_and_gates: usize,
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

    let max_var: usize = parts[1].parse().unwrap();
    let num_inputs: usize = parts[2].parse().unwrap();
    let num_latches: usize = parts[3].parse().unwrap();
    let num_outputs: usize = parts[4].parse().unwrap();
    let num_and_gates: usize = parts[5].parse().unwrap();
    let expected_max_var: usize = num_inputs + num_latches + num_and_gates;

    if max_var < expected_max_var {
        panic!(
            "ASCII AIGER requires M >= I + L + A, Binary requires M = I + L + A, got M={} and I+L+A={}",
            max_var, expected_max_var
        )
    }

    if max_var != expected_max_var && !is_ascii {
        panic!(
            "Binary AIGER requires M = I + L + A, got M={} and I+L+A={}",
            max_var, expected_max_var
        );
    }

    Ok(AigerHeader {
        is_ascii,
        max_var,
        num_inputs,
        num_latches,
        num_outputs,
        num_and_gates,
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
struct Literals(Vec<Option<NodeId>>);

impl Literals {
    fn new() -> Self {
        let map = vec![Some(NodeId::FALSE), Some(NodeId::TRUE)];
        Self(map)
    }

    /// Record that a given AIGER literal corresponds to a given fresh `NodeID`.
    fn add(&mut self, literal: usize, id: NodeId) {
        // TODO crude
        if literal >= self.0.len() {
            self.0.resize(literal + 1, None);
        }

        if literal & 1 == 0 {
            // The literal is already positive.
            self.0[literal] = Some(id);
        } else {
            // The literal is negated; map the positive version instead.
            self.0[literal & !1] = Some(id.invert());
        }
    }

    /// Get the `NodeID` corresponding to a given AIGER literal.
    ///
    /// Panic if the literal is not present.
    fn get(&self, literal: usize) -> NodeId {
        let regular_lit = literal & !1;
        let is_inverted = (literal & 1) == 1;
        match self.0[regular_lit] {
            Some(regular_node) => {
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
