use crate::parsing::new_program_model;
use tape_device::assembler::parser::parse_op;
use tape_device::assembler::program_model::{ConstantModel, OpModel, ProgramModel, Usage};
use tape_device::language::parser::params::Param;

mod reg_areg;
mod reg_reg;
mod reg_val;
mod regs_val;
mod val;

fn make_op_model(opcode: u8, params: Vec<Param>, line: &str, line_num: usize) -> OpModel {
    OpModel::new(opcode, params, line.to_owned(), line.to_owned(), line_num)
}

#[rustfmt::skip]
fn test_single_instruction(line: &str, op: u8, params: Vec<Param>) {
    let mut model = new_program_model("", "");
    parse_op(&mut model, line, 0).unwrap();

    assert_eq!(model.ops[0], make_op_model(op, params, line, 0), "{}", line);
}

#[rustfmt::skip]
fn test_single_invalid_instruction(line: &str, only_valid_op: u8, partial_error_message: &str) {
    test_single_invalid_instruction_model(new_program_model("", ""), line, only_valid_op, partial_error_message)
}

#[rustfmt::skip]
fn test_single_invalid_instruction_model(mut model: ProgramModel, line: &str, only_valid_op: u8, partial_error_message: &str) {
    if partial_error_message.is_empty() {
        panic!("Error test with blank error check on {}", line);
    }
    let result = parse_op(&mut model, line, 0);
    match result {
        Ok(_) => {
            let op = &model.ops[0];
            assert_ne!(op.opcode, only_valid_op, "{}-{:?}", line, model.ops[0]);
        }
        Err(err) => assert!(err.to_string().contains(partial_error_message), "{}: {}", line, err.to_string())
    }
}
