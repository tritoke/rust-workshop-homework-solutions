#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

//! Holds all of the types involved in brainfuck programs

/// the brainfuck program
mod program;
pub use program::{Program, SourceLocation};

/// the instructions of the brainfuck program
mod instruction;
pub use instruction::Instruction;
