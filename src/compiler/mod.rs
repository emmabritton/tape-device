mod program;
mod strings;

use crate::common::read_lines;
use crate::compiler::program::compile;
use crate::compiler::strings::compile_strings;
use crate::constants::code::ALIGNMENT_PADDING;
use crate::constants::system::*;
use anyhow::{Error, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn start(path: Vec<&str>) -> Result<()> {
    let fda = path[0];
    let str = path.get(1);

    let path = PathBuf::from(fda);
    let output_file_name = if let Some(output_file_stem) = path.file_stem() {
        format!("{}.tape", output_file_stem.to_string_lossy())
    } else {
        eprintln!("Error parsing file name");
        String::from("output.tape")
    };
    let mut output_file_path = PathBuf::from(path.parent().unwrap());
    output_file_path.push(output_file_name);

    let (strings_data, string_bytes) = if let Some(str) = str {
        println!("Compiling {} as program and {} as data", fda, str);
        compile_strings(read_lines(str)?)?
    } else {
        println!("Compiling {} as program with no data", fda);
        (HashMap::new(), vec![])
    };
    println!("Writing to {}", output_file_path.to_string_lossy());

    let (name, version, program) = compile(read_lines(fda)?, strings_data)?;

    let mut data: Vec<u8> = program.iter().flatten().cloned().collect();
    let mut header = vec![TAPE_HEADER_1, TAPE_HEADER_2, PRG_VERSION, name.len() as u8];
    header.extend_from_slice(name.as_bytes());
    header.push(version.len() as u8);
    header.extend_from_slice(version.as_bytes());
    let bytes = (program.len() as u16).to_be_bytes();
    header.push(bytes[0]);
    header.push(bytes[1]);

    while header.len() as f64 % 3.0 != 0.0 {
        header.push(ALIGNMENT_PADDING);
    }

    for byte in header.iter().rev() {
        data.insert(0, *byte);
    }

    data.extend_from_slice(&string_bytes);

    match File::create(output_file_path) {
        Ok(mut file) => {
            file.write_all(&data)?;
            file.flush()?;
        }
        Err(err) => {
            eprintln!("Unable to create output file");
            return Err(Error::from(err));
        }
    }

    Ok(())
}
