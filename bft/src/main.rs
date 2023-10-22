#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

//! An interpreter for the brainfuck programming language

use std::{error::Error, io, process::ExitCode};

use bft_interp::{Machine, NewlineWrap, TapeKind};
use bft_types::Program;
use clap::Parser;

/// The CLI for the interpreter
mod cli;
use cli::Args;

fn main() -> ExitCode {
    let args = Args::parse();

    match run_bft(&args) {
        Err(e) => {
            eprintln!("Encountered error in {}: {e}", args.program.display());
            ExitCode::FAILURE
        }
        Ok(_) => ExitCode::SUCCESS,
    }
}

/// Run the brainfuck interpreter using the settings parsed from the CLI arguments
///
/// `args`: The CLI arguments
fn run_bft(args: &Args) -> Result<(), Box<dyn Error>> {
    let tape_kind = if args.extensible {
        TapeKind::Growable
    } else {
        TapeKind::FixedSize
    };
    let program = Program::from_file(&args.program)?;

    let stdin = io::stdin().lock();
    let stdout = NewlineWrap::new(io::stdout().lock());
    let mut machine = Machine::<u8>::new(args.cells, tape_kind, &program);
    machine.run(stdin, stdout)?;

    Ok(())
}
