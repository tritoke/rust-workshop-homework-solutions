#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

//! the brainfuck interpreter

mod machine;
pub use machine::{InterpretError, Machine, TapeKind, DEFAULT_TAPE_SIZE};
