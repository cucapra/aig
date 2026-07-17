use std::io::BufRead;

use super::eval::Value;

/// Parses text stimulus vectors from any buffered reader.
pub struct StimulusParser<R: BufRead>(R);

impl<R: BufRead> StimulusParser<R> {
    pub fn new(reader: R) -> Self {
        Self(reader)
    }
}

/// Something that supplies one input vector per clock cycle.
pub trait Stimulus {
    /// The type containing one clock cycle's input values.
    type Vector: AsRef<[Value]>;

    /// Get the input vector for the next clock cycle.
    ///
    /// `Some(...)` means another vector was read.
    /// `None` means the stimulus is finished.
    ///
    /// This may panic if the stimulus contains invalid input.
    fn next_vector(&mut self) -> Option<Self::Vector>;
}

/// Allow the existing `&[Vec<Value>]` format to be used as a stimulus.
///
/// The returned vector and original data structure must have the same
/// lifetime.
impl<'a> Stimulus for &'a [Vec<Value>] {
    type Vector = &'a [Value];

    fn next_vector(&mut self) -> Option<Self::Vector> {
        let (first, remaining) = (*self).split_first()?;

        *self = remaining;

        Some(first.as_slice())
    }
}

/// Allow a buffered text file using the format from the C library
/// to be used directly as a stimulus.
///
/// Expected format:
///
/// 010
/// 111
/// 000
/// .
///
/// Each line is one clock cycle, and `.` ends the stimulus.
impl<R: BufRead> Stimulus for StimulusParser<R> {
    type Vector = Vec<Value>;

    fn next_vector(&mut self) -> Option<Self::Vector> {
        let mut line = String::new();

        let bytes_read = self
            .0
            .read_line(&mut line)
            .expect("failed to read stimulus file");

        // End of file.
        if bytes_read == 0 {
            return None;
        }

        // `read_line` keeps the newline, so remove "\n" or "\r\n".
        let line = line.trim_end_matches(&['\r', '\n'][..]);

        // The C stimulus format uses "." to end the input sequence.
        if line == "." {
            return None;
        }

        let values: Vec<_> = line
            .bytes()
            .enumerate()
            .map(|(column, byte)| match byte {
                b'0' => 0,

                // True is represented by all 1 bits because eval uses
                // bitwise operations such as `!` and `&`.
                b'1' => Value::MAX,

                b'x' => {
                    panic!("column {}: 'x' is not currently supported", column + 1);
                }

                other => {
                    panic!(
                        "column {}: expected '0' or '1', found {:?}",
                        column + 1,
                        char::from(other)
                    );
                }
            })
            .collect();

        Some(values)
    }
}
