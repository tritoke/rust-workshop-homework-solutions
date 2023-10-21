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
const BF_ALPHABET: [u8; 8] = [b'>', b'<', b'+', b'-', b'.', b',', b'[', b']'];

impl Program {
    /// Construct a new brainfuck program from a filename and it's contents
    ///
    /// `filename`: the file the program was loaded from
    /// `file_contents`: the contents of `filename`
    ///
    /// ```ignore
    /// # use bft_types::Program;
    /// let contents = include_bytes!("../../programs/example.bf");
    /// let program = Program::new("../../programs/example.bf", contents);
    /// ```
    fn new(filename: &Path, mut file_contents: Vec<u8>) -> Result<Self, String> {
        // first filter out comment characters
        file_contents.retain(|b| BF_ALPHABET.contains(b));

        // track of all the jump destinations
        let mut jumps = BTreeMap::new();
        let mut jump_stack = vec![];

        for (i, op) in file_contents.iter().copied().enumerate() {
            if op == b'[' {
                jump_stack.push(i);
            } else if op == b']' {
                let jump_src = jump_stack
                    .pop()
                    .ok_or_else(|| String::from("Unbalanced brackets"))?;

                // insert both the forward and backward jumps
                jumps.insert(jump_src, i);
                jumps.insert(i, jump_src);
            }
        }

        if !jump_stack.is_empty() {
            return Err(String::from("Unbalanced brackets"));
        }

        // construct the instructions
        let instrs = file_contents
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
        let contents = std::fs::read(path)?;
        let filename = path.file_name().ok_or_else(|| {
            format!("Failed to load brainfuck program from {path:?} path doesn't point to a file.")
        })?;
        Ok(Self::new(Path::new(filename), contents)?)
    }

    /// the name of the file this program was loaded from
    ///
    /// ```
    /// # use bft_types::Program;
    /// let program = Program::from_file("../programs/example.bf").unwrap();
    /// assert_eq!(program.filename().to_str(), Some("example.bf"));
    /// ```
    pub fn filename(&self) -> &Path {
        &self.filename
    }

    /// the instructions contained by the program
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let alphabet = vec![b'>', b'<', b'+', b'-', b'.', b',', b'[', b']'];
        let prog = Program::new(Path::new("-"), alphabet).unwrap();
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
}
