use std::collections::HashMap;
use std::io::{BufRead, Error, Read};

use crate::aiger::{AigerHeader, lookup_aiger_literal, read_one_number_line};
use crate::graph::{AigGraph, NodeId};

pub fn parse_binary_aiger_into_graph(
    header: AigerHeader,
    reader: &mut impl BufRead,
    pre_optimize: bool,
) -> Result<AigGraph, Error> {
    let mut graph: AigGraph = AigGraph::new();
    let mut lit_to_node: HashMap<usize, NodeId> = HashMap::new();

    lit_to_node.insert(0, NodeId::FALSE);
    lit_to_node.insert(1, NodeId::TRUE);

    for input_index in 0..header.i {
        let input_lit: usize = 2 * (input_index + 1);
        let input_id: NodeId = graph.add_input();
        lit_to_node.insert(input_lit, input_id);
    }

    // like in ascii parser, save latches for later
    let mut latch_inputs: Vec<(NodeId, usize)> = Vec::new();

    for latch_index in 0..header.l {
        let latch_input_lit: usize = read_one_number_line(reader)?;
        let latch_lit: usize = 2 * (header.i + latch_index + 1);
        let latch_id: NodeId = graph.add_latch(NodeId::FALSE);

        lit_to_node.insert(latch_lit, latch_id);
        latch_inputs.push((latch_id, latch_input_lit));
    }
    // same idea for outputs
    let mut output_lits: Vec<usize> = Vec::new();

    for _ in 0..header.o {
        let output_lit: usize = read_one_number_line(reader)?;
        output_lits.push(output_lit);
    }

    for and_index in 0..header.a {
        let lhs_lit: usize = 2 * (header.i + header.l + and_index + 1);
        let delta0: usize = read_delta(reader)?;
        let delta1: usize = read_delta(reader)?;

        let rhs0_lit: usize = lhs_lit - delta0;
        let rhs1_lit: usize = rhs0_lit - delta1;

        let left: NodeId = lookup_aiger_literal(rhs0_lit, &lit_to_node)?;
        let right: NodeId = lookup_aiger_literal(rhs1_lit, &lit_to_node)?;

        let and_id = if pre_optimize {
            graph.add_and_optimized(left, right)
        } else {
            graph.add_and_raw(left, right)
        };

        lit_to_node.insert(lhs_lit, and_id);
    }

    // resolve lacthes
    for (latch_id, latch_input_lit) in latch_inputs {
        let latch_input_id: NodeId = lookup_aiger_literal(latch_input_lit, &lit_to_node)?;
        graph.set_latch_input(latch_id, latch_input_id);
    }

    // resolve outputs
    for output_lit in output_lits {
        let output_id: NodeId = lookup_aiger_literal(output_lit, &lit_to_node)?;
        graph.add_output(output_id);
    }

    Ok(graph)
}

/// Decodes the binary-encoded AND gate representation.
///
/// In binary AIGER, each AND gate is stored using two deltas values, where:
///
/// ```text
/// delta0 = lhs  - rhs0
/// delta1 = rhs0 - rhs1
/// ```
///
/// Given `lhs`, these deltas let us recover:
///
/// ```text
/// rhs0 = lhs  - delta0
/// rhs1 = rhs0 - delta1
/// ```
///
/// Each delta is encoded as a variable-length little-endian integer.
/// In each byte, the most significant bit is a continuation bit:
///
/// - `1` means the integer continues in the next byte.
/// - `0` means this byte is the last byte of the current integer.
///
/// Therefore, the decoder first reads bytes until it finishes `delta0`,
/// then reads bytes until it finishes `delta1` (on the second call to the function, resuing the same reader)
///
/// AIGER requires the ordering:
///
/// ```text
/// lhs > rhs0 >= rhs1
/// ```
///
/// This guarantees that both deltas are nonnegative. In practice, the
/// deltas are usually small, which makes the encoding nice and compact!
fn read_delta(reader: &mut impl Read) -> Result<usize, Error> {
    let mut value: usize = 0usize;
    let mut shift: u32 = 0u32;

    loop {
        let mut byte: [u8; 1] = [0u8; 1];
        reader.read_exact(&mut byte)?;

        // removes the top bit and keeps only the lower 7 bits
        let chunk: usize = (byte[0] & 0b01111111) as usize;

        // inserts that 7-bit chunk into the correct place in the final number
        value |= chunk << shift;

        if (byte[0] & 0b10000000) == 0 {
            return Ok(value);
        }

        shift += 7;
    }
}
