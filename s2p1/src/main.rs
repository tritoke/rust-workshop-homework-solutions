use std::{collections::HashMap, error::Error, fmt, path::Path, str::FromStr};

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

    for (name, stats) in student_stats {
        println!("{name} took {stats}");
    }

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

impl fmt::Display for TestStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let TestStatistics {
            total,
            no_scores,
            no_missed,
        } = *self;
        let pluralise = |n: u32| if n == 1 { "test" } else { "tests" };
        write!(
            f,
            "{no_scores} {}, with a total score of {total}.  They missed {no_missed} {}.",
            pluralise(no_scores),
            pluralise(no_missed)
        )
    }
}

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
