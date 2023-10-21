use std::error::Error;

const BF_ALPHABET: [char; 8] = ['>', '<', '+', '-', '.', ',', '[', ']'];

fn main() -> Result<(), Box<dyn Error>> {
    let if_name = std::env::args().nth(1)
        .ok_or_else(|| String::from("Insufficient number of arguments, please provide a filename."))?;

    let contents = std::fs::read_to_string(if_name)?;
    let prog: String = contents.chars().filter(|c| BF_ALPHABET.contains(c)).collect();

    println!("{}", prog);

    Ok(())
}
