use std::fs::File;
use std::io::{self, BufRead, BufReader, Error, ErrorKind};
use std::path::Path;

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
    let file: File = File::open(Path::new(file_name))?;
    let mut buffer_reader: BufReader<File> = BufReader::new(file);

    let header: AigerHeader = verify_aiger_header(&mut buffer_reader)?;
    println!("Input file is a valid aiger file. Layout: {:?}", header);

    Ok(())
}

pub fn verify_aiger_header(reader: &mut impl BufRead) -> io::Result<AigerHeader> {
    let mut line: String = String::new();
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

    let m: usize = parse_header_number(parts[1], "M")?;
    let i: usize = parse_header_number(parts[2], "I")?;
    let l: usize = parse_header_number(parts[3], "L")?;
    let o: usize = parse_header_number(parts[4], "O")?;
    let a: usize = parse_header_number(parts[5], "A")?;

    if m < i + l + a {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Structural layout violation: M < I + L + A",
        ));
    }

    Ok(AigerHeader {is_ascii, m, i, l, o, a,})
}

fn parse_header_number(s: &str, name: &str) -> io::Result<usize> {
    s.parse::<usize>().map_err(|_| { Error::new(
            ErrorKind::InvalidData,
            format!("Invalid header number for {}", name),
        )
    })
}
