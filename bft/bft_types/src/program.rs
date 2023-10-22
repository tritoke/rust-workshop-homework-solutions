use core::fmt;
use std::{
    collections::BTreeMap,
    error::Error,
    path::{Path, PathBuf},
};

use crate::Instruction;

/// A brainfuck Program
#[derive(Debug, Clone)]
pub struct Program {
    /// filename the program was created from
    filename: PathBuf,

    /// instructions contained within the file the program was loaded from
    instructions: Vec<Instruction>,
}

/// The alphabet of valid brainfuck characters
const BF_ALPHABET: &str = "><+-.,[]";

impl Program {
    /// Construct a new brainfuck program from a filename and it's contents
    ///
    /// `filename`: the file the program was loaded from
    /// `file_contents`: the contents of `filename`
    ///
    /// if `file_contents` is malformed (has unbalanced brackets) construction will fail
    /// reporting whether an unopened or unclosed bracket caused the failure, and the
    /// source code location of that failure.
    ///
    /// ```ignore
    /// # use bft_types::Program;
    /// let contents = include_bytes!("../../programs/example.bf");
    /// let program = Program::try_new("../../programs/example.bf", contents).unwrap();
    /// ```
    pub fn try_new(filename: &Path, file_contents: impl AsRef<str>) -> Result<Self, BfParseError> {
        // first filter out comment characters
        let mut tokens = Vec::new();
        let mut token_sources = Vec::new();
        for (line_no, line) in file_contents.as_ref().lines().enumerate() {
            for (column, c) in line.chars().enumerate() {
                if c.is_ascii() && BF_ALPHABET.contains(c) {
                    tokens.push(c as u8);
                    token_sources.push(SourceLocation {
                        line: line_no,
                        column,
                    })
                }
            }
        }

        // track of all the jump destinations
        let mut jumps = BTreeMap::new();
        let mut jump_stack = vec![];

        for (i, op) in tokens.iter().copied().enumerate() {
            if op == b'[' {
                jump_stack.push(i);
            } else if op == b']' {
                let jump_src = jump_stack.pop().ok_or_else(|| BfParseError {
                    filename: filename.to_owned(),
                    location: token_sources[i],
                    kind: BfParseErrorKind::UnopenedBracket,
                })?;

                // insert both the forward and backward jumps
                jumps.insert(jump_src, i);
                jumps.insert(i, jump_src);
            }
        }

        // if the jump stack has elements then there is an unbalanced open bracket
        if let Some(unclosed_brack) = jump_stack.pop() {
            return Err(BfParseError {
                filename: filename.to_owned(),
                location: token_sources[unclosed_brack],
                kind: BfParseErrorKind::UnclosedBracket,
            });
        }

        // construct the instructions
        let instrs = tokens
            .into_iter()
            .enumerate()
            .map(|(i, b)| match b {
                b'>' => Instruction::Inc,
                b'<' => Instruction::Dec,
                b'+' => Instruction::Succ,
                b'-' => Instruction::Pred,
                b'.' => Instruction::Out,
                b',' => Instruction::In,
                b'[' => Instruction::Jz {
                    dest: jumps[&i] + 1,
                },
                b']' => Instruction::Jnz {
                    dest: jumps[&i] + 1,
                },
                _ => unreachable!(
                    "domain precondition broken, invalid instruction present after filtering"
                ),
            })
            .collect();

        Ok(Self {
            filename: filename.to_owned(),
            instructions: instrs,
        })
    }

    /// Load a brainfuck program from a file:
    /// `filename`: the file to load the program from
    ///
    /// ```
    /// # use bft_types::Program;
    /// let program = Program::from_file("../../programs/example.bf");
    /// ```
    pub fn from_file<P: AsRef<Path>>(filename: P) -> Result<Self, Box<dyn Error>> {
        let path = filename.as_ref();
        let contents = std::fs::read_to_string(path)?;
        let filename = path.file_name().ok_or_else(|| {
            format!("Failed to load brainfuck program from {path:?} path doesn't point to a file.")
        })?;
        Ok(Self::try_new(Path::new(filename), contents)?)
    }

    /// name of the file this program was loaded from
    ///
    /// ```
    /// # use bft_types::Program;
    /// let program = Program::from_file("../programs/example.bf").unwrap();
    /// assert_eq!(program.filename().to_str(), Some("example.bf"));
    /// ```
    pub fn filename(&self) -> &Path {
        &self.filename
    }

    /// instructions contained by the program
    ///
    /// ```
    /// # use bft_types::Program;
    /// let program = Program::from_file("../programs/example.bf").unwrap();
    /// for instr in program.instructions() {
    ///     println!("{instr:?}");
    /// }
    /// ```
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }
}

/// location of a token in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    /// line of the token in the source code
    pub line: usize,
    /// column of the token in the source code
    pub column: usize,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {} column {}", self.line + 1, self.column + 1)
    }
}

/// errors that can occur while parsing brainfuck programs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BfParseErrorKind {
    /// There was an unclosed bracket in the program
    UnclosedBracket,
    /// There was an unopened bracket in the program
    UnopenedBracket,
}

/// used to hold extra metadata about the location and type of error encountered while parsing
/// brainfuck programs
#[derive(Debug, Clone)]
pub struct BfParseError {
    /// name of the file the error originated in
    filename: PathBuf,
    /// location in the file of the token causing the error
    location: SourceLocation,
    /// kind of error encountered
    kind: BfParseErrorKind,
}

impl fmt::Display for BfParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let BfParseError {
            filename,
            location,
            kind,
        } = self;
        let msg = match kind {
            BfParseErrorKind::UnclosedBracket => "dangling open bracket found at",
            BfParseErrorKind::UnopenedBracket => "dangling close bracket found at",
        };

        write!(
            f,
            "Error in input file {}, {msg} {location}",
            filename.display()
        )
    }
}

impl Error for BfParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let alphabet = "><+-.,[]";
        let prog = Program::try_new(Path::new("-"), alphabet).unwrap();
        let correct = [
            Instruction::Inc,
            Instruction::Dec,
            Instruction::Succ,
            Instruction::Pred,
            Instruction::Out,
            Instruction::In,
            Instruction::Jz { dest: 8 },
            Instruction::Jnz { dest: 7 },
        ];
        assert_eq!(prog.instructions(), correct);
    }

    #[test]
    fn test_parse_fails_malformed() {
        let bad = [
            (
                "[",
                BfParseErrorKind::UnclosedBracket,
                SourceLocation { line: 0, column: 0 },
            ),
            (
                "]",
                BfParseErrorKind::UnopenedBracket,
                SourceLocation { line: 0, column: 0 },
            ),
            (
                "][",
                BfParseErrorKind::UnopenedBracket,
                SourceLocation { line: 0, column: 0 },
            ),
            (
                "[[",
                BfParseErrorKind::UnclosedBracket,
                SourceLocation { line: 0, column: 1 },
            ),
            (
                "]]",
                BfParseErrorKind::UnopenedBracket,
                SourceLocation { line: 0, column: 0 },
            ),
            (
                "[[[[[[[[]]]]]]]]]",
                BfParseErrorKind::UnopenedBracket,
                SourceLocation {
                    line: 0,
                    column: 16,
                },
            ),
            (
                "[[[[[[[[[]]]]]]]]",
                BfParseErrorKind::UnclosedBracket,
                SourceLocation { line: 0, column: 0 },
            ),
        ];
        for (bf, err_kind, err_loc) in bad {
            let BfParseError {
                filename,
                location,
                kind,
            } = Program::try_new(Path::new("-"), bf).unwrap_err();
            assert_eq!(kind, err_kind);
            assert_eq!(location, err_loc);
            assert_eq!(filename, Path::new("-"));
        }
    }
}
