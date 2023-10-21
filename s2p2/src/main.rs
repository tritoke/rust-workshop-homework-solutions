use std::error::Error;
mod brainfuck;

fn main() -> Result<(), Box<dyn Error>> {
    let if_name = std::env::args().nth(1).ok_or_else(|| {
        String::from("Insufficient number of arguments, please provide a filename.")
    })?;

    let prog = brainfuck::read_instructions(&if_name)?;
    for instr in prog {
        println!(
            "[{if_name}:{}:{}] {}",
            1 + instr.line(),
            1 + instr.column(),
            instr.opcode(),
        );
    }

    Ok(())
}
