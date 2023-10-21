use std::error::Error;

use bft_interp::Machine;
use bft_types::Program;

fn main() -> Result<(), Box<dyn Error>> {
    let prog_path = std::env::args().nth(1).ok_or_else(|| {
        String::from("Insufficient number of arguments, please provide a filename.")
    })?;

    let machine = Machine::new_fixed_size(30_000);
    let program = Program::from_file(prog_path)?;

    machine.run(&program);

    Ok(())
}
