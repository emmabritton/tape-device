use crate::common::Instruction;
use crate::constants::code::*;
use crate::constants::hardware::*;
use crate::constants::system::MAX_PRG_SIZE;
use anyhow::{Error, Result};
use std::collections::{HashMap, HashSet};

type LabelTable = HashMap<String, (Option<usize>, Vec<usize>)>;

pub(super) fn compile(
    lines: Vec<String>,
    strings_data: HashMap<String, u16>,
) -> Result<(String, String, Vec<Instruction>)> {
    let mut program = Vec::with_capacity(lines.len());
    let mut used_string_keys = HashSet::with_capacity(strings_data.len());
    let mut labels: LabelTable = HashMap::new();

    if lines.len() < 3 {
        return Err(Error::msg(
            "Program must name, version and at least instruction",
        ));
    }

    let name = lines[0].trim();
    let ver = lines[1].trim();

    if name.is_empty() || name.len() > 20 {
        return Err(Error::msg("Program name must be between 1 and 20 chars"));
    }
    if ver.is_empty() || ver.len() > 10 {
        return Err(Error::msg("Program version must be between 1 and 10 chars"));
    }

    for line in lines.iter().skip(2) {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            let op = if line.contains(':') {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() != 2 {
                    return Err(Error::msg(format!("Unable to parse {}", line)));
                }
                let lbl = parts[0];
                if labels.contains_key(lbl) {
                    labels.get_mut(lbl).unwrap().0 = Some(program.len());
                } else {
                    if lbl
                        .chars()
                        .filter(|chr| !(chr.is_alphanumeric() || chr == &'_'))
                        .count()
                        > 0
                    {
                        return Err(Error::msg(format!("Invalid label: {}", lbl)));
                    }
                    labels.insert(lbl.to_string(), (Some(program.len()), vec![]));
                }

                parts[1]
            } else {
                line
            };
            match decode(
                op,
                program.len(),
                &strings_data,
                &mut used_string_keys,
                &mut labels,
            ) {
                Ok(instr) => program.push(instr),
                Err(err) => {
                    return Err(Error::msg(format!("at line {}: {:?}", program.len(), err)))
                }
            }
        } else {
            program.push([OP_NOP, 0, 0]);
        }
        if program.len() >= MAX_PRG_SIZE as usize {
            return Err(Error::msg(format!(
                "Too many instructions at line {}, max {}",
                program.len(),
                MAX_PRG_SIZE
            )));
        }
    }

    for lbl in labels.keys() {
        if let Some(target) = labels[lbl].0 {
            for caller in &labels[lbl].1 {
                let addr = (target as u16).to_be_bytes();
                program[*caller] = [program[*caller][0], addr[0], addr[1]];
            }
        } else {
            return Err(Error::msg(format!("Label '{}' is never set", lbl)));
        }
    }

    for key in strings_data.keys() {
        if !used_string_keys.contains(key) {
            println!("Unused string: {}", key);
        }
    }

    program.shrink_to_fit();
    Ok((name.to_string(), ver.to_string(), program))
}

fn decode(
    line: &str,
    pc: usize,
    strings_data: &HashMap<String, u16>,
    used_string_keys: &mut HashSet<String>,
    labels: &mut LabelTable,
) -> Result<Instruction> {
    let parts = line.split_whitespace().into_iter().collect::<Vec<&str>>();
    return match parts[0].to_ascii_lowercase().as_str() {
        "add" => {
            validate_len("ADD", parts.len(), 3)?;
            if let Ok(reg) = decode_reg(parts[2], 0) {
                Ok([OP_ADD_REG_REG, decode_reg(parts[1], 1)?, reg])
            } else {
                Ok([
                    OP_ADD_REG_VAL,
                    decode_reg(parts[1], 1)?,
                    parse_num(parts[2], 2)?,
                ])
            }
        }
        "sub" => {
            validate_len("SUB", parts.len(), 3)?;
            if let Ok(reg) = decode_reg(parts[2], 0) {
                Ok([OP_SUB_REG_REG, decode_reg(parts[1], 1)?, reg])
            } else {
                Ok([
                    OP_SUB_REG_VAL,
                    decode_reg(parts[1], 1)?,
                    parse_num(parts[2], 2)?,
                ])
            }
        }
        "cpy" => {
            validate_len("CPY", parts.len(), 3)?;
            if let Ok(reg) = decode_reg(parts[2], 0) {
                Ok([OP_COPY_REG_REG, decode_reg(parts[1], 1)?, reg])
            } else {
                Ok([
                    OP_COPY_REG_VAL,
                    decode_reg(parts[1], 1)?,
                    parse_num(parts[2], 2)?,
                ])
            }
        }
        "memr" => {
            validate_len("MEMr", parts.len(), 2)?;
            match decode_addr(parts[1], 1) {
                Ok(mem) => {
                    let bytes = mem.to_be_bytes();
                    Ok([OP_MEM_READ, bytes[0], bytes[1]])
                }
                Err(err) => Err(err),
            }
        }
        "memw" => {
            validate_len("MEMW", parts.len(), 2)?;
            match decode_addr(parts[1], 1) {
                Ok(mem) => {
                    let bytes = mem.to_be_bytes();
                    Ok([OP_MEM_WRITE, bytes[0], bytes[1]])
                }
                Err(err) => Err(err),
            }
        }
        "cmp" => {
            validate_len("DEC", parts.len(), 3)?;
            if let Ok(reg) = decode_reg(parts[2], 0) {
                Ok([OP_CMP_REG_REG, decode_reg(parts[1], 1)?, reg])
            } else {
                Ok([
                    OP_CMP_REG_VAL,
                    decode_reg(parts[1], 1)?,
                    parse_num(parts[2], 2)?,
                ])
            }
        }
        "jmp" | "jl" | "je" | "jne" | "jg" | "over" | "nover" => {
            Ok(decode_jump(&parts, pc, labels)?)
        }
        "inc" => {
            validate_len("INC", parts.len(), 2)?;
            match decode_reg(parts[1], 1) {
                Ok(reg) => Ok([OP_INC, reg, 0]),
                Err(err) => Err(err),
            }
        }
        "dec" => {
            validate_len("DEC", parts.len(), 2)?;
            match decode_reg(parts[1], 1) {
                Ok(reg) => Ok([OP_DEC, reg, 0]),
                Err(err) => Err(err),
            }
        }
        "prt" => {
            validate_len("PRT", parts.len(), 2)?;
            if let Ok(reg) = decode_reg(parts[1], 0) {
                Ok([OP_PRINT_REG, reg, 0])
            } else if let Ok(mem) = decode_addr(parts[1], 0) {
                let bytes = mem.to_be_bytes();
                Ok([OP_PRINT_MEM, bytes[0], bytes[1]])
            } else {
                Ok([OP_PRINT_VAL, parse_num(parts[1], 1)?, 0])
            }
        }
        "prtd" => {
            validate_len("PRTD", parts.len(), 2)?;
            if strings_data.contains_key(parts[1]) {
                used_string_keys.insert(parts[1].to_string());
                let bytes = strings_data[parts[1]].to_be_bytes();
                Ok([OP_PRINT_DAT, bytes[0], bytes[1]])
            } else {
                Err(Error::msg(format!("Unknown string key: {}", parts[1])))
            }
        }
        "prtln" => {
            validate_len("PRTLN", parts.len(), 1)?;
            Ok([OP_PRINT_LN, 0, 0])
        }
        "fopen" => {
            validate_len("FOPEN", parts.len(), 1)?;
            Ok([OP_OPEN_FILE, 0, 0])
        }
        "fwrite" => {
            validate_len("FWRITE", parts.len(), 2)?;
            match decode_addr(parts[1], 1) {
                Ok(mem) => {
                    let bytes = mem.to_be_bytes();
                    Ok([OP_WRITE_FILE, bytes[0], bytes[1]])
                }
                Err(err) => Err(err),
            }
        }
        "fseek" => {
            validate_len("FWRITE", parts.len(), 1)?;
            Ok([OP_SEEK_FILE, 0, 0])
        }
        "fread" => {
            validate_len("FREAD", parts.len(), 2)?;
            match decode_addr(parts[1], 1) {
                Ok(mem) => {
                    let bytes = mem.to_be_bytes();
                    Ok([OP_READ_FILE, bytes[0], bytes[1]])
                }
                Err(err) => Err(err),
            }
        }
        "fskip" => {
            validate_len("FSKIP", parts.len(), 2)?;
            Ok([OP_SKIP_FILE, decode_reg(parts[1], 1)?, 0])
        }
        "nop" => {
            validate_len("NOP", parts.len(), 1)?;
            Ok([OP_NOP, 0, 0])
        }
        "halt" => {
            validate_len("HALT", parts.len(), 1)?;
            Ok([OP_HALT, 0, 0])
        }
        _ => Err(Error::msg(format!("Unknown instruction: {}", parts[0]))),
    };
}

fn decode_jump(parts: &[&str], pc: usize, labels: &mut LabelTable) -> Result<Instruction> {
    let stmts = ["JMP", "JL", "JE", "JNE", "JG", "OVER", "NOVER"];
    let ops = [
        OP_JMP,
        OP_JL,
        OP_JE,
        OP_JNE,
        OP_JG,
        OP_OVERFLOW,
        OP_NOT_OVERFLOW,
    ];
    let opcode = parts[0].to_ascii_uppercase();
    let idx = stmts.iter().position(|&op| op == opcode).unwrap();
    validate_len(stmts[idx], parts.len(), 2)?;
    match decode_addr(parts[1], 1) {
        Ok(addr) => {
            let bytes = addr.to_be_bytes();
            Ok([ops[idx], bytes[0], bytes[1]])
        }
        Err(_) => {
            if labels.contains_key(parts[1]) {
                labels.get_mut(parts[1]).unwrap().1.push(pc);
            } else {
                labels.insert(parts[1].to_string(), (None, vec![pc]));
            }
            Ok([ops[idx], 0, 0])
        }
    }
}

fn validate_len(op: &'static str, actual: usize, required: usize) -> Result<()> {
    return if required != actual {
        Err(Error::msg(format!(
            "{} requires {} params",
            op,
            required - 1
        )))
    } else {
        Ok(())
    };
}

fn decode_addr(input: &str, param: usize) -> Result<u16> {
    return if input.starts_with('@') {
        let trimmed = input.chars().skip(1).collect::<String>();
        return if trimmed.starts_with('x') {
            let trimmed = trimmed.chars().skip(1).collect::<String>();
            match u16::from_str_radix(&trimmed, 16) {
                Ok(num) => Ok(num),
                Err(err) => Err(Error::msg(format!(
                    "Param {} must be an address @x0..@xFFFF, was {}: {}",
                    param, input, err
                ))),
            }
        } else {
            match trimmed.parse::<u16>() {
                Ok(num) => Ok(num),
                Err(err) => Err(Error::msg(format!(
                    "Param {} must be an address @0..@65535, was {}: {}",
                    param, input, err
                ))),
            }
        };
    } else {
        Err(Error::msg(format!(
            "Param {} must be an address, was {}",
            param, input
        )))
    };
}

fn decode_reg(input: &str, param: usize) -> Result<u8> {
    match input.to_ascii_lowercase().as_str() {
        "d0" => Ok(REG_D0),
        "d1" => Ok(REG_D1),
        "d2" => Ok(REG_D2),
        "d3" => Ok(REG_D3),
        "acc" => Ok(REG_ACC),
        _ => Err(Error::msg(format!(
            "Param {} must be a register, was {}",
            param, input
        ))),
    }
}

fn parse_num(input: &str, param: usize) -> Result<u8> {
    if input.starts_with('x') {
        let trimmed = input.chars().skip(1).collect::<String>();
        match u8::from_str_radix(&trimmed, 16) {
            Ok(num) => Ok(num),
            Err(err) => Err(Error::msg(format!(
                "Param {} must be an number x0..xFF, was {}: {}",
                param, input, err
            ))),
        }
    } else {
        match input.parse::<u8>() {
            Ok(num) => Ok(num),
            Err(err) => Err(Error::msg(format!(
                "Param {} must be a number 0..255, was {}: {}",
                param, input, err
            ))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_number_parsing() {
        assert_eq!(parse_num("10", 0).unwrap(), 10);
        assert_eq!(parse_num("0", 0).unwrap(), 0);
        assert_eq!(parse_num("100", 0).unwrap(), 100);
        assert_eq!(parse_num("255", 0).unwrap(), 255);
        assert!(parse_num("256", 0).is_err());
        assert!(parse_num("1000", 0).is_err());
        assert!(parse_num("-1", 0).is_err());
        assert_eq!(parse_num("xA", 0).unwrap(), 10);
        assert_eq!(parse_num("x0", 0).unwrap(), 0);
        assert_eq!(parse_num("x64", 0).unwrap(), 100);
        assert_eq!(parse_num("xFF", 0).unwrap(), 255);
        assert!(parse_num("x100", 0).is_err());
        assert!(parse_num("x3e8", 0).is_err());
        assert!(parse_num("xF001", 0).is_err());
    }

    #[test]
    fn test_reg_decoding() {
        assert_eq!(decode_reg("d0", 0).unwrap(), REG_D0);
        assert_eq!(decode_reg("d1", 0).unwrap(), REG_D1);
        assert_eq!(decode_reg("d2", 0).unwrap(), REG_D2);
        assert_eq!(decode_reg("d3", 0).unwrap(), REG_D3);
        assert_eq!(decode_reg("acc", 0).unwrap(), REG_ACC);
        assert!(decode_reg("d5", 0).is_err());
        assert!(decode_reg("", 0).is_err());
        assert!(decode_reg("dec", 0).is_err());
    }

    #[test]
    fn test_address_decode() {
        assert_eq!(decode_addr("@10", 0).unwrap(), 10);
        assert_eq!(decode_addr("@0", 0).unwrap(), 0);
        assert_eq!(decode_addr("@100", 0).unwrap(), 100);
        assert_eq!(decode_addr("@255", 0).unwrap(), 255);
        assert_eq!(decode_addr("@1000", 0).unwrap(), 1000);
        assert_eq!(decode_addr("@12000", 0).unwrap(), 12000);
        assert_eq!(decode_addr("@65535", 0).unwrap(), 65535);
        assert!(decode_addr("@65536", 0).is_err());
        assert!(decode_addr("@test", 0).is_err());
        assert_eq!(decode_addr("@255", 0).unwrap(), 255);
        assert_eq!(decode_addr("@xA", 0).unwrap(), 10);
        assert_eq!(decode_addr("@x0", 0).unwrap(), 0);
        assert_eq!(decode_addr("@x64", 0).unwrap(), 100);
        assert_eq!(decode_addr("@xFF", 0).unwrap(), 255);
        assert_eq!(decode_addr("@xFF00", 0).unwrap(), 65280);
        assert_eq!(decode_addr("@xFFFF", 0).unwrap(), 65535);
        assert!(decode_addr("@1x2", 0).is_err());
        assert!(decode_addr("@x2p", 0).is_err());
        assert!(decode_addr("@x", 0).is_err());
    }

    #[test]
    fn test_decode() {
        let mut map = HashMap::new();
        map.insert(String::from("test_str"), 10 as u16);
        let mut used_keys = HashSet::new();
        let pc = 0;
        let mut labels: LabelTable = HashMap::new();

        assert_eq!(
            decode("ADD D0 10", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_ADD_REG_VAL, REG_D0, 10]
        );
        assert_eq!(
            decode("ADD D0 ACC", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_ADD_REG_REG, REG_D0, REG_ACC]
        );
        assert!(decode("ADD D0", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("SUB D0 ACC", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_SUB_REG_REG, REG_D0, REG_ACC]
        );
        assert_eq!(
            decode("SUB D3 xEA", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_SUB_REG_VAL, REG_D3, 234]
        );
        assert!(decode("sub D0", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("mEMw @10", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_MEM_WRITE, 0, 10]
        );
        assert_eq!(
            decode("memw @xEA", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_MEM_WRITE, 0, 234]
        );
        assert!(decode("memW D0", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("MEMW 10", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("inc d2", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_INC, REG_D2, 0]
        );
        assert!(decode("inc 10", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("dec d1", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_DEC, REG_D1, 0]
        );

        assert_eq!(
            decode("cmp d1 d2", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_CMP_REG_REG, REG_D1, REG_D2]
        );
        assert_eq!(
            decode("cmp acc 18", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_CMP_REG_VAL, REG_ACC, 18]
        );
        assert!(decode("cmp @1 d2", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("cpy D2 D1", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_COPY_REG_REG, REG_D2, REG_D1]
        );
        assert_eq!(
            decode("cpy D1 6", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_COPY_REG_VAL, REG_D1, 6]
        );
        assert_eq!(
            decode("cpy D3 x64", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_COPY_REG_VAL, REG_D3, 100]
        );
        assert_eq!(
            decode("MEMR @4", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_MEM_READ, 0, 4]
        );
        assert_eq!(
            decode("memr @x34", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_MEM_READ, 0, 52]
        );
        assert!(decode("memr ACC @1", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("memr D0", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("memr 10", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("PRT 10", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_PRINT_VAL, 10, 0]
        );
        assert_eq!(
            decode("PRT xF", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_PRINT_VAL, 15, 0]
        );
        assert_eq!(
            decode("PRT D0", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_PRINT_REG, REG_D0, 0]
        );
        assert_eq!(
            decode("PRT @xFF", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_PRINT_MEM, 0, 255]
        );
        assert!(decode("PRT", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("PRTD test_str", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_PRINT_DAT, 0, 10]
        );
        assert!(decode("PRTD doesnt_exist", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("PRTD D0", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("PRTD 10", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("PRTD @10", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("PRTD", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("PRTLN", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_PRINT_LN, 0, 0]
        );
        assert!(decode("PRTLN 1", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("PRTLN D0", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("PRTLN @xA", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("FOPEN", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_OPEN_FILE, 0, 0]
        );
        assert!(decode("FOPEN 1", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("FOPEN D2", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("FOPEN @xA", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("FREAD @xF0", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_READ_FILE, 0, 240]
        );
        assert!(decode("FREAD 1", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("FREAD D2", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("FREAD", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("FSKIP ACC", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_SKIP_FILE, REG_ACC, 0]
        );
        assert!(decode("FSKIP 1", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("FSKIP", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("FSKIP @xA", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("NOP", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_NOP, 0, 0]
        );
        assert_eq!(
            decode("HALT", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_HALT, 0, 0]
        );
        assert!(decode("NOP 1", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("NOP @1", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("NOP lbl", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("NOP acc", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("halt 1", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("halt @1", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("halt lbl", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("halt acc", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("JMP tst", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_JMP, 0, 0]
        );
        assert_eq!(
            decode("JMP @55", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_JMP, 0, 55]
        );
        assert!(decode("jmp", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("je tst", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_JE, 0, 0]
        );
        assert_eq!(
            decode("je @3", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_JE, 0, 3]
        );
        assert!(decode("je", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("jl tst", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_JL, 0, 0]
        );
        assert!(decode("jl", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("jg tst", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_JG, 0, 0]
        );
        assert!(decode("jg", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("jne tst", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_JNE, 0, 0]
        );
        assert!(decode("jne", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("over tst", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_OVERFLOW, 0, 0]
        );
        assert!(decode("over", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("nover tst", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_NOT_OVERFLOW, 0, 0]
        );
        assert!(decode("nover", pc, &map, &mut used_keys, &mut labels).is_err());

        assert!(decode("invalid", pc, &map, &mut used_keys, &mut labels).is_err());
    }

    #[test]
    fn test_invalid_compile() {
        let script = vec![String::from("this is invalid")];
        let program = compile(script, HashMap::new());
        assert!(program.is_err());
    }

    #[test]
    fn test_compile() {
        let script = r"test prog
        1
        #program
        ADD D0 10
  goto: SUB D3 xEA
        CPY D2 D1
        JMP goto
        #last statement
        MEMR @4
        ";
        let (name, ver, program) = compile(
            script.lines().map(|str| str.to_string()).collect(),
            HashMap::new(),
        )
        .unwrap();
        assert_eq!(&name, "test prog");
        assert_eq!(&ver, "1");
        assert_eq!(
            program,
            vec![
                [OP_NOP, 0, 0],
                [OP_ADD_REG_VAL, REG_D0, 10],
                [OP_SUB_REG_VAL, REG_D3, 234],
                [OP_COPY_REG_REG, REG_D2, REG_D1],
                [OP_JMP, 0, 2],
                [OP_NOP, 0, 0],
                [OP_MEM_READ, 0, 4],
                [OP_NOP, 0, 0],
            ]
        );
    }
}
