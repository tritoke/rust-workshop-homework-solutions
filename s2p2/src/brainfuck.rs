use std::{error::Error, fmt, path::Path};

/// The brainfuck language commands
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Opcode {
    /// > Increment the data pointer by one (to point to the next cell to the right).
    Inc,

    /// < Decrement the data pointer by one (to point to the next cell to the left).
    Dec,

    /// + Increment the byte at the data pointer by one.
    Succ,

    /// - Decrement the byte at the data pointer by one.
    Pred,

    /// . Output the byte at the data pointer.
    Out,

    /// , Accept one byte of input, storing its value in the byte at the data pointer.
    In,

    /// [ If the byte at the data pointer is zero, then instead of moving the instruction pointer forward to the next command, jump it forward to the command after the matching ] command.
    Jz,

    /// ] If the byte at the data pointer is nonzero, then instead of moving the instruction pointer forward to the next command, jump it back to the command after the matching [ command.
    Jnz,
}

impl Opcode {
    pub fn from_char(c: char) -> Option<Self> {
        let instr = match c {
            '>' => Opcode::Inc,
            '<' => Opcode::Dec,
            '+' => Opcode::Succ,
            '-' => Opcode::Pred,
            '.' => Opcode::Out,
            ',' => Opcode::In,
            '[' => Opcode::Jz,
            ']' => Opcode::Jnz,
            _ => return None,
        };

        Some(instr)
    }
}

impl fmt::Display for Opcode {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::Inc  => write!(f, "Increment the data pointer by one"),
            Opcode::Dec  => write!(f, "Decrement the data pointer by one"),
            Opcode::Succ => write!(f, "Increment the byte at the data pointer by one"),  
            Opcode::Pred => write!(f, "Decrement the byte at the data pointer by one"),                                               
            Opcode::Out  => write!(f, "Output the byte at the data pointer"),                                               
            Opcode::In   => write!(f, "Accept one byte of input"),  
            Opcode::Jz   => write!(f, "Jump if zero"),
            Opcode::Jnz  => write!(f, "Jump if not zero"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    op: Opcode,
    line: usize,
    column: usize,
}

impl Instruction {
    pub fn opcode(&self) -> &Opcode {
        &self.op
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }
}

pub fn read_instructions<P: AsRef<Path>>(filename: P) -> Result<Vec<Instruction>, Box<dyn Error>> {
    let instrs = std::fs::read_to_string(filename)?
        .lines()
        .enumerate()
        .flat_map(|(line_no, line)| {
            line.chars().enumerate().filter_map(move |(column, c)| {
                Opcode::from_char(c).map(|op| Instruction {
                    op,
                    line: line_no,
                    column,
                })
            })
        })
        .collect();

    Ok(instrs)
}
