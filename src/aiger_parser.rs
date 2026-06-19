use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Error, ErrorKind};
use std::path::Path;

use crate::aig_graph::{AigGraph, NodeId};

#[derive(Debug)]
pub struct AigerHeader {
    pub is_ascii: bool,
    pub m: usize,
    pub i: usize,
    pub l: usize,
    pub o: usize,
    pub a: usize,
}

pub fn run_parser(file_name: &str) -> io::Result<()> {
    run_parser_with_options(file_name, false)
}

pub fn run_parser_with_options(file_name: &str, pre_optimize: bool) -> io::Result<()> {
    let file = File::open(Path::new(file_name))?;
    let mut reader = BufReader::new(file);

    let header = verify_aiger_header(&mut reader)?;
    println!("Input file is a valid AIGER file. Layout: {:?}", header);

    if !header.is_ascii {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Binary AIGER parsing is not supported yet",
        ));
    }

    let graph = parse_ascii_aiger_into_graph(header, &mut reader, pre_optimize)?;
    println!("Parsed AIG graph: {:#?}", graph);

    Ok(())
}

pub fn verify_aiger_header(reader: &mut impl BufRead) -> io::Result<AigerHeader> {
    let mut line = String::new();
    reader.read_line(&mut line)?;

    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() != 6 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Header must have format: aag/aig M I L O A",
        ));
    }

    let is_ascii = match parts[0] {
        "aag" => true,
        "aig" => false,
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid tag, must be either 'aag' or 'aig'",
            ));
        }
    };

    let m = parse_header_number(parts[1], "M")?;
    let i = parse_header_number(parts[2], "I")?;
    let l = parse_header_number(parts[3], "L")?;
    let o = parse_header_number(parts[4], "O")?;
    let a = parse_header_number(parts[5], "A")?;

    if m < i + l + a {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Structural layout violation: M < I + L + A",
        ));
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

fn parse_header_number(s: &str, name: &str) -> io::Result<usize> {
    s.parse::<usize>().map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Invalid header number for {}", name),
        )
    })
}

pub fn parse_ascii_aiger_into_graph(
    header: AigerHeader,
    reader: &mut impl BufRead,
    pre_optimize: bool,
) -> io::Result<AigGraph> {
    if header.l != 0 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Latches are not supported yet",
        ));
    }

    let mut graph = AigGraph::new();

    let mut lit_to_node: HashMap<usize, NodeId> = HashMap::new();

    lit_to_node.insert(0, graph.const_false());
    lit_to_node.insert(1, graph.const_true());

    for _ in 0..header.i {
        let input_lit = read_one_number_line(reader, "input")?;

        if input_lit == 0 || input_lit == 1 || input_lit % 2 != 0 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Input literal must be a positive even literal, got {}",
                    input_lit
                ),
            ));
        }

        if lit_to_node.contains_key(&input_lit) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Duplicate literal definition: {}", input_lit),
            ));
        }

        let input_id = graph.add_input();
        lit_to_node.insert(input_lit, input_id);
    }

    let mut output_lits: Vec<usize> = Vec::new();

    for _ in 0..header.o {
        let output_lit = read_one_number_line(reader, "output")?;
        output_lits.push(output_lit);
    }

    for _ in 0..header.a {
        let (lhs_lit, rhs0_lit, rhs1_lit) = read_and_line(reader)?;

        if lhs_lit == 0 || lhs_lit == 1 || lhs_lit % 2 != 0 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("AND lhs must be a positive even literal, got {}", lhs_lit),
            ));
        }

        if lit_to_node.contains_key(&lhs_lit) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Duplicate literal definition: {}", lhs_lit),
            ));
        }

        let left = lookup_aiger_literal(rhs0_lit, &lit_to_node)?;
        let right = lookup_aiger_literal(rhs1_lit, &lit_to_node)?;

        let and_id = if pre_optimize {
            graph.add_and_optimized(left, right)
        } else {
            graph.add_and_raw(left, right)
        };

        lit_to_node.insert(lhs_lit, and_id);
    }

    for output_lit in output_lits {
        let output_id = lookup_aiger_literal(output_lit, &lit_to_node)?;
        graph.add_output(output_id);
    }

    Ok(graph)
}

fn lookup_aiger_literal(
    aiger_lit: usize,
    lit_to_node: &HashMap<usize, NodeId>,
) -> io::Result<NodeId> {
    if let Some(node_id) = lit_to_node.get(&aiger_lit).copied() {
        return Ok(node_id);
    }

    let regular_lit = aiger_lit & !1;
    let is_inverted = (aiger_lit & 1) == 1;

    let regular_node = lit_to_node.get(&regular_lit).copied().ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidData,
            format!(
                "Literal {} refers to undefined literal {}",
                aiger_lit, regular_lit
            ),
        )
    })?;

    if is_inverted {
        Ok(regular_node.invert())
    } else {
        Ok(regular_node)
    }
}

fn read_one_number_line(reader: &mut impl BufRead, section_name: &str) -> io::Result<usize> {
    let mut line = String::new();
    let bytes_read = reader.read_line(&mut line)?;

    if bytes_read == 0 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            format!("Unexpected end of file while reading {}", section_name),
        ));
    }

    let trimmed = line.trim();

    trimmed.parse::<usize>().map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Invalid {} line: {}", section_name, trimmed),
        )
    })
}

fn read_and_line(reader: &mut impl BufRead) -> io::Result<(usize, usize, usize)> {
    let mut line = String::new();
    let bytes_read = reader.read_line(&mut line)?;

    if bytes_read == 0 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "Unexpected end of file while reading AND gate",
        ));
    }

    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() != 3 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Invalid AND line: {}", line.trim()),
        ));
    }

    let lhs = parts[0].parse::<usize>().map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Invalid AND lhs literal: {}", parts[0]),
        )
    })?;

    let rhs0 = parts[1].parse::<usize>().map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Invalid AND rhs0 literal: {}", parts[1]),
        )
    })?;

    let rhs1 = parts[2].parse::<usize>().map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Invalid AND rhs1 literal: {}", parts[2]),
        )
    })?;

    Ok((lhs, rhs0, rhs1))
}
