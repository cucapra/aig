use std::io::{BufRead, Error};

use crate::aiger::{AigerHeader, Literals};
use crate::graph::{AigBuilder, AigGraph, NodeId};

pub fn parse_ascii_aiger_into_graph(
    header: AigerHeader,
    reader: &mut impl BufRead,
    pre_optimize: bool,
) -> Result<AigGraph, Error> {
    let mut graph = AigBuilder::new();
    let mut literals = Literals::new();
    let mut line_reader = LineReader::new(reader);

    for _ in 0..header.num_inputs {
        let input_lit = line_reader.read_int()?.expect("malformed input line");

        let input_id: NodeId = graph.add_input();
        literals.add(input_lit, input_id);
    }

    let mut latch_inputs: Vec<(NodeId, usize)> = Vec::with_capacity(header.num_latches);

    // note: we add Nodeid::FALSE because latches may
    // contain nodes that are not defined yet (ex. AND nodes),
    // so we put them in the graph but save them in a hashmap for later
    for _ in 0..header.num_latches {
        let [latch_lit, latch_input_lit] = line_reader.read_ints()?;

        let latch_id = graph.add_latch(NodeId::FALSE);
        literals.add(latch_lit, latch_id);
        latch_inputs.push((latch_id, latch_input_lit));
    }

    // same idea for outputs! save 'em for later
    let mut output_lits: Vec<usize> = Vec::with_capacity(header.num_outputs);

    for _ in 0..header.num_outputs {
        let output_lit = line_reader.read_int()?.expect("malformed output line");
        output_lits.push(output_lit);
    }

    for _ in 0..header.num_and_gates {
        let [lhs_lit, rhs0_lit, rhs1_lit] = line_reader.read_ints()?;

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

    fn read_line(&mut self) -> std::io::Result<()> {
        self.parser.clear();
        self.reader.read_until(b'\n', &mut self.parser.buf)?;
        // TODO detect EOF
        Ok(())
    }

    fn read_int(&mut self) -> std::io::Result<Option<usize>> {
        self.read_line()?;
        Ok(self.parser.parse_int())
    }

    fn read_ints<const N: usize>(&mut self) -> std::io::Result<[usize; N]> {
        self.read_line()?;
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
        assert_eq!(parser.parse_ints(), [42, 27]);
        assert_eq!(parser.rest().len(), 2);
    }
}
