use anyhow::{Context, Error, Result};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;

pub type Instruction = [u8; 3];

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
