use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, Error, ErrorKind};

mod aig_graph;
mod aiger_ascii_parser;
mod aiger_binary_parser;
mod aiger_parser;

use crate::aiger_parser::run_parser_with_options;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Usage: cargo run -- <input.aag|input.aig|-> [--pre-optimize]",
        ));
    }

    let input: &String = &args[1];
    let pre_optimize: bool = args.iter().any(|arg| arg == "--pre-optimize");

    let graph: aig_graph::AigGraph = if input == "-" {
        let stdin: io::Stdin = io::stdin();
        let mut reader: BufReader<io::StdinLock<'_>> = BufReader::new(stdin.lock());

        run_parser_with_options(&mut reader, pre_optimize)?
    } else {
        let file: File = File::open(input)?;
        let mut reader: BufReader<File> = BufReader::new(file);

        run_parser_with_options(&mut reader, pre_optimize)?
    };

    fs::write("graph.dot", graph.to_dot())?;

    println!("Wrote graph.dot");

    Ok(())
}
