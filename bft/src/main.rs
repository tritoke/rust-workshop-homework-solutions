use std::error::Error;

use bft_interp::Machine;

fn main() -> Result<(), Box<dyn Error>> {
    let _machine = Machine::new_fixed_size(30_000);

    Ok(())
}
