use crate::parsing::single::{
    test_single_instruction, test_single_invalid_instruction, test_single_invalid_instruction_model,
};
use crate::parsing::{
    new_program_model_with_data, new_program_model_with_label, new_program_model_with_string,
};
use tape_device::constants::code::ARG_REG_VAL;
use tape_device::constants::hardware::{REG_A0, REG_A1, REG_ACC, REG_D0, REG_D1, REG_D2, REG_D3};
use tape_device::language::parser::params::Param::{AddrReg, DataReg, Number};

#[test]
#[rustfmt::skip]
fn test_regs_val() {
    for op in [("ARG", ARG_REG_VAL)] {
        test_valid_int(op.0, op.1);
        test_valid_hex(op.0, op.1);
        test_valid_chr(op.0, op.1);
        test_valid_bin(op.0, op.1);
        test_invalid_int(op.0, op.1);
        test_invalid_hex(op.0, op.1);
        test_invalid_chr(op.0, op.1);
        test_invalid_bin(op.0, op.1);
        test_invalid_bad_reg(op.0, op.1);
        test_invalid_addr(op.0, op.1);
        test_invalid_constant(op.0, op.1);
        test_invalid_keys(op.0, op.1);
        test_invalid_no_2nd_param(op.0, op.1);
        test_invalid_due_3rg_param(op.0, op.1);
    }
}

#[rustfmt::skip]
fn test_valid_int(op: &str, opcode: u8) {
    test_single_instruction(&format!("{} D0 0", op), opcode, vec![DataReg(REG_D0), Number(0)]);
    test_single_instruction(&format!("{} D1 1", op), opcode, vec![DataReg(REG_D1), Number(1)]);
    test_single_instruction(&format!("{} D2 100", op), opcode, vec![DataReg(REG_D2), Number(100)]);
    test_single_instruction(&format!("{} D3 255", op), opcode, vec![DataReg(REG_D3), Number(255)]);
}

#[rustfmt::skip]
fn test_valid_hex(op: &str, opcode: u8) {
    test_single_instruction(&format!("{} ACC x0", op), opcode, vec![DataReg(REG_ACC), Number(0)]);
    test_single_instruction(&format!("{} A0 x1", op), opcode, vec![AddrReg(REG_A0), Number(1)]);
    test_single_instruction(&format!("{} A1 xF", op), opcode, vec![AddrReg(REG_A1), Number(15)]);
    test_single_instruction(&format!("{} D0 x4F", op), opcode, vec![DataReg(REG_D0), Number(79)]);
}

#[rustfmt::skip]
fn test_valid_chr(op: &str, opcode: u8) {
    test_single_instruction(&format!("{} D1 '0'", op), opcode, vec![DataReg(REG_D1), Number(48)]);
    test_single_instruction(&format!("{} D2 'a'", op), opcode, vec![DataReg(REG_D2), Number(97)]);
    test_single_instruction(&format!("{} D3 '.'", op), opcode, vec![DataReg(REG_D3), Number(46)]);
    test_single_instruction(&format!("{} ACC 'A'", op), opcode, vec![DataReg(REG_ACC), Number(65)]);
}

#[rustfmt::skip]
fn test_valid_bin(op: &str, opcode: u8) {
    test_single_instruction(&format!("{} A0 b00000000", op), opcode, vec![AddrReg(REG_A0), Number(0)]);
    test_single_instruction(&format!("{} A1 b10001000", op), opcode, vec![AddrReg(REG_A1), Number(136)]);
    test_single_instruction(&format!("{} D0 b11111111", op), opcode, vec![DataReg(REG_D0), Number(255)]);
}

#[rustfmt::skip]
fn test_invalid_int(op: &str, opcode: u8) {
    test_single_invalid_instruction(&format!("{} D0 -1", op), opcode, &format!("{} supports", op));
    test_single_invalid_instruction(&format!("{} D1 256", op), opcode, &format!("{} supports", op));
    test_single_invalid_instruction(&format!("{} D2 600", op), opcode, &format!("{} supports", op));
}

#[rustfmt::skip]
fn test_invalid_hex(op: &str, opcode: u8) {
    test_single_invalid_instruction(&format!("{} D0 FF", op), opcode, &format!("{} supports", op));
    test_single_invalid_instruction(&format!("{} D1 xFF1", op), opcode, &format!("{} supports", op));
    test_single_invalid_instruction(&format!("{} D2 xH", op), opcode, &format!("{} supports", op));
    test_single_invalid_instruction(&format!("{} D3 0x0", op), opcode, &format!("{} supports", op));
}

#[rustfmt::skip]
fn test_invalid_bin(op: &str, opcode: u8) {
    test_single_invalid_instruction(&format!("{} A0 b0", op), opcode, &format!("{} supports", op));
    test_single_invalid_instruction(&format!("{} A1 10101010", op), opcode, &format!("{} supports", op));
    test_single_invalid_instruction(&format!("{} D2 b101101101", op), opcode, &format!("{} supports", op));
    test_single_invalid_instruction(&format!("{} D3 b10000002", op), opcode, &format!("{} supports", op));
}

#[rustfmt::skip]
fn test_invalid_chr(op: &str, opcode: u8) {
    test_single_invalid_instruction(&format!("{} D0 ''", op), opcode, &format!("{} supports", op));
    test_single_invalid_instruction(&format!("{} D1 'aa'", op), opcode, &format!("{} supports", op));
}

#[rustfmt::skip]
fn test_invalid_bad_reg(op: &str, opcode: u8) {
    test_single_invalid_instruction(&format!("{} D5 0", op), opcode, &format!("{} supports", op));
    test_single_invalid_instruction(&format!("{} A2 0", op), opcode, &format!("{} supports", op));
}

#[rustfmt::skip]
fn test_invalid_addr(op: &str, opcode: u8) {
    test_single_invalid_instruction(&format!("{} D0 @0", op), opcode, &format!("{} supports", op));
    test_single_invalid_instruction(&format!("{} D0 @x0", op), opcode, &format!("{} supports", op));
}

#[rustfmt::skip]
fn test_invalid_constant(op: &str, opcode: u8) {
    test_single_invalid_instruction(&format!("{} A0 foo", op), opcode, &format!("{} supports", op));
}

#[rustfmt::skip]
fn test_invalid_keys(op: &str, opcode: u8) {
    let model_data = new_program_model_with_data("", "", "foo");
    test_single_invalid_instruction_model(model_data, &format!("{} D0 foo", op), opcode, &format!("{} supports", op));

    let model_str = new_program_model_with_string("", "", "foo");
    test_single_invalid_instruction_model(model_str, &format!("{} D0 foo", op), opcode, &format!("{} supports", op));

    let model_lbl = new_program_model_with_label("", "", "foo");
    test_single_invalid_instruction_model(model_lbl, &format!("{} A1 foo", op), opcode, &format!("{} supports", op));
}

#[rustfmt::skip]
fn test_invalid_no_2nd_param(op: &str, opcode: u8) {
    test_single_invalid_instruction(&format!("{} D0", op), opcode, &format!("{} supports", op));
}

#[rustfmt::skip]
fn test_invalid_due_3rg_param(op: &str, opcode: u8) {
    test_single_invalid_instruction(&format!("{} D0 0 1", op), opcode, &format!("{} supports", op));
}
