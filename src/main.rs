use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub mod aiger;
pub mod graph;

use aiger::run_parser_with_options;

#[derive(Parser, Debug)]
#[command(version, about = "AIGER command-line tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Parse an AIGER file, but do not write any .dot output
    Parse {
        /// Input .aag/.aig file, or '-' to read from stdin
        input: String,

        /// Optimize while constructing the graph
        #[arg(long)]
        pre_optimize: bool,

        /// Print AIG graph
        #[arg(long)]
        print: bool,
    },

    /// Simulate an AIGER circuit using a stimulus file
    Simulate {
        /// Input .aag/.aig file
        input: String,

        /// Stimulus file containing one 0/1 input vector per line
        stimulus: String,

        /// Optimize while constructing the graph
        #[arg(long)]
        pre_optimize: bool,
    },

    /// Convert an ASCII AIGER file to binary AIGER, or binary AIGER to ASCII
    Convert {
        /// Input .aag/.aig file, or '-' to read from stdin
        input: String,

        /// Output .aag/.aig name and location file
        /// examples:
        ///   --output aiger.aag
        ///   --output ./aiger.aag
        ///   --output /Users/Modi/Projects/AIG/aiger.aag
        #[arg(short, long, value_parser = parse_aiger_output_path)]
        output: Option<PathBuf>,
    },

    /// Parse an AIGER file and produce Graphviz DOT output
    Dot {
        /// Input .aag/.aig file, or '-' to read from stdin
        input: String,

        /// Optimize while constructing the graph
        #[arg(long)]
        pre_optimize: bool,

        /// Output .dot name and location file
        /// examples:
        ///   --output graph.dot
        ///   --output ./graph.dot
        ///   --output /Users/Modi/Projects/AIG/graph.dot
        #[arg(short, long, value_parser = parse_dot_path)]
        output: Option<PathBuf>,
    },
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse {
            input,
            pre_optimize,
            print,
        } => {
            let graph = parse_input(&input, pre_optimize)?;

            if print {
                println!("{graph:#?}");
            }
        }

        Commands::Simulate {
            input,
            stimulus,
            pre_optimize,
        } => {
            let graph = parse_input(&input, pre_optimize)?;

            let stimulus_file = File::open(&stimulus)?;
            let stimulus_reader = BufReader::new(stimulus_file);

            let output_trace = graph.simulate(stimulus_reader);

            // Print one output vector per clock cycle.
            for outputs in output_trace {
                for value in outputs {
                    if value == 0 {
                        print!("0");
                    } else {
                        print!("1");
                    }
                }

                println!();
            }
        }

        Commands::Convert { input, output } => {
            todo!("implement conversion logic for {input} with output {output:?}");
        }

        Commands::Dot {
            input,
            pre_optimize,
            output,
        } => {
            let graph = parse_input(&input, pre_optimize)?;
            let dot: String = graph.to_dot();

            if let Some(output) = output {
                fs::write(&output, &dot)?;
                println!("Wrote dot file to {}", output.display());
            } else {
                print!("{dot}");
            }
        }
    }

    Ok(())
}

fn parse_input(input: &str, pre_optimize: bool) -> io::Result<graph::AigGraph> {
    if input == "-" {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin.lock());

        run_parser_with_options(&mut reader, pre_optimize)
    } else {
        let file = File::open(input)?;
        let mut reader = BufReader::new(file);

        run_parser_with_options(&mut reader, pre_optimize)
    }
}

fn parse_dot_path(s: &str) -> Result<PathBuf, String> {
    if s.ends_with(".dot") {
        Ok(PathBuf::from(s))
    } else {
        Err(format!("output file must end with .dot: {s}"))
    }
}

fn parse_aiger_output_path(s: &str) -> Result<PathBuf, String> {
    if s.ends_with(".aag") || s.ends_with(".aig") {
        Ok(PathBuf::from(s))
    } else {
        Err(format!("output file must end with .aag or .aig: {s}"))
    }
}

