use std::{error::Error, path::Path};

#[derive(Debug)]
pub enum Student {
    NameAndNumber { name: String, score: u8 },
    Name { name: String },
}

impl TryFrom<&str> for Student {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let student = match value.split_once(':') {
            Some((name, number)) => Self::NameAndNumber {
                name: name.to_owned(),
                score: number.parse()?,
            },
            None => Self::Name {
                name: value.to_owned(),
            },
        };

        Ok(student)
    }
}

pub fn read_students<P: AsRef<Path>>(filename: P) -> Result<Vec<Student>, Box<dyn Error>> {
    std::fs::read_to_string(filename)?
        .lines()
        .map(Student::try_from)
        .collect()
}
