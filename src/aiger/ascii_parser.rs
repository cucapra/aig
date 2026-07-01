use std::io::{BufRead, Error};

use crate::aiger::{AigerHeader, Literals, read_one_number_line};
use crate::graph::{AigBuilder, AigGraph, NodeId};

pub fn parse_ascii_aiger_into_graph(
    header: AigerHeader,
    reader: &mut impl BufRead,
    pre_optimize: bool,
) -> Result<AigGraph, Error> {
    let mut graph = AigBuilder::new();
    let mut literals = Literals::new();

    for _ in 0..header.num_inputs {
        let input_lit: usize = read_one_number_line(reader)?;

        let input_id: NodeId = graph.add_input();
        literals.add(input_lit, input_id);
    }

    let mut latch_inputs: Vec<(NodeId, usize)> = Vec::with_capacity(header.num_latches);

    // note: we add Nodeid::FALSE because latches may
    // contain nodes that are not defined yet (ex. AND nodes),
    // so we put them in the graph but save them in a hashmap for later
    for _ in 0..header.num_latches {
        let (latch_lit, latch_input_lit) = read_latch_line(reader)?;

        let latch_id = graph.add_latch(NodeId::FALSE);
        literals.add(latch_lit, latch_id);
        latch_inputs.push((latch_id, latch_input_lit));
    }

    // same idea for outputs! save 'em for later
    let mut output_lits: Vec<usize> = Vec::with_capacity(header.num_outputs);

    for _ in 0..header.num_outputs {
        let output_lit = read_one_number_line(reader)?;
        output_lits.push(output_lit);
    }

    for _ in 0..header.num_and_gates {
        let (lhs_lit, rhs0_lit, rhs1_lit) = read_and_line(reader)?;

        let left: NodeId = literals.get(rhs0_lit);
        let right: NodeId = literals.get(rhs1_lit);

        let and_id: NodeId = if pre_optimize {
            graph.add_and_optimized(left, right)
        } else {
            graph.add_and_raw(left, right)
        };

        literals.add(lhs_lit, and_id);
    }

    // now resolve lateches!
    for (latch_id, latch_input_lit) in latch_inputs {
        let latch_input_id: NodeId = literals.get(latch_input_lit);
        graph.node(latch_id).set_latch_input(latch_input_id);
    }

    // now resolve outputs!
    for output_lit in output_lits {
        let output_id: NodeId = literals.get(output_lit);
        graph.add_output(output_id);
    }

    Ok(graph.build())
}

fn read_latch_line(reader: &mut impl BufRead) -> Result<(usize, usize), Error> {
    let mut line: String = String::new();

    if (reader.read_line(&mut line)?) == 0 {
        panic!("no data read from number line")
    }

    let parts: Vec<&str> = line.split_whitespace().collect();

    let latch: usize = parts[0].parse().unwrap();
    let input: usize = parts[1].parse().unwrap();

    Ok((latch, input))
}

fn read_and_line(reader: &mut impl BufRead) -> Result<(usize, usize, usize), Error> {
    let mut line: String = String::new();

    if (reader.read_line(&mut line)?) == 0 {
        panic!("no data read from number line")
    }

    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() != 3 {
        panic!("and gate must have 3 parts")
    }

    let lhs: usize = parts[0].parse().unwrap();
    let rhs0: usize = parts[1].parse().unwrap();
    let rhs1: usize = parts[2].parse().unwrap();

    Ok((lhs, rhs0, rhs1))
}

#[derive(Default)]
struct LineParser {
    buf: Vec<u8>,
    pos: usize,
}

impl LineParser {
    fn new(buf: Vec<u8>) -> Self {
        Self { buf, pos: 0 }
    }

    fn clear(&mut self) {
        self.buf.clear();
        self.pos = 0;
    }

    fn rest(&self) -> &[u8] {
        &self.buf[self.pos..]
    }

    fn read<R: BufRead>(&mut self, stream: &mut R) -> std::io::Result<()> {
        self.clear();
        stream.read_until(b'\n', &mut self.buf)?;
        // TODO detect EOF
        Ok(())
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

    fn parse_int(&mut self) -> Option<usize> {
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

    fn skip_whitespace(&mut self) {
        while self.pop_if(|b| b.is_ascii_whitespace()).is_some() {}
    }

    fn parse_ints<const N: usize>(&mut self) -> [usize; N] {
        std::array::from_fn(|_| {
            self.skip_whitespace();
            self.parse_int().expect("not enough ints")
        })
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
        assert_eq!(parser.parse_ints(), [42, 27]);
        assert_eq!(parser.rest().len(), 2);
    }
}
