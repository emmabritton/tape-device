use crate::language::ops::OPS;
use crate::language::parser::params::Param;
use anyhow::{Error, Result};

mod ops;
pub mod parser;

///This method converts a TASM instruction into usable parts for the assembler
///The line can not contain any comments or a label
pub fn parse_line(input: &str) -> Result<(u8, Vec<Param>)> {
    let parts = input.split_whitespace().collect::<Vec<&str>>();

    for op in OPS.iter() {
        if op.matches(&parts[0]) {
            let result = if parts.len() > 1 {
                op.parse(&parts[1..])
            } else {
                op.parse(&[])
            };
            return match result {
                None => Err(Error::msg(format!(
                    "parsing line '{}'\n{}",
                    input,
                    op.error_text()
                ))),
                Some(params) => Ok(params),
            };
        }
    }

    Err(Error::msg(format!(
        "Unable to parse {}, instruction not recognised",
        input
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::code::*;
    use crate::constants::hardware::{REG_A1, REG_D0};
    use anyhow::Context;

    #[test]
    fn test_reg_vals() {
        test_reg_val("ADD", ADD_REG_REG, ADD_REG_VAL);
        test_reg_val("SUB", SUB_REG_REG, SUB_REG_VAL);
        test_reg_val("CPY", CPY_REG_REG, CPY_REG_VAL);
        test_reg_val("CMP", CMP_REG_REG, CMP_REG_VAL);
    }

    #[test]
    fn test_jmps() {
        test_jmp("JMP", JMP_ADDR, JMP_AREG);
        test_jmp("JE", JE_ADDR, JE_AREG);
        test_jmp("JNE", JNE_ADDR, JNE_AREG);
        test_jmp("JL", JL_ADDR, JL_AREG);
        test_jmp("JG", JG_ADDR, JG_AREG);
        test_jmp("OVER", OVER_ADDR, OVER_AREG);
        test_jmp("NOVER", NOVER_ADDR, NOVER_AREG);
        test_jmp("CALL", CALL_ADDR, CALL_AREG);
    }

    #[test]
    fn test_no_params() {
        test_no_param("HALT", HALT);
        test_no_param("NOP", NOP);
        test_no_param("FOPEN", FOPEN);
        test_no_param("FSEEK", FSEEK);
        test_no_param("RET", RET);
        test_no_param("SWPAR", SWPAR);
        test_no_param("CMPAR", CMPAR);
        test_no_param("PRTLN", PRTLN);
    }

    #[test]
    fn test_single_regs() {
        test_single_reg("INC", INC_REG);
        test_single_reg("DEC", DEC_REG);
        test_single_reg("POP", POP_REG);
    }

    #[test]
    fn test_strings() {
        test_string("PRTD", PRTD_STR);
    }

    #[test]
    fn test_mems() {
        test_mem("MEMR", MEMR_ADDR, MEMR_AREG);
        test_mem("MEMW", MEMW_ADDR, MEMW_AREG);
        test_mem("FILER", FILER_ADDR, FILER_AREG);
        test_mem("FILEW", FILEW_ADDR, FILEW_AREG);
    }

    #[test]
    fn test_reg_reg_addrs() {
        test_reg_reg_addr("CPYA0", CPY_A0_ADDR, CPY_A0_REG_REG);
        test_reg_reg_addr("CPYA1", CPY_A1_ADDR, CPY_A1_REG_REG);
    }

    #[test]
    fn test_reg_regs() {
        test_reg_reg("LDA0", LDA0_REG_REG);
        test_reg_reg("LDA1", LDA1_REG_REG);
    }

    #[test]
    fn test_regvals() {
        test_regval("PRT", PRT_REG, PRT_VAL);
        test_regval("PRTC", PRTC_REG, PRTC_VAL);
        test_regval("FSKIP", FSKIP_REG, FSKIP_VAL);
    }

    #[test]
    fn test_addrregvals() {
        test_addrregval("PUSH", PUSH_REG, PUSH_VAL);
    }

    #[test]
    fn new_addrreg_regvals() {
        test_addrreg_regval("ARG", ARG_REG_REG, ARG_REG_VAL);
    }

    const LABEL: &str = "label";
    const DATA_REG: &str = "D0";
    const ADDR_REG: &str = "a1";
    const NUMBER: &str = "34";
    const NUMBER_HEX: &str = "x3a";
    const ADDR: &str = "@900";
    const ADDR_HEX: &str = "@xFe00";

    fn test_parse_line(input: String, opcode: u8, param1: Param, param2: Param) {
        let result = parse_line(&input)
            .context(format!("testing '{}'", input))
            .unwrap();
        assert_eq!(result.0, opcode, "parse line opcode '{}'", input);
        if result.1.len() > 0 {
            assert_eq!(result.1[0], param1, "parse line first param '{}'", input);
        } else {
            assert_eq!(param1, Param::Empty, "parse line first empty '{}'", input);
        }
        if result.1.len() > 1 {
            assert_eq!(result.1[1], param2, "parse line second param '{}'", input);
        } else {
            assert_eq!(param2, Param::Empty, "parse line second empty '{}'", input);
        }
    }

    fn test_invalid_parse_line(
        op: &str,
        param1_valid: Vec<&str>,
        param2_valid: Vec<&str>,
        param1_invalid: Vec<&str>,
        param2_invalid: Vec<&str>,
    ) {
        for valid in &param1_valid {
            for invalid in &param2_invalid {
                assert!(
                    parse_line(&format!("{} {} {}", op, valid, invalid)).is_err(),
                    "vi {} {} for {}",
                    valid,
                    invalid,
                    op
                );
            }
        }
        for invalid in &param1_invalid {
            for valid in &param2_valid {
                assert!(
                    parse_line(&format!("{} {} {}", op, invalid, valid)).is_err(),
                    "iv {} {} for {}",
                    invalid,
                    valid,
                    op
                );
            }
        }
        for invalid1 in &param1_valid {
            for invalid2 in &param2_invalid {
                assert!(
                    parse_line(&format!("{} {} {}", op, invalid1, invalid2)).is_err(),
                    "ii {} {} for {}",
                    invalid1,
                    invalid2,
                    op
                );
            }
        }
    }

    /// Test operations that support
    /// data_reg data_reg
    /// data_reg number
    fn test_reg_val(op: &str, opcode_reg_reg: u8, opcode_reg_val: u8) {
        test_parse_line(
            format!("{} {} {}", op, DATA_REG, DATA_REG),
            opcode_reg_reg,
            Param::DataReg(REG_D0),
            Param::DataReg(REG_D0),
        );
        test_parse_line(
            format!("{} {} {}", op, DATA_REG, NUMBER),
            opcode_reg_val,
            Param::DataReg(REG_D0),
            Param::Number(34),
        );
        test_parse_line(
            format!("{} {} {}", op, DATA_REG, NUMBER_HEX),
            opcode_reg_val,
            Param::DataReg(REG_D0),
            Param::Number(58),
        );
        test_invalid_parse_line(
            op,
            vec![DATA_REG],
            vec![DATA_REG, NUMBER, NUMBER_HEX],
            vec![LABEL, NUMBER, NUMBER_HEX, ADDR, ADDR_HEX, ADDR_REG],
            vec![LABEL, ADDR, ADDR_HEX, ADDR_REG],
        );
    }

    /// Test operations that supports
    /// reg|addr_reg
    fn test_single_reg(op: &str, opcode: u8) {
        test_parse_line(
            format!("{} {}", op, DATA_REG),
            opcode,
            Param::DataReg(REG_D0),
            Param::Empty,
        );
        test_parse_line(
            format!("{} {}", op, ADDR_REG),
            opcode,
            Param::AddrReg(REG_A1),
            Param::Empty,
        );
        test_invalid_parse_line(
            op,
            vec![ADDR_REG, DATA_REG],
            vec![],
            vec![NUMBER, NUMBER_HEX, ADDR, ADDR_HEX, LABEL],
            vec![
                NUMBER, NUMBER_HEX, ADDR, ADDR_HEX, ADDR_REG, DATA_REG, LABEL,
            ],
        );
    }

    /// Test operations that support no params
    fn test_no_param(op: &str, opcode: u8) {
        test_parse_line(format!("{}", op), opcode, Param::Empty, Param::Empty);
        test_invalid_parse_line(
            op,
            vec![],
            vec![],
            vec![
                NUMBER, NUMBER_HEX, ADDR, ADDR_HEX, ADDR_REG, DATA_REG, LABEL,
            ],
            vec![
                NUMBER, NUMBER_HEX, ADDR, ADDR_HEX, ADDR_REG, DATA_REG, LABEL,
            ],
        );
    }

    /// Test operations that support
    /// str_key
    fn test_string(op: &str, opcode: u8) {
        test_parse_line(
            format!("{} {}", op, LABEL),
            opcode,
            Param::StrKey(String::from("label")),
            Param::Empty,
        );
        test_invalid_parse_line(
            op,
            vec![],
            vec![],
            vec![
                NUMBER, NUMBER_HEX, ADDR, ADDR_HEX, ADDR_REG, DATA_REG, LABEL,
            ],
            vec![
                NUMBER, NUMBER_HEX, ADDR, ADDR_HEX, ADDR_REG, DATA_REG, LABEL,
            ],
        );
    }

    /// Test operations that support
    /// addr|label
    /// addr_reg
    fn test_jmp(op: &str, opcode_addr: u8, opcode_addr_reg: u8) {
        test_parse_line(
            format!("{} {}", op, ADDR),
            opcode_addr,
            Param::Addr(900),
            Param::Empty,
        );
        test_parse_line(
            format!("{} {}", op, ADDR_HEX),
            opcode_addr,
            Param::Addr(65024),
            Param::Empty,
        );
        test_parse_line(
            format!("{} {}", op, ADDR_REG),
            opcode_addr_reg,
            Param::AddrReg(REG_A1),
            Param::Empty,
        );
        test_parse_line(
            format!("{} {}", op, LABEL),
            opcode_addr,
            Param::Label(String::from("label")),
            Param::Empty,
        );
        test_invalid_parse_line(
            op,
            vec![ADDR, ADDR_HEX, LABEL, ADDR_REG],
            vec![],
            vec![DATA_REG, NUMBER_HEX, NUMBER],
            vec![
                DATA_REG, NUMBER, NUMBER_HEX, ADDR_REG, ADDR, ADDR_HEX, LABEL,
            ],
        );
    }

    /// Test operations that support
    /// addr
    /// addr_reg
    fn test_mem(op: &str, opcode_addr: u8, opcode_addr_reg: u8) {
        test_parse_line(
            format!("{} {}", op, ADDR),
            opcode_addr,
            Param::Addr(900),
            Param::Empty,
        );
        test_parse_line(
            format!("{} {}", op, ADDR_HEX),
            opcode_addr,
            Param::Addr(65024),
            Param::Empty,
        );
        test_parse_line(
            format!("{} {}", op, ADDR_REG),
            opcode_addr_reg,
            Param::AddrReg(REG_A1),
            Param::Empty,
        );
        test_invalid_parse_line(
            op,
            vec![ADDR, ADDR_HEX, ADDR_REG],
            vec![],
            vec![DATA_REG, NUMBER_HEX, NUMBER, LABEL],
            vec![
                DATA_REG, NUMBER, NUMBER_HEX, ADDR_REG, ADDR, ADDR_HEX, LABEL,
            ],
        );
    }

    /// Test operations that support
    /// addr|label
    /// data_reg data_reg
    fn test_reg_reg_addr(op: &str, opcode_addr: u8, opcode_regs: u8) {
        test_parse_line(
            format!("{} {}", op, ADDR),
            opcode_addr,
            Param::Addr(900),
            Param::Empty,
        );
        test_parse_line(
            format!("{} {}", op, ADDR_HEX),
            opcode_addr,
            Param::Addr(65024),
            Param::Empty,
        );
        test_parse_line(
            format!("{} {}", op, LABEL),
            opcode_addr,
            Param::Label(String::from("label")),
            Param::Empty,
        );
        test_parse_line(
            format!("{} {} {}", op, DATA_REG, DATA_REG),
            opcode_regs,
            Param::DataReg(REG_D0),
            Param::DataReg(REG_D0),
        );
        test_invalid_parse_line(
            op,
            vec![ADDR, ADDR_HEX, LABEL, DATA_REG],
            vec![DATA_REG],
            vec![NUMBER_HEX, NUMBER],
            vec![NUMBER, NUMBER_HEX, ADDR_REG, ADDR, ADDR_HEX, LABEL],
        );
    }

    /// Test operations that support
    /// data_reg data_reg
    fn test_reg_reg(op: &str, opcode: u8) {
        test_parse_line(
            format!("{} {} {}", op, DATA_REG, DATA_REG),
            opcode,
            Param::DataReg(REG_D0),
            Param::DataReg(REG_D0),
        );
        test_invalid_parse_line(
            op,
            vec![DATA_REG],
            vec![DATA_REG],
            vec![NUMBER, NUMBER_HEX, ADDR_REG, ADDR, ADDR_HEX, LABEL],
            vec![NUMBER, NUMBER_HEX, ADDR_REG, ADDR, ADDR_HEX, LABEL],
        );
    }

    /// Test operations that support
    /// data_reg
    /// num
    fn test_regval(op: &str, opcode_reg: u8, opcode_val: u8) {
        test_parse_line(
            format!("{} {}", op, DATA_REG),
            opcode_reg,
            Param::DataReg(REG_D0),
            Param::Empty,
        );
        test_parse_line(
            format!("{} {}", op, NUMBER),
            opcode_val,
            Param::Number(34),
            Param::Empty,
        );
        test_parse_line(
            format!("{} {}", op, NUMBER_HEX),
            opcode_val,
            Param::Number(58),
            Param::Empty,
        );
        test_invalid_parse_line(
            op,
            vec![DATA_REG, NUMBER, NUMBER_HEX],
            vec![],
            vec![ADDR_REG, ADDR, ADDR_HEX, LABEL],
            vec![
                DATA_REG, NUMBER, NUMBER_HEX, ADDR_REG, ADDR, ADDR_HEX, LABEL,
            ],
        );
    }

    /// Test operations that support
    /// data_reg|addr_reg
    /// num
    fn test_addrregval(op: &str, opcode_reg: u8, opcode_val: u8) {
        test_parse_line(
            format!("{} {}", op, DATA_REG),
            opcode_reg,
            Param::DataReg(REG_D0),
            Param::Empty,
        );
        test_parse_line(
            format!("{} {}", op, ADDR_REG),
            opcode_reg,
            Param::AddrReg(REG_A1),
            Param::Empty,
        );
        test_parse_line(
            format!("{} {}", op, NUMBER),
            opcode_val,
            Param::Number(34),
            Param::Empty,
        );
        test_parse_line(
            format!("{} {}", op, NUMBER_HEX),
            opcode_val,
            Param::Number(58),
            Param::Empty,
        );
        test_invalid_parse_line(
            op,
            vec![DATA_REG, NUMBER, NUMBER_HEX, ADDR_REG],
            vec![],
            vec![ADDR, ADDR_HEX, LABEL],
            vec![
                DATA_REG, NUMBER, NUMBER_HEX, ADDR_REG, ADDR, ADDR_HEX, LABEL,
            ],
        );
    }

    /// Test operations that support
    /// data_reg|addr_reg
    /// num|data_reg
    fn test_addrreg_regval(op: &str, opcode_regreg: u8, opcode_regval: u8) {
        test_parse_line(
            format!("{} {} {}", op, DATA_REG, DATA_REG),
            opcode_regreg,
            Param::DataReg(REG_D0),
            Param::DataReg(REG_D0),
        );
        test_parse_line(
            format!("{} {} {}", op, ADDR_REG, DATA_REG),
            opcode_regreg,
            Param::AddrReg(REG_A1),
            Param::DataReg(REG_D0),
        );
        test_parse_line(
            format!("{} {} {}", op, DATA_REG, NUMBER),
            opcode_regval,
            Param::DataReg(REG_D0),
            Param::Number(34),
        );
        test_parse_line(
            format!("{} {} {}", op, DATA_REG, NUMBER_HEX),
            opcode_regval,
            Param::DataReg(REG_D0),
            Param::Number(58),
        );

        test_invalid_parse_line(
            op,
            vec![DATA_REG, ADDR_REG],
            vec![DATA_REG, NUMBER_HEX, NUMBER],
            vec![ADDR, ADDR_HEX, LABEL, NUMBER_HEX, NUMBER],
            vec![ADDR_REG, ADDR, ADDR_HEX, LABEL],
        );
    }
}
