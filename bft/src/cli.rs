use bft_interp::DEFAULT_TAPE_SIZE;
use clap::Parser;
use clap_num::number_range;
use std::path::PathBuf;

/// CLI Arguments for the interpreter
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The path to the brainfuck program to run
    pub program: PathBuf,

    /// Should the interpreter's tape automatically extend?
    #[arg(short, long)]
    pub extensible: bool,

    /// The number of cells to allocate for the interpreter's tape
    #[arg(short, long, default_value_t = DEFAULT_TAPE_SIZE, value_parser = forbid_zero)]
    pub cells: usize,
}

/// Value parser to prevent forbid a value from being zero
fn forbid_zero(s: &str) -> Result<usize, String> {
    number_range(s, 1, usize::MAX)
}
