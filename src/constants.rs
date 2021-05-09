use crate::constants::code::*;

pub mod hardware {
    pub const RAM_SIZE: usize = 0xFFFF;
    pub const DATA_REG_COUNT: usize = 4;
    pub const ADDR_REG_COUNT: usize = 2;

    pub const REG_ACC: u8 = 0x01;

    pub const REG_D0: u8 = 0x10;
    pub const REG_D1: u8 = 0x11;
    pub const REG_D2: u8 = 0x12;
    pub const REG_D3: u8 = 0x13;

    pub const REG_A0: u8 = 0x20;
    pub const REG_A1: u8 = 0x21;
}

pub mod compare {
    pub const EQUAL: u8 = 0;
    pub const LESSER: u8 = 1;
    pub const GREATER: u8 = 2;
}

pub mod system {
    pub const TAPE_HEADER_1: u8 = 0xFD;
    pub const TAPE_HEADER_2: u8 = 0xA0;

    pub const PRG_VERSION: u8 = 11;
}

pub mod code {
    pub const ADD_REG_REG: u8 = 0x01;
    pub const ADD_REG_VAL: u8 = 0x02;
    pub const SUB_REG_REG: u8 = 0x03;
    pub const SUB_REG_VAL: u8 = 0x04;
    pub const INC_REG: u8 = 0x05;
    pub const DEC_REG: u8 = 0x06;

    pub const CPY_REG_REG: u8 = 0x10;
    pub const CPY_REG_VAL: u8 = 0x11;
    pub const CPY_A0_REG_REG: u8 = 0x12;
    pub const CPY_A0_ADDR: u8 = 0x13;
    pub const CPY_A1_REG_REG: u8 = 0x14;
    pub const CPY_A1_ADDR: u8 = 0x15;
    pub const LDA0_REG_REG: u8 = 0x16;
    pub const LDA1_REG_REG: u8 = 0x17;
    pub const SWPAR: u8 = 0x18;

    pub const JMP_ADDR: u8 = 0x20;
    pub const JMP_AREG: u8 = 0x21;
    pub const JE_ADDR: u8 = 0x22;
    pub const JE_AREG: u8 = 0x23;
    pub const JNE_ADDR: u8 = 0x24;
    pub const JNE_AREG: u8 = 0x25;
    pub const JL_ADDR: u8 = 0x26;
    pub const JL_AREG: u8 = 0x27;
    pub const JG_ADDR: u8 = 0x28;
    pub const JG_AREG: u8 = 0x29;
    pub const OVER_ADDR: u8 = 0x2A;
    pub const OVER_AREG: u8 = 0x2B;
    pub const NOVER_ADDR: u8 = 0x2C;
    pub const NOVER_AREG: u8 = 0x2D;

    pub const CMP_REG_REG: u8 = 0x30;
    pub const CMP_REG_VAL: u8 = 0x31;
    pub const CMPAR: u8 = 0x32;

    pub const MEMR_ADDR: u8 = 0x40;
    pub const MEMR_AREG: u8 = 0x41;
    pub const MEMW_ADDR: u8 = 0x42;
    pub const MEMW_AREG: u8 = 0x43;

    pub const CALL_ADDR: u8 = 0x70;
    pub const CALL_AREG: u8 = 0x71;
    pub const RET: u8 = 0x72;
    pub const PUSH_REG: u8 = 0x73;
    pub const PUSH_VAL: u8 = 0x74;
    pub const POP_REG: u8 = 0x75;
    pub const ARG_REG_VAL: u8 = 0x76;
    pub const ARG_REG_REG: u8 = 0x77;

    pub const PRT_REG: u8 = 0x90;
    pub const PRT_VAL: u8 = 0x91;
    pub const PRTLN: u8 = 0x92;
    pub const PRTD_STR: u8 = 0x93;
    pub const PRTC_REG: u8 = 0x94;
    pub const PRTC_VAL: u8 = 0x95;
    pub const PSTR_ADDR: u8 = 0x96;
    pub const PSTR_AREG: u8 = 0x97;

    pub const FOPEN: u8 = 0xC0;
    pub const FILER_ADDR: u8 = 0xC1;
    pub const FILER_AREG: u8 = 0xC2;
    pub const FILEW_ADDR: u8 = 0xC3;
    pub const FILEW_AREG: u8 = 0xC4;
    pub const FSEEK: u8 = 0xC5;
    pub const FSKIP_REG: u8 = 0xC6;
    pub const FSKIP_VAL: u8 = 0xC7;
    pub const FCHK_ADDR: u8 = 0xC8;
    pub const FCHK_AREG: u8 = 0xC9;

    pub const IPOLL_ADDR: u8 = 0xD0;
    pub const IPOLL_AREG: u8 = 0xD1;
    pub const RCHR_REG: u8 = 0xD2;
    pub const RSTR_ADDR: u8 = 0xD3;
    pub const RSTR_AREG: u8 = 0xD4;

    pub const NOP: u8 = 0xFE;
    pub const HALT: u8 = 0xFF;
}

pub fn get_byte_count(opcode: u8) -> usize {
    match opcode {
        FSEEK | PRTLN | FOPEN | SWPAR | CMPAR | RET | NOP | HALT => 1,
        INC_REG | DEC_REG | JMP_AREG | JE_AREG | JNE_AREG | JL_AREG | JG_AREG | OVER_AREG
        | NOVER_AREG | MEMR_AREG | MEMW_AREG | CALL_AREG | PUSH_REG | PUSH_VAL | POP_REG
        | PRT_REG | PRT_VAL | PRTC_REG | PRTC_VAL | FILER_AREG | FILEW_AREG | RCHR_REG => 2,
        ADD_REG_REG | ADD_REG_VAL | SUB_REG_REG | SUB_REG_VAL | CPY_REG_REG | CPY_REG_VAL
        | CPY_A0_REG_REG | CPY_A0_ADDR | CPY_A1_REG_REG | CPY_A1_ADDR | LDA0_REG_REG
        | LDA1_REG_REG | JMP_ADDR | JE_ADDR | JNE_ADDR | JL_ADDR | JG_ADDR | OVER_ADDR
        | NOVER_ADDR | CMP_REG_REG | CMP_REG_VAL | MEMR_ADDR | MEMW_ADDR | CALL_ADDR | PRTD_STR
        | FILER_ADDR | FILEW_ADDR | FSKIP_REG | ARG_REG_VAL | ARG_REG_REG | PSTR_ADDR
        | PSTR_AREG | FCHK_ADDR | FCHK_AREG | IPOLL_ADDR | IPOLL_AREG | RSTR_ADDR | RSTR_AREG => 3,
        _ => panic!("Unknown opcode: {}", opcode),
    }
}

pub fn is_jump_op(opcode: u8) -> bool {
    matches!(
        opcode,
        JMP_ADDR
            | JMP_AREG
            | JE_ADDR
            | JE_AREG
            | JL_ADDR
            | JL_AREG
            | JNE_ADDR
            | JNE_AREG
            | JG_ADDR
            | JG_AREG
            | OVER_ADDR
            | OVER_AREG
            | NOVER_ADDR
            | NOVER_AREG
            | CALL_ADDR
            | CALL_AREG
            | RET
            | FCHK_AREG
            | FCHK_ADDR
            | IPOLL_AREG
            | IPOLL_ADDR
    )
}

#[cfg(test)]
mod tests {
    use crate::constants::code::*;
    use crate::constants::get_byte_count;
    use std::collections::HashSet;

    const ALL_OPS: [u8; 66] = [
        ADD_REG_REG,
        ADD_REG_VAL,
        SUB_REG_REG,
        SUB_REG_VAL,
        INC_REG,
        DEC_REG,
        CPY_REG_REG,
        CPY_REG_VAL,
        CPY_A0_REG_REG,
        CPY_A0_ADDR,
        CPY_A1_REG_REG,
        CPY_A1_ADDR,
        LDA0_REG_REG,
        LDA1_REG_REG,
        SWPAR,
        JMP_ADDR,
        JMP_AREG,
        JE_ADDR,
        JE_AREG,
        JNE_ADDR,
        JNE_AREG,
        JL_ADDR,
        JL_AREG,
        JG_ADDR,
        JG_AREG,
        OVER_ADDR,
        OVER_AREG,
        NOVER_ADDR,
        NOVER_AREG,
        CMP_REG_REG,
        CMP_REG_VAL,
        CMPAR,
        MEMR_ADDR,
        MEMR_AREG,
        MEMW_ADDR,
        MEMW_AREG,
        CALL_ADDR,
        CALL_AREG,
        RET,
        PUSH_REG,
        PUSH_VAL,
        POP_REG,
        PRT_REG,
        PRT_VAL,
        PRTLN,
        PRTD_STR,
        PRTC_REG,
        PRTC_VAL,
        FOPEN,
        FILER_ADDR,
        FILER_AREG,
        FILEW_ADDR,
        FILEW_AREG,
        FSEEK,
        FSKIP_REG,
        NOP,
        HALT,
        ARG_REG_VAL,
        ARG_REG_REG,
        FCHK_ADDR,
        FCHK_AREG,
        IPOLL_ADDR,
        IPOLL_AREG,
        RCHR_REG,
        RSTR_AREG,
        RSTR_ADDR,
    ];

    #[test]
    fn check_ops_are_unique() {
        let mut found = HashSet::new();
        for op in ALL_OPS.iter() {
            if found.contains(op) {
                panic!("Found duplicate: {}", op);
            }
            found.insert(op);
        }
    }

    #[test]
    fn check_ops_have_byte_counts() {
        for op in ALL_OPS.iter() {
            let count = get_byte_count(*op);
            if count > 3 {
                panic!("Invalid byte count for {}: {}", op, count);
            }
        }
    }
}
