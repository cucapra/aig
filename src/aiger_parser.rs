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

#[derive(Debug)]
pub struct Aiger {
    pub header: AigerHeader,
    pub inputs: Vec<usize>,
    pub latches: Vec<AigerLatch>,
    pub outputs: Vec<usize>,
    pub ands: Vec<AigerAnd>,
}

#[derive(Debug)]
pub struct AigerLatch {
    pub current: usize,
    pub next: usize,
}

#[derive(Debug)]
pub struct AigerAnd {
    pub lhs: usize,
    pub rhs0: usize,
    pub rhs1: usize,
}

pub fn run_parser(file_name: &str) -> io::Result<()> {
    let file: File = File::open(Path::new(file_name))?;
    let mut buffer_reader: BufReader<File> = BufReader::new(file);

    let header: AigerHeader = verify_aiger_header(&mut buffer_reader)?;
    println!("Input file is a valid aiger file. Layout: {:?}", header);

    if !header.is_ascii {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Binary aiger parsing is not supported yet",
        ));
    }

    let aiger = parse_ascii_aiger_body(header, &mut buffer_reader)?;
    println!("Parsed AIGER: {:#?}", aiger);

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

fn read_one_number_line(reader: &mut impl BufRead, section_name: &str) -> io::Result<usize> {
    let mut line: String = String::new();
    let bytes_read: usize = reader.read_line(&mut line)?;

    if bytes_read == 0 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            format!("Unexpected end of file while reading {}", section_name),
        ));
    }

    let trimmed: &str = line.trim();

    trimmed.parse::<usize>().map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Invalid {} line: {}", section_name, trimmed),
        )
    })
}

fn read_latch_line(reader: &mut impl BufRead) -> io::Result<AigerLatch> {
    let mut line: String = String::new();
    let bytes_read: usize = reader.read_line(&mut line)?;

    if bytes_read == 0 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "Unexpected end of file while reading latch",
        ));
    }

    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() != 2 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Invalid latch line: {}", line.trim()),
        ));
    }

    let current: usize = parts[0].parse().map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Invalid latch current literal: {}", parts[0]),
        )
    })?;

    let next: usize = parts[1].parse().map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Invalid latch next literal: {}", parts[1]),
        )
    })?;

    Ok(AigerLatch { current, next })
}

fn read_and_line(reader: &mut impl BufRead) -> io::Result<AigerAnd> {
    let mut line: String = String::new();
    let bytes_read: usize = reader.read_line(&mut line)?;

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

    let lhs: usize = parts[0].parse().map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Invalid AND lhs literal: {}", parts[0]),
        )
    })?;

    let rhs0: usize = parts[1].parse().map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Invalid AND rhs0 literal: {}", parts[1]),
        )
    })?;

    let rhs1: usize = parts[2].parse().map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Invalid AND rhs1 literal: {}", parts[2]),
        )
    })?;

    Ok(AigerAnd { lhs, rhs0, rhs1 })
}

pub fn parse_ascii_aiger_body(header: AigerHeader, reader: &mut impl BufRead) -> io::Result<Aiger> {
    let mut inputs: Vec<usize> = Vec::new();
    let mut latches: Vec<AigerLatch> = Vec::new();
    let mut outputs: Vec<usize> = Vec::new();
    let mut ands: Vec<AigerAnd> = Vec::new();

    for _ in 0..header.i {
        let input: usize = read_one_number_line(reader, "input")?;
        inputs.push(input);
    }

    for _ in 0..header.l {
        let latch = read_latch_line(reader)?;
        latches.push(latch);
    }

    for _ in 0..header.o {
        let output = read_one_number_line(reader, "output")?;
        outputs.push(output);
    }

    for _ in 0..header.a {
        let and_gate = read_and_line(reader)?;
        ands.push(and_gate);
    }

    Ok(Aiger {
        header,
        inputs,
        latches,
        outputs,
        ands,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parses_simple_and_gate_header() {
        let input: &[u8; 26] = b"aag 3 2 0 1 1\n2\n4\n6\n6 2 4\n";
        let mut reader: Cursor<&[u8; 26]> = Cursor::new(input);

        let header: AigerHeader = verify_aiger_header(&mut reader).unwrap();

        assert!(header.is_ascii);
        assert_eq!(header.m, 3);
        assert_eq!(header.i, 2);
        assert_eq!(header.l, 0);
        assert_eq!(header.o, 1);
        assert_eq!(header.a, 1);
    }

    #[test]
    fn rejects_invalid_tag() {
        let input: &[u8; 14] = b"bad 3 2 0 1 1\n";
        let mut reader: Cursor<&[u8; 14]> = Cursor::new(input);

        let result: Result<AigerHeader, Error> = verify_aiger_header(&mut reader);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_missing_header_number() {
        let input: &[u8; 12] = b"aag 3 2 0 1\n";
        let mut reader: Cursor<&[u8; 12]> = Cursor::new(input);

        let result: Result<AigerHeader, Error> = verify_aiger_header(&mut reader);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_structural_violation() {
        let input: &[u8; 14] = b"aag 2 2 0 1 1\n";
        let mut reader: Cursor<&[u8; 14]> = Cursor::new(input);

        let result: Result<AigerHeader, Error> = verify_aiger_header(&mut reader);

        assert!(result.is_err());
    }
}
