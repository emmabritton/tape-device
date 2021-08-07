pub mod debug_model;
mod generator;
pub mod parser;
pub mod program_model;

use crate::assembler::generator::generate_byte_code;
use crate::assembler::parser::generate_program_model;
use crate::common::{read_lines, reset_cursor};
use crate::constants::code::{DIVDERS, KEYWORDS, MNEMONICS, REGISTERS};
use anyhow::{Error, Result};
use lazy_static::lazy_static;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn start(basm: &str, build_debug: bool, debug: bool) -> Result<()> {
    let path = PathBuf::from(basm);

    let (output_file_name, build_file_name, debug_file_name) =
        if let Some(output_file_stem) = path.file_stem() {
            (
                format!("{}.tape", output_file_stem.to_string_lossy()),
                format!("{}.build.json", output_file_stem.to_string_lossy()),
                format!("{}.debug", output_file_stem.to_string_lossy()),
            )
        } else {
            eprintln!("Error parsing file name");
            (
                String::from("output.tape"),
                String::from("output.build.json"),
                String::from("output.debug"),
            )
        };
    let mut output_file_path = PathBuf::from(path.parent().unwrap());
    output_file_path.push(output_file_name);

    let build_file = match build_debug {
        true => Some(build_file_name),
        false => None,
    };

    let debug_file = match debug {
        true => Some(debug_file_name),
        false => None,
    };

    let bytes = assemble(read_lines(basm)?, build_file, debug_file)?;

    let path = output_file_path.to_string_lossy().to_string();
    match File::create(output_file_path) {
        Ok(mut file) => {
            println!("Writing to {}", path);
            file.write_all(&bytes)?;
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

fn assemble(
    input: Vec<String>,
    build_file: Option<String>,
    debug_file: Option<String>,
) -> Result<Vec<u8>> {
    let program_model = generate_program_model(input)?;
    if let Some(path) = build_file {
        println!("Writing intermediate/interpretation stage to {}", path);
        std::fs::write(path, serde_json::to_string(&program_model)?)?;
    }
    program_model.validate()?;
    let (bytes, debug) = generate_byte_code(program_model)?;
    if let Some(path) = debug_file {
        println!("Writing debug data to {}", path);
        std::fs::write(path, serde_json::to_string(&debug)?)?;
    }

    Ok(bytes)
}

lazy_static! {
    static ref KEY_NAME_ERROR: String = format!("Key names must not include any register, keyword, section divider or mnemonic.\nThese include:\n{}\n{}\n{}\n{}",
        MNEMONICS.join(" "),KEYWORDS.join(" "),REGISTERS.join(" "),DIVDERS.join(" ")
        );
}

const FORMAT_ERROR: &str = r#"Invalid BASM file, expected format:
<Program Name>
<Program Version>
[.strings
<strings>]
[.data
<datas>]
.ops
<program>

Program name must be between 1 and 20 ASCII characters and be the first line
Program version must between 1 and 10 ASCII characters and be the second line

Blank lines are ok from the third line onwards
Case matters for section dividers (.strings, .data and .ops)

Strings and data take this format:
<key>=<value>
e.g.
greeting=Hello World!
numbers=[[10,20],[xF,x10]]

See language document for ops
"#;

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::code::{
        ADD_REG_REG, ARG_REG_VAL, CALL_ADDR, CMP_REG_REG, CPY_REG_AREG, CPY_REG_VAL, HALT, JE_ADDR,
        LD_AREG_DATA_VAL_VAL, PRTC_VAL, PRTLN, PRTS_STR, PRT_REG, PUSH_REG, RET,
    };
    use crate::constants::hardware::{REG_A0, REG_ACC, REG_D0, REG_D1, REG_D2};
    use crate::constants::system::*;

    #[test]
    #[rustfmt::skip]
    fn test_simple_program() {
        let program = [
            "Test Prog",
            "1.0",
            ".ops",
            "CPY D0 10",
            "CPY D2 xF",
            "ADD D0 D2",
        ].iter().map(|str| str.to_string()).collect();
        let bytes = assemble(program, None, None).unwrap();
        
        assert_eq!(bytes,
           vec![
            TAPE_HEADER_1, TAPE_HEADER_2, PRG_VERSION,
            9, 84, 101, 115, 116, 32, 80, 114, 111, 103,
            3, 49, 46, 48,
            0, 9,
            CPY_REG_VAL, REG_D0, 10,
            CPY_REG_VAL, REG_D2, 15,
            ADD_REG_REG, REG_D0, REG_D2,
            0, 0
        ]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_full_program() {
        let program = "Math Test\n1\n.strings\npass=PASS\nfail=FAIL\n.data\nvalues=[[1,2]]\n.strings\nplus=+\n.ops\nld a0 values 1 0\ncpy d0 a0\nprt d0\nprts plus\nprt d0\nprtc '='\nadd d0 d0\npush acc\nld a0 values 1 1\ncpy d0 a0\npush d0\nprt d0\ncall assert_eq\nhalt\nassert_eq:\narg d0 1\narg d1 2\ncmp d0 d1\nprtc ' '\nje assert_pass\nprts fail\nje done\nassert_pass:prts pass\ndone: \nprtln\nret\n"
            .lines()
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();
        
        let bytes  = assemble(program, None, None).unwrap();
        
        assert_eq!(bytes, vec![
            TAPE_HEADER_1, TAPE_HEADER_2, PRG_VERSION,
            9, 77, 97, 116, 104, 32, 84, 101, 115, 116,
            1, 49,
            0, 65,
            LD_AREG_DATA_VAL_VAL, REG_A0, 0, 0, 1, 0,
            CPY_REG_AREG, REG_D0, REG_A0,
            PRT_REG, REG_D0,
            PRTS_STR, 0, 10,
            PRT_REG, REG_D0,
            PRTC_VAL, 61,
            ADD_REG_REG, REG_D0, REG_D0,
            PUSH_REG, REG_ACC,
            LD_AREG_DATA_VAL_VAL, REG_A0, 0, 0, 1, 1,
            CPY_REG_AREG, REG_D0, REG_A0,
            PUSH_REG, REG_D0,
            PRT_REG, REG_D0,
            CALL_ADDR, 0, 40,
            HALT,
            ARG_REG_VAL, REG_D0, 1,
            ARG_REG_VAL, REG_D1, 2,
            CMP_REG_REG, REG_D0, REG_D1,
            PRTC_VAL, 32,
            JE_ADDR, 0, 60,
            PRTS_STR, 0, 0,
            JE_ADDR, 0, 63,
            PRTS_STR, 0, 5,
            PRTLN,
            RET,
            0, 12,
            4, 70, 65, 73, 76,
            4, 80, 65, 83, 83,
            1, 43,
            1, 2, 1, 2
        ]);
    }
}
