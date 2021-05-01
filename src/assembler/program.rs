use crate::constants::hardware::*;
use crate::language::parse_line;
use crate::language::parser::params::Param;
use anyhow::{Error, Result};
use std::collections::{HashMap, HashSet};

type LabelTable = HashMap<String, (Option<usize>, Vec<usize>)>;

pub(super) fn assemble(lines: Vec<String>, strings_data: HashMap<String, u16>) -> Result<Vec<u8>> {
    let mut program = Vec::with_capacity(lines.len() * 2);
    let mut used_string_keys = HashSet::with_capacity(strings_data.len());
    let mut labels: LabelTable = HashMap::new();

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
                } else {
                    if labels.contains_key(lbl) {
                        labels.get_mut(lbl).unwrap().0 = Some(program.len());
                    } else {
                        labels.insert(lbl.to_string(), (Some(program.len()), vec![]));
                    }
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
                if labels.contains_key(&lbl) {
                    labels.get_mut(&lbl).unwrap().1.push(pc);
                } else {
                    labels.insert(lbl, (None, vec![pc]));
                }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decode() {
        let mut map = HashMap::new();
        map.insert(String::from("test_str"), 10 as u16);
        let mut used_keys = HashSet::new();
        let pc = 0;
        let mut labels: LabelTable = HashMap::new();

        assert_eq!(
            decode("ADD D0 10", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [ADD_REG_VAL, REG_D0, 10]
        );
        assert_eq!(
            decode("ADD D0 ACC", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [ADD_REG_REG, REG_D0, REG_ACC]
        );
        assert!(decode("ADD D0", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("SUB D0 ACC", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [SUB_REG_REG, REG_D0, REG_ACC]
        );
        assert_eq!(
            decode("SUB D3 xEA", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [SUB_REG_VAL, REG_D3, 234]
        );
        assert!(decode("sub D0", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("addrh A0 D0", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_LOAD_ADDR_HIGH, REG_A0, REG_D0]
        );
        assert_eq!(
            decode("addrh A1 233", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_LOAD_ADDR_HIGH_VAL, REG_A1, 233]
        );
        assert_eq!(
            decode("addrl A1 ACC", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_LOAD_ADDR_LOW, REG_A1, REG_ACC]
        );
        assert_eq!(
            decode("addrl A0 2", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_LOAD_ADDR_LOW_VAL, REG_A0, 2]
        );
        assert!(decode("addrh", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("addrl", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("addrh 10", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("addrh d0", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("addrl a0", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("addrh a0 @a", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("addrl @a a0", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("addrh d1 a0", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("mEMw @10", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_MEM_WRITE, 0, 10]
        );
        assert_eq!(
            decode("memw @xEA", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_MEM_WRITE, 0, 234]
        );
        assert_eq!(
            decode("memw A0", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_MEM_WRITE_REG, REG_A0, 0]
        );
        assert!(decode("memW D0", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("MEMW 10", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("inc d2", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [INC, REG_D2, 0]
        );
        assert!(decode("inc 10", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("dec d1", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [DEC, REG_D1, 0]
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
            decode("MEMR A1", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_MEM_READ_REG, REG_A1, 0]
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
        assert_eq!(
            decode("PRT A0", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_PRINT_MEM_REG, REG_A0, 0]
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
        assert_eq!(
            decode("JMP A0", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_JMP_REG, REG_A0, 0]
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
            decode("push d0", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_PUSH_REG, REG_D0, 0]
        );
        assert_eq!(
            decode("push 25", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_PUSH_VAL, 25, 0]
        );
        assert!(decode("push", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("push a0", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("push @1", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("push test", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("pop d3", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_POP_REG, REG_D3, 0]
        );
        assert!(decode("pop ", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("pop 25", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("pop  a0", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("pop  @1", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("pop  test", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("call lbl", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_CALL_ADDR, 0, 0]
        );
        assert_eq!(
            decode("call @xABCD", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_CALL_ADDR, 0xAB, 0xCD]
        );
        assert_eq!(
            decode("call A0", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_CALL_REG, REG_A0, 0]
        );
        assert_eq!(
            decode("call 123", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_CALL_ADDR, 0, 0]
        );
        assert_eq!(
            decode("call acc", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_CALL_ADDR, 0, 0]
        );

        assert_eq!(
            decode("reT", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_RETURN, 0, 0]
        );
        assert!(decode("ret d0", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("ret 25", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("ret  a0", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("ret  @1", pc, &map, &mut used_keys, &mut labels).is_err());
        assert!(decode("ret  test", pc, &map, &mut used_keys, &mut labels).is_err());

        assert_eq!(
            decode("over tst", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_OVERFLOW, 0, 0]
        );
        assert_eq!(
            decode("over A1", pc, &map, &mut used_keys, &mut labels).unwrap(),
            [OP_OVERFLOW_REG, REG_A1, 0]
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
        let program = assemble(script, HashMap::new());
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
        let (name, ver, program) = assemble(
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
                [ADD_REG_VAL, REG_D0, 10],
                [SUB_REG_VAL, REG_D3, 234],
                [OP_COPY_REG_REG, REG_D2, REG_D1],
                [OP_JMP, 0, 2],
                [OP_NOP, 0, 0],
                [OP_MEM_READ, 0, 4],
                [OP_NOP, 0, 0],
            ]
        );
    }

    #[test]
    fn test_addressing() {
        let script = r"test prog
        1
        ADDRH A0 2
        CPY D0 xFF
        CPY D1 xAA
        ADDRH A0 D0
        ADDRL A0 D1
        ADDRH A1 D1
        ADDRL A1 D0
        ";
        let (_, _, program) = assemble(
            script.lines().map(|str| str.to_string()).collect(),
            HashMap::new(),
        )
        .unwrap();
        assert_eq!(
            program,
            vec![
                [OP_LOAD_ADDR_HIGH_VAL, REG_A0, 2],
                [OP_COPY_REG_VAL, REG_D0, 255],
                [OP_COPY_REG_VAL, REG_D1, 170],
                [OP_LOAD_ADDR_HIGH, REG_A0, REG_D0],
                [OP_LOAD_ADDR_LOW, REG_A0, REG_D1],
                [OP_LOAD_ADDR_HIGH, REG_A1, REG_D1],
                [OP_LOAD_ADDR_LOW, REG_A1, REG_D0],
                [OP_NOP, 0, 0],
            ]
        );
    }
}
