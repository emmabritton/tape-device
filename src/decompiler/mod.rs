use crate::constants::code::*;
use crate::constants::get_byte_count;
use crate::constants::hardware::*;
use crate::tape_reader::read_tape;
use anyhow::Result;
use std::collections::HashSet;

pub struct Decoded {
    pub bytes: Vec<u8>,
    pub strings: Vec<String>,
    pub byte_offset: usize,
    pub is_jump_target: bool,
}

impl Decoded {
    pub fn new(
        bytes: Vec<u8>,
        strings: Vec<String>,
        byte_offset: usize,
        is_jump_target: bool,
    ) -> Self {
        Decoded {
            bytes,
            strings,
            byte_offset,
            is_jump_target,
        }
    }
}

impl Decoded {
    pub fn is_param_16_bit(&self) -> bool {
        matches!(
            self.bytes[0],
            MEMR_ADDR
                | MEMW_ADDR
                | PRTD_STR
                | FILER_ADDR
                | FILEW_ADDR
                | JMP_ADDR
                | JE_ADDR
                | JG_ADDR
                | JNE_ADDR
                | JL_ADDR
                | OVER_ADDR
                | NOVER_ADDR
                | CALL_ADDR
                | CPY_AREG_ADDR
                | CMP_AREG_ADDR
        )
    }
}

pub fn start(path: &str) -> Result<()> {
    println!("Decompiling tape at {}", path);

    let mut tape = read_tape(path)?;

    println!(
        "\n\nProgram\nName: {}\nVersion: {}",
        tape.name, tape.version
    );
    let (strings, unused) = collect_strings(&tape.ops, &tape.data);

    println!(
        "{}b ops, {}b data ({}b unused or possibly indirectly referenced)",
        tape.ops.len(),
        tape.data.len(),
        unused
    );
    println!("\n\nStrings:");
    for content in &strings {
        println!("\"{}\"", content);
    }
    println!("\n\nOps:");
    let jmp_target = collect_jump_targets(&tape.ops);

    let mut pc = 0;
    println!("byte  addr op      param1 param2");
    while !tape.ops.is_empty() {
        let op = decode(&mut tape.ops, &tape.data, pc, jmp_target.contains(&pc));
        let lbl = if op.is_jump_target {
            format!("{:04X}", op.byte_offset)
        } else {
            String::from("    ")
        };
        println!(
            "{: <4}  {} {:<6}  {:<5}  {:<5}",
            pc, lbl, op.strings[0], op.strings[1], op.strings[2]
        );
        pc += get_byte_count(op.bytes[0]);
    }

    Ok(())
}

pub fn collect_strings(ops: &[u8], data: &[u8]) -> (Vec<String>, usize) {
    let mut op_idx = 0;
    let mut addresses = HashSet::new();
    while op_idx < ops.len() {
        if ops[op_idx] == PRTD_STR {
            let addr = u16::from_be_bytes([ops[op_idx + 1], ops[op_idx + 2]]);
            addresses.insert(addr);
        }
        op_idx += get_byte_count(ops[op_idx]);
    }

    let mut bytes_accounted = 0;
    let mut results = vec![];
    for str_addr in addresses {
        bytes_accounted += data[str_addr as usize] as usize + 1;
        let content = &data
            [(str_addr + 1) as usize..(str_addr as usize + 1 + data[str_addr as usize] as usize)];
        results.push(String::from_utf8_lossy(content).to_string());
    }
    (results, data.len() - bytes_accounted)
}

pub fn collect_jump_targets(ops: &[u8]) -> Vec<usize> {
    let mut jmp_target = vec![];
    for op in ops.windows(3) {
        if matches!(
            op[0],
            JMP_ADDR | JE_ADDR | JNE_ADDR | JL_ADDR | JG_ADDR | OVER_ADDR | NOVER_ADDR | CALL_ADDR
        ) {
            let addr = u16::from_be_bytes([op[1], op[2]]) as usize;
            jmp_target.push(addr);
        }
    }
    jmp_target
}

pub fn decode(
    bytes: &mut Vec<u8>,
    strings: &[u8],
    byte_offset: usize,
    is_jump_target: bool,
) -> Decoded {
    let mut op = vec![];
    let mut count = get_byte_count(bytes[0]);
    while count > 0 {
        op.push(bytes.remove(0));
        count -= 1;
    }
    let (op_str, params): (&str, Vec<String>) = match op[0] {
        ADD_REG_VAL => ("ADD", vec![decode_reg(op[1]), decode_num(op[2])]),
        ADD_REG_REG => ("ADD", vec![decode_reg(op[1]), decode_reg(op[2])]),
        CPY_REG_VAL => ("CPY", vec![decode_reg(op[1]), decode_num(op[2])]),
        CPY_REG_REG | CPY_AREG_AREG => ("CPY", vec![decode_reg(op[1]), decode_reg(op[2])]),
        CPY_AREG_ADDR => ("CPY", vec![decode_reg(op[1]), decode_addr(op[2], op[3])]),
        CPY_REG_REG_AREG | CPY_AREG_REG_REG => (
            "CPY",
            vec![decode_reg(op[1]), decode_reg(op[2]), decode_reg(op[3])],
        ),
        CMP_REG_VAL => ("CMP", vec![decode_reg(op[1]), decode_num(op[2])]),
        CMP_REG_REG | CMP_AREG_AREG => ("CMP", vec![decode_reg(op[1]), decode_reg(op[2])]),
        CMP_AREG_ADDR => ("CMP", vec![decode_reg(op[1]), decode_addr(op[2], op[3])]),
        CMP_REG_REG_AREG | CMP_AREG_REG_REG => (
            "CMP",
            vec![decode_reg(op[1]), decode_reg(op[2]), decode_reg(op[3])],
        ),
        SUB_REG_VAL => ("SUB", vec![decode_reg(op[1]), decode_num(op[2])]),
        SUB_REG_REG => ("SUB", vec![decode_reg(op[1]), decode_reg(op[2])]),
        FOPEN => ("FOPEN", vec![]),
        PRTLN => ("PRTLN", vec![]),
        INC_REG => ("INC", vec![decode_reg(op[1])]),
        DEC_REG => ("DEC", vec![decode_reg(op[1])]),
        FSKIP_REG => ("FSKIP", vec![decode_reg(op[1])]),
        PRT_VAL => ("PRT", vec![decode_num(op[1])]),
        PRT_REG => ("PRT", vec![decode_reg(op[1])]),
        PRTC_VAL => ("PRTC", vec![decode_num(op[1])]),
        PRTC_REG => ("PRTC", vec![decode_reg(op[1])]),
        FILER_ADDR => ("FILER", vec![decode_addr(op[1], op[2])]),
        FILER_AREG => ("FILER", vec![decode_reg(op[1])]),
        FILEW_ADDR => ("FILEW", vec![decode_addr(op[1], op[2])]),
        FILEW_AREG => ("FILEW", vec![decode_reg(op[1])]),
        FSEEK => ("FSEEK", vec![]),
        MEMR_ADDR => ("MEMR", vec![decode_addr(op[1], op[2])]),
        MEMR_AREG => ("MEMR", vec![decode_reg(op[1])]),
        MEMW_ADDR => ("MEMW", vec![decode_addr(op[1], op[2])]),
        MEMW_AREG => ("MEMW", vec![decode_reg(op[1])]),
        PRTD_STR => ("PRTD", vec![decode_string(op[1], op[2], strings)]),
        JMP_ADDR => ("JMP", vec![decode_addr(op[1], op[2])]),
        JE_ADDR => ("JE", vec![decode_addr(op[1], op[2])]),
        JNE_ADDR => ("JNE", vec![decode_addr(op[1], op[2])]),
        JL_ADDR => ("JL", vec![decode_addr(op[1], op[2])]),
        JG_ADDR => ("JG", vec![decode_addr(op[1], op[2])]),
        OVER_ADDR => ("OVER", vec![decode_addr(op[1], op[2])]),
        NOVER_ADDR => ("NOVER", vec![decode_addr(op[1], op[2])]),
        JMP_AREG => ("JMP", vec![decode_reg(op[1])]),
        JE_AREG => ("JE", vec![decode_reg(op[1])]),
        JNE_AREG => ("JNE", vec![decode_reg(op[1])]),
        JL_AREG => ("JL", vec![decode_reg(op[1])]),
        JG_AREG => ("JG", vec![decode_reg(op[1])]),
        OVER_AREG => ("OVER", vec![decode_reg(op[1])]),
        NOVER_AREG => ("NOVER", vec![decode_reg(op[1])]),
        NOP => ("NOP", vec![]),
        HALT => ("HALT", vec![]),
        RET => ("RET", vec![]),
        CALL_ADDR => ("CALL", vec![decode_addr(op[1], op[2])]),
        CALL_AREG => ("CALL", vec![decode_reg(op[1])]),
        POP_REG => ("POP", vec![decode_reg(op[1])]),
        PUSH_REG => ("PUSH", vec![decode_reg(op[1])]),
        PUSH_VAL => ("PUSH", vec![decode_num(op[1])]),
        SWP_REG_REG | SWP_AREG_AREG => ("SWP", vec![decode_reg(op[1]), decode_reg(op[2])]),
        ARG_REG_VAL => ("ARG", vec![decode_reg(op[1]), decode_num(op[2])]),
        ARG_REG_REG => ("ARG", vec![decode_reg(op[1]), decode_reg(op[2])]),
        FCHK_AREG => ("FCHK", vec![decode_reg(op[1])]),
        FCHK_ADDR => ("FCHK", vec![decode_addr(op[1], op[2])]),
        IPOLL_AREG => ("IPOLL", vec![decode_reg(op[1])]),
        IPOLL_ADDR => ("IPOLL", vec![decode_addr(op[1], op[2])]),
        RSTR_AREG => ("RSTR", vec![decode_reg(op[1])]),
        RSTR_ADDR => ("RSTR", vec![decode_addr(op[1], op[2])]),
        PSTR_AREG => ("PSTR", vec![decode_reg(op[1])]),
        PSTR_ADDR => ("PSTR", vec![decode_addr(op[1], op[2])]),
        RCHR_REG => ("RCHR", vec![decode_reg(op[1])]),
        _ => ("???", vec![]),
    };
    let mut strings = params;
    strings.insert(0, op_str.to_string());
    Decoded::new(op, strings, byte_offset, is_jump_target)
}

fn decode_string(b1: u8, b2: u8, data: &[u8]) -> String {
    let mut addr = u16::from_be_bytes([b1, b2]) as usize;
    let len = data[addr] as usize;
    addr += 1;
    let mut output = vec![];
    for i in 0..len {
        output.push(data[addr + i]);
    }
    String::from_utf8(output)
        .map(|str| format!("\"{}\"", str))
        .unwrap_or_else(|_| format!("Unable to decode string (address was {})", addr))
}

fn decode_addr(b1: u8, b2: u8) -> String {
    let addr = u16::from_be_bytes([b1, b2]);
    format!("{:04X}", addr)
}

fn decode_num(value: u8) -> String {
    format!("{}", value)
}

fn decode_reg(reg: u8) -> String {
    match reg {
        REG_D0 => String::from("D0"),
        REG_D1 => String::from("D1"),
        REG_D2 => String::from("D2"),
        REG_D3 => String::from("D3"),
        REG_ACC => String::from("ACC"),
        REG_A0 => String::from("A0"),
        REG_A1 => String::from("A1"),
        _ => String::from("?"),
    }
}
