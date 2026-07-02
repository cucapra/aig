use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, Error, ErrorKind};

pub mod aiger;
pub mod graph;

use aiger::run_parser_with_options;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Usage: aig <input.aag|input.aig|-> [--pre-optimize] [--stdout] [--parse-only]",
        ));
    }

    let input: &String = &args[1];

    let pre_optimize: bool = args.iter().any(|arg| arg == "--pre-optimize");
    let write_to_stdout: bool = args.iter().any(|arg| arg == "--stdout");

    let graph: graph::AigGraph = if input == "-" {
        let stdin: io::Stdin = io::stdin();
        let mut reader: BufReader<io::StdinLock<'_>> = BufReader::new(stdin.lock());

        run_parser_with_options(&mut reader, pre_optimize)?
    } else {
        let file: File = File::open(input)?;
        let mut reader: BufReader<File> = BufReader::new(file);

        run_parser_with_options(&mut reader, pre_optimize)?
    };

    if !args.iter().any(|arg| arg == "--parse-only") {
        let dot: String = graph.to_dot();
        if write_to_stdout {
            print!("{}", dot);
        } else {
            fs::write("graph.dot", dot)?;
            println!("Wrote graph.dot");
        }
    }

    Ok(())
}
