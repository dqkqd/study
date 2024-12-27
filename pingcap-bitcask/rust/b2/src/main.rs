use std::{
    error::Error,
    fs::OpenOptions,
    io::{BufReader, Write},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Move {
    direction: Direction,
    step: u32,
}

impl Move {
    fn new(direction: Direction, step: u32) -> Move {
        Move { direction, step }
    }
}

fn json_exercise() -> Result<(), Box<dyn Error>> {
    let filename = "serialized.json";

    let a = Move::new(Direction::Up, 30);
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filename)?;
    serde_json::to_writer(file, &a)?;

    let file = OpenOptions::new().read(true).open(filename)?;
    let b: Move = serde_json::from_reader(file)?;

    assert_eq!(a, b);
    Ok(())
}

fn ron_exercise() -> Result<(), Box<dyn Error>> {
    let a = Move::new(Direction::Up, 30);

    let mut data = Vec::new();
    ron::ser::to_writer(&mut data, &a)?;

    let b: Move = ron::de::from_reader(data.as_slice())?;

    assert_eq!(a, b);
    Ok(())
}

fn bson_exercise() -> Result<(), Box<dyn Error>> {
    let filename = "serialized.bson";

    let n_moves = 1000;

    let moves: Vec<Move> = (0..n_moves)
        .map(|i| {
            let dir = match i % 4 {
                0 => Direction::Up,
                1 => Direction::Down,
                2 => Direction::Left,
                3 => Direction::Right,
                _ => unreachable!(),
            };
            Move {
                direction: dir,
                step: i,
            }
        })
        .collect();

    // file
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filename)?;
    for m in &moves {
        let v = bson::to_vec(m)?;
        let _ = file.write(&v)?;
    }
    let file = OpenOptions::new().read(true).open(filename)?;
    for original_move in &moves {
        let m: Move = bson::from_reader(&file)?;
        assert_eq!(&m, original_move)
    }

    // vec
    let mut values = Vec::new();
    for m in &moves {
        let v = bson::to_vec(m)?;
        values.extend(v.into_iter())
    }
    let mut reader = BufReader::new(values.as_slice());
    for original_move in &moves {
        let m: Move = bson::from_reader(&mut reader)?;
        assert_eq!(&m, original_move);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    json_exercise()?;
    ron_exercise()?;
    bson_exercise()?;
    Ok(())
}
