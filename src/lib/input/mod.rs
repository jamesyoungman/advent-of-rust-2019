use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use crate::error::Fail;

use clap::{Arg, Command};

#[derive(Debug)]
pub enum InputError {
    NoInputFile,
    IoError {
        filename: Option<PathBuf>,
        err: std::io::Error,
    },
}

impl Display for InputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InputError::NoInputFile => write!(f, "no input file was specified"),
            InputError::IoError {
                filename: Some(name),
                err,
            } => write!(f, "read error on input file '{}': {}", name.display(), err),
            InputError::IoError {
                filename: None,
                err,
            } => write!(f, "read error on input: {}", err),
        }
    }
}

impl Error for InputError {}

impl From<InputError> for Fail {
    fn from(e: InputError) -> Fail {
        Fail(e.to_string())
    }
}

fn open_input_file(input_file_name: &Path) -> Result<BufReader<File>, InputError> {
    match OpenOptions::new().read(true).open(input_file_name) {
        Ok(file) => Ok(BufReader::new(file)),
        Err(e) => Err(InputError::IoError {
            filename: Some(input_file_name.to_path_buf()),
            err: e,
        }),
    }
}

pub fn read_file_as_string(input_file_name: &Path) -> Result<String, InputError> {
    let mut input: String = String::new();
    match open_input_file(input_file_name) {
        Ok(mut reader) => match reader.read_to_string(&mut input) {
            Ok(_) => Ok(input),
            Err(e) => Err(InputError::IoError {
                filename: Some(input_file_name.to_path_buf()),
                err: e,
            }),
        },
        Err(e) => Err(e),
    }
}

pub fn read_file_as_lines(input_file_name: &Path) -> Result<Vec<String>, InputError> {
    match open_input_file(input_file_name) {
        Ok(reader) => {
            let result: Result<Vec<String>, InputError> = reader
                .lines()
                .map(|item| match item {
                    Ok(line) => Ok(line),
                    Err(e) => Err(InputError::IoError {
                        filename: Some(input_file_name.to_path_buf()),
                        err: e,
                    }),
                })
                .collect();
            result
        }
        Err(e) => Err(e),
    }
}

pub fn run_with_input<ErrorType, InputErrorType, InputReader, F, T, InputType>(
    day: i8,
    input_reader: InputReader,
    runner: F,
) -> Result<T, ErrorType>
where
    InputReader: Fn(&Path) -> Result<InputType, InputErrorType>,
    ErrorType: From<InputError> + From<InputErrorType> + Error,
    F: Fn(InputType) -> Result<T, ErrorType>,
{
    let program_name: String = format!("Advent of code 2019 day {}", day);
    let about = format!("Solves Advent of Code 2019 puzzle for day {}", day);
    let cmd = Command::new(program_name.as_str())
        .author("James Youngman, james@youngman.org")
        .about(about.as_str())
        .arg(Arg::new("input_file").allow_invalid_utf8(true).index(1));
    let m = cmd.get_matches();
    match m.value_of_os("input_file") {
        Some(input_file_name) => {
            let path_name = PathBuf::from(input_file_name);
            match input_reader(&path_name) {
                Err(e) => Err(ErrorType::from(e)),
                Ok(the_input) => runner(the_input),
            }
        }
        None => Err(ErrorType::from(InputError::NoInputFile)),
    }
}
