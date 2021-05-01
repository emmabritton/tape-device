mod data;
mod program;

use crate::assembler::data::compile_strings;
use crate::assembler::program::assemble;
use crate::common::{clean_up_lines, read_lines, reset_cursor};
use crate::constants::system::*;
use anyhow::{Error, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub(super) const FORMAT_ERROR: &str = r#"Invalid TASM file, expected format:
<Program Name>
<Program Version>
.strings <optional>
<strings, optional>
.ops
<program>

Program name must be between 1 and 20 ASCII characters and be the first line
Program version must between 1 and 10 ASCII characters and be the second line

Blank lines are ok from the third line onwards
Case matters for section dividers (.strings and .ops)

Strings take this format:
<key>=<value>
e.g.
greeting=Hello World!

See language document for ops
"#;

pub fn start(tasm: &str, keep_whitespace: bool) -> Result<()> {
    let path = PathBuf::from(tasm);
    let output_file_name = if let Some(output_file_stem) = path.file_stem() {
        format!("{}.tape", output_file_stem.to_string_lossy())
    } else {
        eprintln!("Error parsing file name");
        String::from("output.tape")
    };
    let mut output_file_path = PathBuf::from(path.parent().unwrap());
    output_file_path.push(output_file_name);
    let mut lines = clean_up_lines(read_lines(tasm)?);

    if lines.len() < 4 {
        return Err(Error::msg(FORMAT_ERROR));
    }

    let name = lines.remove(0).trim().to_string();
    let version = lines.remove(0).trim().to_string();

    println!("Compiling strings");

    let (strings_data, string_bytes) = if lines[0].trim() == ".strings" {
        lines.remove(0);
        compile_strings(&mut lines, keep_whitespace)?
    } else {
        (HashMap::new(), vec![])
    };

    reset_cursor();
    println!("Compiling program");

    if lines.is_empty() {
        return Err(Error::msg(format!(
            "Program must have at least one instruction\n\n{}",
            FORMAT_ERROR
        )));
    }

    let mut program = assemble(lines, strings_data)?;

    reset_cursor();
    println!("Generating binary");

    let mut header = vec![TAPE_HEADER_1, TAPE_HEADER_2, PRG_VERSION, name.len() as u8];
    header.extend_from_slice(name.as_bytes());
    header.push(version.len() as u8);
    header.extend_from_slice(version.as_bytes());
    let bytes = (program.len() as u16).to_be_bytes();
    header.push(bytes[0]);
    header.push(bytes[1]);

    for byte in header.iter().rev() {
        program.insert(0, *byte);
    }

    program.extend_from_slice(&string_bytes);

    let path = output_file_path.to_string_lossy().to_string();
    match File::create(output_file_path) {
        Ok(mut file) => {
            reset_cursor();
            println!("Writing to {}", path);
            file.write_all(&program)?;
            file.flush()?;
            reset_cursor();
            println!("Compiled and written to {}", path);
        }
        Err(err) => {
            eprintln!("Unable to create output file");
            return Err(Error::from(err));
        }
    }

    Ok(())
}
