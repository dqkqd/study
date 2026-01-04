use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use rust::InputFile;

#[derive(Debug)]
enum Rotation {
    Left(i64),
    Right(i64),
}

impl From<&str> for Rotation {
    fn from(value: &str) -> Self {
        let distance: i64 = value[1..].parse().unwrap();
        match value.chars().next() {
            Some('L') => Rotation::Left(distance),
            Some('R') => Rotation::Right(distance),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
struct Dial {
    position: i64,
    stop_at_zero: usize,
    seen_zero: usize,
}

impl Default for Dial {
    fn default() -> Self {
        Self {
            position: 50,
            stop_at_zero: 0,
            seen_zero: 0,
        }
    }
}

impl Dial {
    fn update(&mut self, rotation: &Rotation) {
        match rotation {
            Rotation::Right(distance) => {
                self.position += distance;
                self.seen_zero += (self.position / 100) as usize;
            }
            Rotation::Left(distance) => {
                if self.position == 0 {
                    self.position -= distance;
                    self.seen_zero += (distance / 100) as usize;
                } else {
                    self.position -= distance;
                    if self.position == 0 {
                        self.seen_zero += 1;
                    } else if self.position < 0 {
                        self.seen_zero += (self.position.abs() / 100) as usize + 1;
                    }
                }
            }
        }
        self.position %= 100;
        self.position += 100;
        self.position %= 100;

        if self.position == 0 {
            self.stop_at_zero += 1;
        }
    }
}

fn solve(path: PathBuf) -> Result<(), Box<dyn Error>> {
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let mut rotations = vec![];
    for line in reader.lines() {
        let line = line?;
        let rotation = Rotation::from(line.as_str());
        rotations.push(rotation);
    }
    let mut dial = Dial::default();
    for rotation in rotations {
        dial.update(&rotation);
    }
    println!("Part 1: {}", dial.stop_at_zero);
    println!("Part 2: {}", dial.seen_zero);

    Ok(())
}

fn main() {
    solve(InputFile::example(1).path()).unwrap();
    solve(InputFile::actual(1).path()).unwrap();
}
