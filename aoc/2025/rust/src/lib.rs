use std::path::PathBuf;

pub struct InputFile {
    day: u8,
    file_type: InputFileType,
}

pub enum InputFileType {
    Example,
    Actual,
}

impl InputFile {
    pub fn actual(day: u8) -> InputFile {
        InputFile {
            day,
            file_type: InputFileType::Actual,
        }
    }
    pub fn example(day: u8) -> InputFile {
        InputFile {
            day,
            file_type: InputFileType::Example,
        }
    }

    pub fn path(&self) -> PathBuf {
        let input_dir = find_root_input_dir().join(format!("day{:02}", self.day));
        match self.file_type {
            InputFileType::Example => input_dir.join("example.txt"),
            InputFileType::Actual => input_dir.join("actual.txt"),
        }
    }
}

/// Walk parent directories upward from the current directory
/// until an `input` folder is found.
///
/// Returns the full path to the folder if found.
fn find_root_input_dir() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current dir must exist");

    loop {
        let candidate = dir.join("input");
        if candidate.is_dir() {
            return candidate;
        }
        // Stop when we reach the filesystem root
        if !dir.pop() {
            break;
        }
    }
    unreachable!("input must be present");
}
