use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let if_name = std::env::args().nth(1)
        .ok_or_else(|| String::from("Insufficient number of arguments, please provide a filename."))?;

    let contents = std::fs::read_to_string(if_name)?;
    let parsed_nums: Vec<i32> = contents.lines().map(str::parse).collect::<Result<_, _>>()?;
    let sum: i32 = parsed_nums.into_iter().sum();

    println!("The numbers in the file had a sum of: {sum}");

    Ok(())
}
