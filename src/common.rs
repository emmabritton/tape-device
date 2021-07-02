use anyhow::{Context, Error, Result};
use crossterm::cursor::{MoveToColumn, MoveUp};
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use std::fs::File;
use std::io::stdout;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;

///Read file as bytes
pub fn read_bytes(path_str: &str) -> Result<Vec<u8>> {
    let path = PathBuf::from(path_str);

    if !path.exists() {
        return Err(Error::msg(format!("File does not exist: {}", path_str)));
    }

    let mut file = File::open(path).context(path_str.to_string())?;
    let mut buffer = vec![];
    match file.read_to_end(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(err) => Err(Error::from(err).context("read_bytes")),
    }
}

///Read file as lines
pub fn read_lines(path_str: &str) -> Result<Vec<String>> {
    let path = PathBuf::from(path_str);

    if !path.exists() {
        return Err(Error::msg(format!("File does not exist: {}", path_str)));
    }

    let file = File::open(path).context(path_str.to_string())?;
    Ok(BufReader::new(file)
        .lines()
        .map(|line| line.unwrap())
        .collect())
}

///Move the terminal up one line and to first column
#[allow(unused_must_use)]
pub fn reset_cursor() {
    execute!(
        stdout(),
        MoveUp(1),
        MoveToColumn(0),
        Clear(ClearType::CurrentLine)
    );
}
