use crate::constants::hardware::*;
use crate::language::parse_line;
use crate::language::parser::params::Param;
use anyhow::{Error, Result};
use std::collections::{HashMap, HashSet};

type LabelTable = HashMap<String, (Option<usize>, Vec<usize>)>;

fn process_constants(lines: Vec<String>) -> Result<(Vec<String>, Vec<String>)> {
    let (constants, ops): (Vec<&str>, Vec<&str>) = lines
        .iter()
        .map(|line| line.trim())
        .partition(|line| line.to_ascii_lowercase().starts_with("const"));

    let mut constant_names = vec![];
    let mut constant_lookup = HashMap::new();

    for constant in constants {
        let split = constant.split_ascii_whitespace().collect::<Vec<&str>>();
        if split.len() != 3 {
            return Err(Error::msg(format!(
                "Unable to parse constant: '{}'",
                constant
            )));
        }
        if is_invalid_constant_name(split[1]) {
            return Err(Error::msg(format!(
                "Invalid constant name for '{}', constant names can't be mnemonic or registers",
                constant
            )));
        }
        if constant_names.contains(&split[1]) {
            return Err(Error::msg(format!("Constant defined twice: {}", split[1])));
        }
        constant_lookup.insert(split[1], split[2]);
        constant_names.push(split[1]);
    }

    let mut processed_ops = vec![];

    for op in ops {
        let parts = op.split(':').collect::<Vec<&str>>();
        let (label, mut op) = match parts.len() {
            1 => (
                String::new(),
                parts[0].split_ascii_whitespace().collect::<Vec<&str>>(),
            ),
            2 => (
                format!("{}: ", parts[0]),
                parts[1].split_ascii_whitespace().collect::<Vec<&str>>(),
            ),
            _ => return Err(Error::msg(format!("Unable to parse '{}'", op))),
        };
        if op.len() > 1 && constant_names.contains(&op[1]) {
            op[1] = constant_lookup[op[1]]
        }
        if op.len() > 2 && constant_names.contains(&op[2]) {
            op[2] = constant_lookup[op[2]]
        }
        let op = op.join(" ");
        processed_ops.push(format!("{}{}", label, op));
    }

    let constant_names = constant_names
        .into_iter()
        .map(|line| line.to_string())
        .collect();

    Ok((constant_names, processed_ops))
}

pub(super) fn assemble(lines: Vec<String>, strings_data: HashMap<String, u16>) -> Result<Vec<u8>> {
    let mut program = Vec::with_capacity(lines.len() * 2);
    let mut used_string_keys = HashSet::with_capacity(strings_data.len());
    let mut labels: LabelTable = HashMap::new();

    let (constants, lines) = process_constants(lines)?;

    for line in lines.iter() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            let op = if line.contains(':') {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() != 2 {
                    return Err(Error::msg(format!(
                        "Unable to parse '{}', labels must have instructions",
                        line
                    )));
                }
                let lbl = parts[0];
                if let Some(err) = is_valid_label(lbl) {
                    return Err(Error::msg(format!(
                        " with label on line '{}': {}",
                        line, err
                    )));
                } else if labels.contains_key(lbl) {
                    labels.get_mut(lbl).unwrap().0 = Some(program.len());
                } else {
                    labels.insert(lbl.to_string(), (Some(program.len()), vec![]));
                }

                parts[1]
            } else {
                line
            };
            let (opcode, params) = parse_line(op)?;

            match decode(
                opcode,
                params,
                program.len(),
                &strings_data,
                &mut used_string_keys,
                &mut labels,
            ) {
                Ok(mut instr) => {
                    while !instr.is_empty() {
                        program.push(instr.remove(0))
                    }
                }
                Err(err) => return Err(Error::msg(format!("on line '{}': {:?}", line, err))),
            }
        }
        if program.len() >= RAM_SIZE as usize {
            return Err(Error::msg(format!(
                "Too many instructions, max is {:4X} bytes",
                RAM_SIZE
            )));
        }
    }

    for lbl in labels.keys() {
        if constants.contains(lbl) {
            return Err(Error::msg(format!(
                "Label and constant share name: {}",
                lbl
            )));
        }
        if let Some(target) = labels[lbl].0 {
            for caller in &labels[lbl].1 {
                let addr = (target as u16).to_be_bytes();
                program[*caller + 1] = addr[0];
                program[*caller + 2] = addr[1];
            }
        } else {
            return Err(Error::msg(format!("Label '{}' is never set", lbl)));
        }
    }

    let mut any_found = false;
    for key in strings_data.keys() {
        if !used_string_keys.contains(key) {
            any_found = true;
            println!("Unused string: {}", key);
        }
    }
    if any_found {
        println!(); //just for formatting
    }

    Ok(program)
}

fn decode(
    opcode: u8,
    params: Vec<Param>,
    pc: usize,
    strings_data: &HashMap<String, u16>,
    used_string_keys: &mut HashSet<String>,
    labels: &mut LabelTable,
) -> Result<Vec<u8>> {
    let mut result = vec![opcode];
    for param in params {
        match param {
            Param::Empty => {}
            Param::Number(num) => result.push(num),
            Param::DataReg(reg) => result.push(reg),
            Param::AddrReg(reg) => result.push(reg),
            Param::Addr(addr) => {
                let bytes = addr.to_be_bytes();
                result.push(bytes[0]);
                result.push(bytes[1]);
            }
            Param::Label(lbl) => {
                labels.entry(lbl).or_insert((None, vec![])).1.push(pc);
                result.push(0);
                result.push(0);
            }
            Param::StrKey(key) => {
                if strings_data.contains_key(&key) {
                    let bytes = strings_data.get(&key).unwrap().to_be_bytes();
                    result.push(bytes[0]);
                    result.push(bytes[1]);
                    used_string_keys.insert(key);
                } else {
                    return Err(Error::msg(format!("Undefined string key: {}", key)));
                }
            }
        }
    }
    Ok(result)
}

fn is_valid_label(label: &str) -> Option<String> {
    if label.is_empty() {
        return Some(String::from("Must not be empty"));
    }
    if label
        .chars()
        .filter(|chr| !(chr.is_alphanumeric() || chr == &'_'))
        .count()
        > 0
        && label
            .chars()
            .next()
            .map(|chr| chr.is_alphabetic())
            .is_some()
    {
        return Some(format!(
            "Invalid label: {} (must be [a-zA-Z][a-zA-Z0-9_]*)",
            label
        ));
    }
    if matches!(label, "A0" | "A1") {
        return Some(format!(
            "Invalid label: {} (can't use address regs as labels)",
            label
        ));
    }
    None
}

fn is_invalid_constant_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "d0" | "d1"
            | "d2"
            | "d3"
            | "a0"
            | "a1"
            | "acc"
            | "const"
            | "inc"
            | "dec"
            | "add"
            | "sub"
            | "cmp"
            | "over"
            | "jmp"
            | "je"
            | "jne"
            | "jl"
            | "jg"
            | "cpy"
            | "prtd"
            | "prt"
            | "push"
            | "pop"
            | "prtc"
            | "call"
            | "ret"
            | "halt"
            | "nop"
            | "lda0"
            | "lda1"
            | "cpya0"
            | "cpya1"
            | "nover"
            | "fopen"
            | "filer"
            | "filew"
            | "fseek"
            | "fskip"
            | "memr"
            | "memw"
            | "swpar"
            | "cmpar"
            | "prtln"
            | "arg"
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::code::*;

    #[test]
    fn test_basic_file() {
        let input = vec![
            "INC D0",
            "PRT d0",
            "sample: CPY ACC D0",
            "inC D0",
            "PRtC ACC",
            "jmP sample",
        ]
        .iter()
        .map(|line| line.to_string())
        .collect();

        let result = assemble(input, HashMap::new());
        assert!(result.is_ok());

        assert_eq!(
            result.unwrap(),
            [
                INC_REG,
                REG_D0,
                PRT_REG,
                REG_D0,
                CPY_REG_REG,
                REG_ACC,
                REG_D0,
                INC_REG,
                REG_D0,
                PRTC_REG,
                REG_ACC,
                JMP_ADDR,
                0,
                4
            ]
        );
    }

    #[test]
    fn test_all_ops() {
        let input = vec![
            "start: CPY D1 0",
            "CPY D2 ACC",
            "ADD D3 0",
            "ADD D1 D2",
            "SUB ACC x10",
            "SUB ACC D0",
            "inc d0",
            "inc a0",
            "dec d1",
            "dec a1",
            "CMP D1 xF",
            "cmp d3 d3",
            "jmp start",
            "jmp @xfFf",
            "jmp a0",
            "je start",
            "je @0",
            "je a0",
            "jne start",
            "jne @0",
            "jne a0",
            "jg start",
            "jg @0",
            "jg a0",
            "jl start",
            "jl @0",
            "jl a0",
            "over start",
            "over @0",
            "over a0",
            "nover start",
            "nover @0",
            "nover a0",
            "memr a0",
            "memr @xa2",
            "memw a1",
            "memw @911",
            "fopen",
            "fseek",
            "fskip d0",
            "fskip 11",
            "filew @1",
            "filew @xF",
            "filew a0",
            "filer @1500",
            "filer @x0",
            "filer a1",
            "cmpar",
            "swpar",
            "lda0 d0 d1",
            "lda1 d2 d3",
            "cpya0 d2 d3",
            "cpya1 d2 d3",
            "cpya0 @334",
            "cpya1 @334",
            "prtc 0",
            "prtc acc",
            "prt 0",
            "prt acc",
            "prtd str",
            "prtln",
            "call a0",
            "call start",
            "push a0",
            "push d3",
            "push 99",
            "push xFF",
            "pop d3",
            "ret",
            "nop",
            "halt",
            "arg d0 d1",
            "arg d2 3",
        ]
        .iter()
        .map(|line| line.to_string())
        .collect();

        let mut strings = HashMap::new();
        strings.insert(String::from("str"), 0_u16);

        let result = assemble(input, strings);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [
                CPY_REG_VAL,
                REG_D1,
                0,
                CPY_REG_REG,
                REG_D2,
                REG_ACC,
                ADD_REG_VAL,
                REG_D3,
                0,
                ADD_REG_REG,
                REG_D1,
                REG_D2,
                SUB_REG_VAL,
                REG_ACC,
                16,
                SUB_REG_REG,
                REG_ACC,
                REG_D0,
                INC_REG,
                REG_D0,
                INC_REG,
                REG_A0,
                DEC_REG,
                REG_D1,
                DEC_REG,
                REG_A1,
                CMP_REG_VAL,
                REG_D1,
                15,
                CMP_REG_REG,
                REG_D3,
                REG_D3,
                JMP_ADDR,
                0,
                0,
                JMP_ADDR,
                15,
                255,
                JMP_AREG,
                REG_A0,
                JE_ADDR,
                0,
                0,
                JE_ADDR,
                0,
                0,
                JE_AREG,
                REG_A0,
                JNE_ADDR,
                0,
                0,
                JNE_ADDR,
                0,
                0,
                JNE_AREG,
                REG_A0,
                JG_ADDR,
                0,
                0,
                JG_ADDR,
                0,
                0,
                JG_AREG,
                REG_A0,
                JL_ADDR,
                0,
                0,
                JL_ADDR,
                0,
                0,
                JL_AREG,
                REG_A0,
                OVER_ADDR,
                0,
                0,
                OVER_ADDR,
                0,
                0,
                OVER_AREG,
                REG_A0,
                NOVER_ADDR,
                0,
                0,
                NOVER_ADDR,
                0,
                0,
                NOVER_AREG,
                REG_A0,
                MEMR_AREG,
                REG_A0,
                MEMR_ADDR,
                0,
                162,
                MEMW_AREG,
                REG_A1,
                MEMW_ADDR,
                3,
                143,
                FOPEN,
                FSEEK,
                FSKIP_REG,
                REG_D0,
                FSKIP_VAL,
                11,
                FILEW_ADDR,
                0,
                1,
                FILEW_ADDR,
                0,
                15,
                FILEW_AREG,
                REG_A0,
                FILER_ADDR,
                5,
                220,
                FILER_ADDR,
                0,
                0,
                FILER_AREG,
                REG_A1,
                CMPAR,
                SWPAR,
                LDA0_REG_REG,
                REG_D0,
                REG_D1,
                LDA1_REG_REG,
                REG_D2,
                REG_D3,
                CPY_A0_REG_REG,
                REG_D2,
                REG_D3,
                CPY_A1_REG_REG,
                REG_D2,
                REG_D3,
                CPY_A0_ADDR,
                1,
                78,
                CPY_A1_ADDR,
                1,
                78,
                PRTC_VAL,
                0,
                PRTC_REG,
                REG_ACC,
                PRT_VAL,
                0,
                PRT_REG,
                REG_ACC,
                PRTD_STR,
                0,
                0,
                PRTLN,
                CALL_AREG,
                REG_A0,
                CALL_ADDR,
                0,
                0,
                PUSH_REG,
                REG_A0,
                PUSH_REG,
                REG_D3,
                PUSH_VAL,
                99,
                PUSH_VAL,
                255,
                POP_REG,
                REG_D3,
                RET,
                NOP,
                HALT,
                ARG_REG_REG,
                REG_D0,
                REG_D1,
                ARG_REG_VAL,
                REG_D2,
                3
            ]
        );
    }

    #[test]
    fn test_invalid_label() {
        let input = vec![String::from("JMP nowhere")];
        let result = assemble(input, HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_string() {
        let input = vec![String::from("PRTD nowhere")];
        let result = assemble(input, HashMap::new());
        assert!(result.is_err());
    }
}
