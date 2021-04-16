use crate::common::Instruction;
use crate::constants::code::*;
use crate::constants::hardware::*;
use crate::tape_reader::read_tape;
use anyhow::Result;

pub fn start(path: &str) -> Result<()> {
    println!("Decompiling tape at {}", path);

    let tape = read_tape(path)?;

    println!(
        "\n\nProgram\nName: {}\nVersion: {}",
        tape.name, tape.version
    );
    println!("{} ops, {}b data", tape.ops.len(), tape.data.len());
    println!("\n\nOps:");
    let mut jmp_target = vec![];
    for op in &tape.ops {
        if matches!(
            op[0],
            OP_JMP | OP_JE | OP_JNE | OP_JL | OP_JG | OP_OVERFLOW | OP_NOT_OVERFLOW
        ) {
            let addr = u16::from_be_bytes([op[1], op[2]]) as usize;
            jmp_target.push(addr);
        }
    }
    for (idx, op) in tape.ops.iter().enumerate() {
        let lbl = if jmp_target.contains(&idx) {
            format!("{:04X}", idx)
        } else {
            String::from("    ")
        };
        println!("{}", decode(op, &tape.data, lbl));
    }

    Ok(())
}

fn decode(op: &Instruction, strings: &[u8], lbl: String) -> String {
    let (op, p1, p2): (&str, String, String) = match op[0] {
        OP_ADD_REG_VAL => ("ADD", decode_reg(op[1]), decode_num(op[2])),
        OP_ADD_REG_REG => ("ADD", decode_reg(op[1]), decode_reg(op[2])),
        OP_COPY_REG_VAL => ("CPY", decode_reg(op[1]), decode_num(op[2])),
        OP_COPY_REG_REG => ("CPY", decode_reg(op[1]), decode_reg(op[2])),
        OP_SUB_REG_VAL => ("SUB", decode_reg(op[1]), decode_num(op[2])),
        OP_SUB_REG_REG => ("SUB", decode_reg(op[1]), decode_reg(op[2])),
        OP_OPEN_FILE => ("FOPEN", String::new(), String::new()),
        OP_PRINT_LN => ("PRTLN", String::new(), String::new()),
        OP_INC => ("INC", decode_reg(op[1]), String::new()),
        OP_DEC => ("DEC", decode_reg(op[1]), String::new()),
        OP_PRINT_REG => ("PRT", decode_reg(op[1]), String::new()),
        OP_SKIP_FILE => ("FSKIP", decode_reg(op[1]), String::new()),
        OP_PRINT_VAL => ("PRT", decode_num(op[1]), String::new()),
        OP_CMP_REG_VAL => ("CMP", decode_reg(op[1]), decode_num(op[2])),
        OP_CMP_REG_REG => ("CMP", decode_reg(op[1]), decode_reg(op[2])),
        OP_READ_FILE => ("FREAD", decode_addr(op[1], op[2]), String::new()),
        OP_WRITE_FILE => ("FWRITE", decode_addr(op[1], op[2]), String::new()),
        OP_SEEK_FILE => ("FSEEK", String::new(), String::new()),
        OP_MEM_READ => ("MEMR", decode_addr(op[1], op[2]), String::new()),
        OP_MEM_WRITE => ("MEMW", decode_addr(op[1], op[2]), String::new()),
        OP_PRINT_DAT => ("PRTD", decode_string(op[1], op[2], strings), String::new()),
        OP_PRINT_MEM => ("PRT", decode_addr(op[1], op[2]), String::new()),
        OP_JMP => ("JMP", decode_addr(op[1], op[2]), String::new()),
        OP_JE => ("JE", decode_addr(op[1], op[2]), String::new()),
        OP_JNE => ("JNE", decode_addr(op[1], op[2]), String::new()),
        OP_JL => ("JL", decode_addr(op[1], op[2]), String::new()),
        OP_JG => ("JG", decode_addr(op[1], op[2]), String::new()),
        OP_OVERFLOW => ("OVER", decode_addr(op[1], op[2]), String::new()),
        OP_NOT_OVERFLOW => ("NOVER", decode_addr(op[1], op[2]), String::new()),
        OP_NOP => ("NOP", String::new(), String::new()),
        OP_HALT => ("HALT", String::new(), String::new()),
        _ => {
            return format!(
                "{} {:<6}  {:<5}  {:<5} ** Unknown op",
                lbl, op[0], op[1], op[2]
            )
        }
    };
    format!("{} {:<6}  {:<5}  {:<5}", lbl, op, p1, p2)
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
        .unwrap_or_else(|_| String::from("Unable to decode string"))
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
        _ => String::from("?"),
    }
}
