#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

//! An interpreter for the brainfuck programming language

use std::error::Error;

use bft_interp::Machine;
use bft_types::Program;
use clap::Parser;

/// The CLI for the interpreter
mod cli;
use cli::Args;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let machine = if args.extensible {
        Machine::new(args.cells)
    } else {
        Machine::new_fixed_size(args.cells)
    };
    let program = Program::from_file(args.program)?;

    machine.run(&program);

    Ok(())
}
