use crate::constants::code::*;
use crate::constants::get_byte_count;
use crate::constants::hardware::*;
use crate::tape_reader::read_tape;
use anyhow::Result;
use std::collections::HashSet;

pub struct Decoded {
    pub bytes: Vec<u8>,
    pub strings: Vec<String>,
    pub line_num: usize,
    pub is_jump_target: bool,
}

impl Decoded {
    pub fn new(
        bytes: Vec<u8>,
        strings: Vec<String>,
        line_num: usize,
        is_jump_target: bool,
    ) -> Self {
        Decoded {
            bytes,
            strings,
            line_num,
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
                | CPY_A0_ADDR
                | CPY_A1_ADDR
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
            format!("{:04X}", op.line_num)
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
    line_num: usize,
    is_jump_target: bool,
) -> Decoded {
    let mut op = vec![];
    let mut count = get_byte_count(bytes[0]);
    while count > 0 {
        op.push(bytes.remove(0));
        count -= 1;
    }
    let (op_str, p1_str, p2_str): (&str, String, String) = match op[0] {
        ADD_REG_VAL => ("ADD", decode_reg(op[1]), decode_num(op[2])),
        ADD_REG_REG => ("ADD", decode_reg(op[1]), decode_reg(op[2])),
        CPY_REG_VAL => ("CPY", decode_reg(op[1]), decode_num(op[2])),
        CPY_REG_REG => ("CPY", decode_reg(op[1]), decode_reg(op[2])),
        SUB_REG_VAL => ("SUB", decode_reg(op[1]), decode_num(op[2])),
        SUB_REG_REG => ("SUB", decode_reg(op[1]), decode_reg(op[2])),
        FOPEN => ("FOPEN", String::new(), String::new()),
        PRTLN => ("PRTLN", String::new(), String::new()),
        INC_REG => ("INC", decode_reg(op[1]), String::new()),
        DEC_REG => ("DEC", decode_reg(op[1]), String::new()),
        FSKIP_REG => ("FSKIP", decode_reg(op[1]), String::new()),
        PRT_VAL => ("PRT", decode_num(op[1]), String::new()),
        PRT_REG => ("PRT", decode_reg(op[1]), String::new()),
        CMP_REG_VAL => ("CMP", decode_reg(op[1]), decode_num(op[2])),
        CMP_REG_REG => ("CMP", decode_reg(op[1]), decode_reg(op[2])),
        FILER_ADDR => ("FILER", decode_addr(op[1], op[2]), String::new()),
        FILER_AREG => ("FILER", decode_reg(op[1]), String::new()),
        FILEW_ADDR => ("FILEW", decode_addr(op[1], op[2]), String::new()),
        FILEW_AREG => ("FILEW", decode_reg(op[1]), String::new()),
        FSEEK => ("FSEEK", String::new(), String::new()),
        MEMR_ADDR => ("MEMR", decode_addr(op[1], op[2]), String::new()),
        MEMR_AREG => ("MEMR", decode_reg(op[1]), String::new()),
        MEMW_ADDR => ("MEMW", decode_addr(op[1], op[2]), String::new()),
        MEMW_AREG => ("MEMW", decode_reg(op[1]), String::new()),
        PRTD_STR => ("PRTD", decode_string(op[1], op[2], strings), String::new()),
        JMP_ADDR => ("JMP", decode_addr(op[1], op[2]), String::new()),
        JE_ADDR => ("JE", decode_addr(op[1], op[2]), String::new()),
        JNE_ADDR => ("JNE", decode_addr(op[1], op[2]), String::new()),
        JL_ADDR => ("JL", decode_addr(op[1], op[2]), String::new()),
        JG_ADDR => ("JG", decode_addr(op[1], op[2]), String::new()),
        OVER_ADDR => ("OVER", decode_addr(op[1], op[2]), String::new()),
        NOVER_ADDR => ("NOVER", decode_addr(op[1], op[2]), String::new()),
        JMP_AREG => ("JMP", decode_reg(op[1]), String::new()),
        JE_AREG => ("JE", decode_reg(op[1]), String::new()),
        JNE_AREG => ("JNE", decode_reg(op[1]), String::new()),
        JL_AREG => ("JL", decode_reg(op[1]), String::new()),
        JG_AREG => ("JG", decode_reg(op[1]), String::new()),
        OVER_AREG => ("OVER", decode_reg(op[1]), String::new()),
        NOVER_AREG => ("NOVER", decode_reg(op[1]), String::new()),
        NOP => ("NOP", String::new(), String::new()),
        HALT => ("HALT", String::new(), String::new()),
        RET => ("RET", String::new(), String::new()),
        CALL_ADDR => ("CALL", decode_addr(op[1], op[2]), String::new()),
        CALL_AREG => ("CALL", decode_reg(op[1]), String::new()),
        POP_REG => ("POP", decode_reg(op[1]), String::new()),
        PUSH_REG => ("PUSH", decode_reg(op[1]), String::new()),
        PUSH_VAL => ("PUSH", decode_num(op[1]), String::new()),
        SWPAR => ("SWPAR", String::new(), String::new()),
        CMPAR => ("CMPAR", String::new(), String::new()),
        CPY_A0_REG_REG => ("CPYA0", decode_reg(op[1]), decode_reg(op[2])),
        CPY_A0_ADDR => ("CPYA0", decode_addr(op[1], op[2]), String::new()),
        CPY_A1_REG_REG => ("CPYA1", decode_reg(op[1]), decode_reg(op[2])),
        CPY_A1_ADDR => ("CPYA1", decode_addr(op[1], op[2]), String::new()),
        LDA0_REG_REG => ("LDA0", decode_reg(op[1]), decode_reg(op[2])),
        LDA1_REG_REG => ("LDA1", decode_reg(op[1]), decode_reg(op[2])),
        ARG_REG_VAL => ("ARG", decode_reg(op[1]), decode_num(op[2])),
        ARG_REG_REG => ("ARG", decode_reg(op[1]), decode_reg(op[2])),
        _ => ("???", String::new(), String::new()),
    };
    Decoded::new(
        op,
        vec![op_str.to_string(), p1_str, p2_str],
        line_num,
        is_jump_target,
    )
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
