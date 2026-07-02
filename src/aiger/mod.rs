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
    let mut parser = LineParser::default();
    parser.read_line(reader)?;

    let tag = parser.parse_word();
    let is_ascii = match tag {
        b"aag" => true,
        b"aig" => false,
        _ => panic!("Invalid tag, must be either 'aag' or 'aig'"),
    };

    let [max_var, num_inputs, num_latches, num_outputs, num_and_gates] = parser
        .parse_ints()
        .expect("Header must have format: aag/aig M I L O A");
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

/// A mapping from AIGER literal indices to our internal `NodeId`s.
#[derive(Default)]
struct Literals(Vec<Option<NodeId>>);

impl Literals {
    fn new(max_var: usize) -> Self {
        // We store one entry per *variable*: i.e., one entry in this mapping
        // covers both the regular and inverted literals corresponding to the
        // same node.
        let mut map = vec![None; max_var + 1];
        map[0] = Some(NodeId::FALSE);
        Self(map)
    }

    /// Split a literal into a variable index and an inverted flag.
    fn split(literal: usize) -> (usize, bool) {
        (literal >> 1, literal & 1 == 1)
    }

    /// Record that a given AIGER literal corresponds to a given fresh `NodeID`.
    fn add(&mut self, literal: usize, id: NodeId) {
        let (var_idx, inverted) = Self::split(literal);
        self.0[var_idx] = Some(if inverted { id.invert() } else { id });
    }

    /// Get the `NodeID` corresponding to a given AIGER literal.
    ///
    /// Panic if the literal is not present.
    fn get(&self, literal: usize) -> NodeId {
        let (var_idx, inverted) = Self::split(literal);
        match self.0[var_idx] {
            Some(regular_node) => {
                if inverted {
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

    /// Consume a sequence of non-whitespace bytes.
    pub fn parse_word(&mut self) -> &[u8] {
        let start_pos = self.pos;
        while self.pop_if(|b| !b.is_ascii_whitespace()).is_some() {}
        &self.buf[start_pos..self.pos]
    }
}

/// A utility for reading integers from lines in a text stream.
///
/// This wraps `LineReader` in utilities for consuming lines directly from a
/// `BufRead` stream.
struct LineReader<'a, R: BufRead> {
    reader: &'a mut R,
    parser: LineParser,
}

impl<'a, R: BufRead> LineReader<'a, R> {
    fn new(reader: &'a mut R) -> Self {
        LineReader {
            reader,
            parser: LineParser::default(),
        }
    }

    /// Read a line containing a single integer.
    fn read_int(&mut self) -> std::io::Result<Option<usize>> {
        self.parser.read_line(self.reader)?;
        Ok(self.parser.parse_int())
    }

    /// Read a line containing a whitespace-separated list of integers.
    fn read_ints<const N: usize>(&mut self) -> std::io::Result<Option<[usize; N]>> {
        self.parser.read_line(self.reader)?;
        Ok(self.parser.parse_ints())
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
