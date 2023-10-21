use std::{collections::HashMap, error::Error};

mod student;
use student::{read_students, Student};

fn main() -> Result<(), Box<dyn Error>> {
    let if_name = std::env::args().nth(1).ok_or_else(|| {
        String::from("Insufficient number of arguments, please provide a filename.")
    })?;

    let students = read_students(if_name)?;
    let mut student_stats: HashMap<String, TestStatistics> = Default::default();

    for student in students {
        match student {
            Student::Name { name } => {
                student_stats.entry(name).or_default().missed_test();
            }
            Student::NameAndNumber { name, score } => {
                student_stats.entry(name).or_default().add_score(score);
            }
        }
    }

    dbg!(student_stats);

    Ok(())
}

#[derive(Default, Debug)]
struct TestStatistics {
    total: u32,
    no_scores: u32,
    no_missed: u32,
}

impl TestStatistics {
    fn add_score(&mut self, score: u8) {
        self.total += score as u32;
        self.no_scores += 1;
    }

    fn missed_test(&mut self) {
        self.no_missed += 1;
    }
}
