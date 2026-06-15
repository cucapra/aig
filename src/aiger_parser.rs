use std::fs::File;
use std::io::{self, BufReader, Error, ErrorKind, Read};
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

pub fn verify_aiger_header(reader: &mut impl Read) -> io::Result<AigerHeader> {
    let mut tag: [u8; 4] = [0; 4];
    reader.read_exact(&mut tag)?;

    let is_ascii: bool = match &tag {
        b"aag " => true,
        b"aig " => false,
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid tag, must be either 'aag' or 'aig'",
            ));
        }
    };

    let mut numbers: [usize; 5] = [0; 5];
    let mut num_index: usize = 0;
    let mut current_val: usize = 0;
    let mut byte: [u8; 1] = [0; 1];

    loop {
        reader.read_exact(&mut byte)?;
        let b: u8 = byte[0];

        if b == b' ' || b == b'\n' {
            if num_index >= 5 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Too many header variables",
                ));
            }

            numbers[num_index] = current_val;
            num_index += 1;
            current_val = 0;

            if b == b'\n' {
                break;
            }
        } else if b.is_ascii_digit() {
            let digit: usize = (b - b'0') as usize;

            current_val = current_val
                .checked_mul(10)
                .and_then(|v| v.checked_add(digit))
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Integer overflow in header"))?;
        } else {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Invalid character in header",
            ));
        }
    }

    if num_index != 5 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Missing header variables",
        ));
    }

    let m: usize = numbers[0];
    let i: usize = numbers[1];
    let l: usize = numbers[2];
    let o: usize = numbers[3];
    let a: usize = numbers[4];

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
