use std::collections::HashMap;
use std::io::{BufRead, Error};

use crate::aiger::{AigerHeader, lookup_aiger_literal, read_one_number_line};
use crate::graph::{AigGraph, NodeId};

pub fn parse_ascii_aiger_into_graph(
    header: AigerHeader,
    reader: &mut impl BufRead,
    pre_optimize: bool,
) -> Result<AigGraph, Error> {
    let mut graph: AigGraph = AigGraph::new();
    let mut lit_to_node: HashMap<usize, NodeId> = HashMap::new();

    lit_to_node.insert(0, NodeId::FALSE);
    lit_to_node.insert(1, NodeId::TRUE);

    for _ in 0..header.i {
        let input_lit: usize = read_one_number_line(reader)?;

        let input_id: NodeId = graph.add_input();
        lit_to_node.insert(input_lit, input_id);
    }

    let mut latch_inputs: Vec<(NodeId, usize)> = Vec::new();

    // note: we add Nodeid::FALSE because latches may
    // contain nodes that are not defined yet (ex. AND nodes),
    // so we put them in the graph but save them in a hashmap for later
    for _ in 0..header.l {
        let (latch_lit, latch_input_lit) = read_latch_line(reader)?;

        let latch_id = graph.add_latch(NodeId::FALSE);
        lit_to_node.insert(latch_lit, latch_id);
        latch_inputs.push((latch_id, latch_input_lit));
    }

    // same idea for outputs! save 'em for later
    let mut output_lits: Vec<usize> = Vec::new();

    for _ in 0..header.o {
        let output_lit = read_one_number_line(reader)?;
        output_lits.push(output_lit);
    }

    for _ in 0..header.a {
        let (lhs_lit, rhs0_lit, rhs1_lit) = read_and_line(reader)?;

        let left: NodeId = lookup_aiger_literal(rhs0_lit, &lit_to_node)?;
        let right: NodeId = lookup_aiger_literal(rhs1_lit, &lit_to_node)?;

        let and_id: NodeId = if pre_optimize {
            graph.add_and_optimized(left, right)
        } else {
            graph.add_and_raw(left, right)
        };

        lit_to_node.insert(lhs_lit, and_id);
    }

    // now resolve lateches!
    for (latch_id, latch_input_lit) in latch_inputs {
        let latch_input_id: NodeId = lookup_aiger_literal(latch_input_lit, &lit_to_node)?;
        graph.set_latch_input(latch_id, latch_input_id);
    }

    // now resolve outputs!
    for output_lit in output_lits {
        let output_id: NodeId = lookup_aiger_literal(output_lit, &lit_to_node)?;
        graph.add_output(output_id);
    }

    Ok(graph)
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
