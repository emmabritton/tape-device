use crate::parsing::single::{test_single_instruction, test_single_invalid_instruction};
use tape_device::constants::code::PUSH_VAL;
use tape_device::language::parser::params::Param::Number;

#[test]
#[rustfmt::skip]
fn test_val() {
    for op in [("PUSH", PUSH_VAL)] {
        test_valid_int(op.0, op.1);
        test_invalid_int(op.0, op.1);
        test_invalid_addr(op.0, op.1);
        test_invalid_reg(op.0, op.1);
        test_invalid_hex(op.0, op.1);
        test_invalid_bin(op.0, op.1);
        test_invalid_no_param(op.0, op.1);
        test_invalid_two_param(op.0, op.1);
    }
}

#[rustfmt::skip]
fn test_valid_int(op: &str, opcode: u8) {
    test_single_instruction(&format!("{} 0", op), opcode, vec![Number(0)]);
    test_single_instruction(&format!("{} 10", op), opcode, vec![Number(10)]);
    test_single_instruction(&format!("{} 'a'", op), opcode, vec![Number(97)]);
    test_single_instruction(&format!("{} xF", op), opcode, vec![Number(15)]);
    test_single_instruction(&format!("{} b01010101", op), opcode, vec![Number(85)]);
}

#[rustfmt::skip]
fn test_invalid_int(op: &str, opcode: u8) {
    test_single_invalid_instruction(&format!("{} -1", op), opcode, &format!("{} supports", op));
    test_single_invalid_instruction(&format!("{} 256", op), opcode, &format!("{} supports", op));
}

#[rustfmt::skip]
fn test_invalid_addr(op: &str, opcode: u8) {
    test_single_invalid_instruction(&format!("{} @0", op), opcode, &format!("{} supports", op));
    test_single_invalid_instruction(&format!("{} @x1", op), opcode, &format!("{} supports", op));
}

#[rustfmt::skip]
fn test_invalid_reg(op: &str, opcode: u8) {
    test_single_invalid_instruction(&format!("{} D0", op), opcode, &format!("{} supports", op));
}

#[rustfmt::skip]
fn test_invalid_hex(op: &str, opcode: u8) {
    test_single_invalid_instruction(&format!("{} D0", op), opcode, &format!("{} supports", op));
}

#[rustfmt::skip]
fn test_invalid_bin(op: &str, opcode: u8) {
    test_single_invalid_instruction(&format!("{} D0", op), opcode, &format!("{} supports", op));
}

#[rustfmt::skip]
fn test_invalid_no_param(op: &str, opcode: u8) {
    test_single_invalid_instruction(&format!("{} D0", op), opcode, &format!("{} supports", op));
}

#[rustfmt::skip]
fn test_invalid_two_param(op: &str, opcode: u8) {
    test_single_invalid_instruction(&format!("{} 0 1", op), opcode, &format!("{} supports", op));
}
