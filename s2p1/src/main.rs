use std::error::Error;

mod student;
use student::read_students;

fn main() -> Result<(), Box<dyn Error>> {
    let if_name = std::env::args().nth(1).ok_or_else(|| {
        String::from("Insufficient number of arguments, please provide a filename.")
    })?;

    let students = read_students(if_name)?;
    dbg!(students);

    Ok(())
}
