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
        if literal & 1 == 0 {
            // The literal is already positive.
            self.0.insert(literal, id);
        } else {
            // The literal is negated; map the positive version instead.
            self.0.insert(literal & !1, id.invert());
        }
    }

    /// Get the `NodeID` corresponding to a given AIGER literal.
    ///
    /// Panic if the literal is not present.
    fn get(&self, literal: usize) -> NodeId {
        let regular_lit = literal & !1;
        let is_inverted = (literal & 1) == 1;
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

/// A utility for parsing integers from lines in text files.
#[derive(Default)]
pub struct LineParser {
    pub buf: Vec<u8>,
    pub pos: usize,
}

impl LineParser {
    pub fn new(buf: Vec<u8>) -> Self {
        Self { buf, pos: 0 }
    }

    /// Empty the line buffer.
    pub fn clear(&mut self) {
        self.buf.clear();
        self.pos = 0;
    }

    /// Get all the data remaining in the line buffer.
    pub fn rest(&self) -> &[u8] {
        &self.buf[self.pos..]
    }

    /// Read a line from a text-file stream. Return a flag indicating EOF.
    pub fn read_line<R: BufRead>(&mut self, reader: &mut R) -> std::io::Result<bool> {
        self.clear();
        reader.read_until(b'\n', &mut self.buf)?;
        Ok(self.buf.is_empty())
    }

    fn peek(&self) -> Option<u8> {
        self.buf.get(self.pos).copied()
    }

    fn skip(&mut self) {
        debug_assert!(self.pos < self.buf.len());
        self.pos += 1;
    }

    fn pop_if(&mut self, pred: impl Fn(u8) -> bool) -> Option<u8> {
        if let Some(byte) = self.peek()
            && pred(byte)
        {
            self.skip();
            Some(byte)
        } else {
            None
        }
    }

    /// Consume a single ASCII integer from the current point in the line buffer.
    pub fn parse_int(&mut self) -> Option<usize> {
        let mut out: Option<usize> = None;

        while let Some(byte) = self.pop_if(|b| b.is_ascii_digit()) {
            let value = byte - b'0';
            out = Some(match out {
                Some(old) => old * 10 + (value as usize),
                None => value as usize,
            });
        }

        out
    }

    /// Advance the line buffer to consume a sequence of ASCII whitespace.
    pub fn skip_whitespace(&mut self) {
        while self.pop_if(|b| b.is_ascii_whitespace()).is_some() {}
    }

    /// Consume a sequence of whitespace-separated integers.
    pub fn parse_ints<const N: usize>(&mut self) -> Option<[usize; N]> {
        // Sadly, `std::array::try_from_fn` is unstable. We fake it by setting a
        // flag on parsing errors and, on failure, wastefully continuing to
        // construct an array that we will eventually throw away.
        let mut failed = false;
        let arr = std::array::from_fn(|_| {
            self.skip_whitespace();
            match self.parse_int() {
                Some(i) => i,
                None => {
                    failed = true;
                    0
                }
            }
        });
        if failed { None } else { Some(arr) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn just_int() {
        let mut parser = LineParser::new("42".as_bytes().to_vec());
        assert_eq!(parser.parse_int(), Some(42));
        assert!(parser.rest().is_empty());
    }

    #[test]
    fn int_with_stuff() {
        let mut parser = LineParser::new("42x".as_bytes().to_vec());
        assert_eq!(parser.parse_int(), Some(42));
        assert_eq!(parser.rest().len(), 1);
    }

    #[test]
    fn two_ints() {
        let mut parser = LineParser::new("42 27 x".as_bytes().to_vec());
        assert_eq!(parser.parse_ints(), Some([42, 27]));
        assert_eq!(parser.rest().len(), 2);
    }
}
