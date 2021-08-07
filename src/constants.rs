use crate::constants::code::*;

pub mod hardware {
    pub const RAM_SIZE: usize = 0xFFFF;
    pub const DATA_REG_COUNT: usize = 4;
    pub const ADDR_REG_COUNT: usize = 2;
    pub const MAX_DATA_ARRAY_LEN: usize = 255;
    pub const MAX_DATA_ARRAY_COUNT: usize = 254;
    pub const MAX_STRING_LEN: usize = 255;
    pub const MAX_STRING_BYTES: usize = 65535;
    pub const MAX_DATA_BYTES: usize = 65535;

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

    pub const PRG_VERSION: u8 = 1;
}

pub mod code {
    pub const DIVDERS: [&str; 3] = [".data", ".strings", ".ops"];
    pub const KEYWORDS: [&str; 1] = ["const"];
    pub const MNEMONICS: [&str; 47] = [
        "add", "sub", "inc", "dec", "cmp", "cpy", "swp", "jmp", "je", "jg", "jl", "jne", "over",
        "nover", "memr", "memw", "memp", "ld", "call", "ret", "push", "pop", "arg", "prt", "prtc",
        "prtln", "prtd", "prts", "and", "or", "xor", "not", "fchk", "fopen", "fseek", "fskip",
        "filew", "filer", "ipoll", "rchr", "rstr", "time", "rand", "seed", "debug", "halt", "nop",
    ];
    pub const REGISTERS: [&str; 7] = ["d0", "d1", "d2", "d3", "acc", "a0", "a1"];

    pub const ADD_REG_REG: u8 = 0x01;
    pub const ADD_REG_VAL: u8 = 0x02;
    pub const SUB_REG_REG: u8 = 0x03;
    pub const SUB_REG_VAL: u8 = 0x04;
    pub const INC_REG: u8 = 0x05;
    pub const DEC_REG: u8 = 0x06;
    pub const ADD_REG_AREG: u8 = 0x07;
    pub const SUB_REG_AREG: u8 = 0x08;

    pub const CPY_REG_REG: u8 = 0x10;
    pub const CPY_REG_VAL: u8 = 0x11;
    pub const CPY_AREG_REG_REG: u8 = 0x12;
    pub const CPY_AREG_ADDR: u8 = 0x13;
    pub const CPY_REG_REG_AREG: u8 = 0x14;
    pub const CPY_AREG_AREG: u8 = 0x15;
    pub const SWP_REG_REG: u8 = 0x16;
    pub const SWP_AREG_AREG: u8 = 0x17;
    pub const CPY_REG_AREG: u8 = 0x18;

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
    pub const CMP_AREG_AREG: u8 = 0x32;
    pub const CMP_AREG_ADDR: u8 = 0x33;
    pub const CMP_REG_REG_AREG: u8 = 0x34;
    pub const CMP_AREG_REG_REG: u8 = 0x35;
    pub const CMP_REG_AREG: u8 = 0x36;

    pub const MEMR_ADDR: u8 = 0x40;
    pub const MEMR_AREG: u8 = 0x41;
    pub const MEMW_ADDR: u8 = 0x42;
    pub const MEMW_AREG: u8 = 0x43;
    pub const LD_AREG_DATA_REG_REG: u8 = 0x44;
    pub const LD_AREG_DATA_REG_VAL: u8 = 0x45;
    pub const LD_AREG_DATA_VAL_REG: u8 = 0x46;
    pub const LD_AREG_DATA_VAL_VAL: u8 = 0x47;

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
    pub const PRTS_STR: u8 = 0x93;
    pub const PRTC_REG: u8 = 0x94;
    pub const PRTC_VAL: u8 = 0x95;
    pub const MEMP_ADDR: u8 = 0x96;
    pub const MEMP_AREG: u8 = 0x97;
    pub const PRTD_AREG: u8 = 0x98;
    pub const PRT_AREG: u8 = 0x99;
    pub const PRTC_AREG: u8 = 0x9A;

    pub const AND_REG_REG: u8 = 0xA0;
    pub const AND_REG_VAL: u8 = 0xA1;
    pub const OR_REG_REG: u8 = 0xA2;
    pub const OR_REG_VAL: u8 = 0xA3;
    pub const XOR_REG_REG: u8 = 0xA4;
    pub const XOR_REG_VAL: u8 = 0xA5;
    pub const NOT_REG: u8 = 0xA6;
    pub const AND_REG_AREG: u8 = 0xA7;
    pub const OR_REG_AREG: u8 = 0xA8;
    pub const XOR_REG_AREG: u8 = 0xA9;

    pub const FOPEN_REG: u8 = 0xC0;
    pub const FILER_REG_ADDR: u8 = 0xC1;
    pub const FILER_REG_AREG: u8 = 0xC2;
    pub const FILEW_REG_ADDR: u8 = 0xC3;
    pub const FILEW_REG_AREG: u8 = 0xC4;
    pub const FSEEK_REG: u8 = 0xC5;
    pub const FSKIP_REG_REG: u8 = 0xC6;
    pub const FSKIP_REG_VAL: u8 = 0xC7;
    pub const FCHK_REG_ADDR: u8 = 0xC8;
    pub const FCHK_REG_AREG: u8 = 0xC9;
    pub const FOPEN_VAL: u8 = 0xCA;
    pub const FILER_VAL_ADDR: u8 = 0xCB;
    pub const FILER_VAL_AREG: u8 = 0xCC;
    pub const FILEW_VAL_ADDR: u8 = 0xCD;
    pub const FILEW_VAL_AREG: u8 = 0xCE;
    pub const FSEEK_VAL: u8 = 0xCF;
    pub const FSKIP_VAL_REG: u8 = 0xD0;
    pub const FSKIP_VAL_VAL: u8 = 0xD1;
    pub const FCHK_VAL_ADDR: u8 = 0xD2;
    pub const FCHK_VAL_AREG: u8 = 0xD3;
    pub const FILEW_REG_REG: u8 = 0xD4;
    pub const FILEW_REG_VAL: u8 = 0xD5;
    pub const FILEW_VAL_REG: u8 = 0xD6;
    pub const FILEW_VAL_VAL: u8 = 0xD7;

    pub const IPOLL_ADDR: u8 = 0xE0;
    pub const IPOLL_AREG: u8 = 0xE1;
    pub const RCHR_REG: u8 = 0xE2;
    pub const RSTR_ADDR: u8 = 0xE3;
    pub const RSTR_AREG: u8 = 0xE4;
    pub const RAND_REG: u8 = 0xE5;
    pub const TIME: u8 = 0xE6;
    pub const SEED_REG: u8 = 0xE7;

    pub const DEBUG: u8 = 0xFD;
    pub const NOP: u8 = 0xFE;
    pub const HALT: u8 = 0xFF;
}

pub fn get_byte_count(opcode: u8) -> usize {
    match opcode {
        PRTLN | RET | NOP | HALT | TIME | DEBUG => 1,
        INC_REG | DEC_REG | JMP_AREG | JE_AREG | JNE_AREG | JL_AREG | JG_AREG | OVER_AREG
        | NOVER_AREG | MEMR_AREG | MEMW_AREG | CALL_AREG | PUSH_REG | PUSH_VAL | POP_REG
        | PRT_REG | PRT_VAL | PRTC_REG | PRTC_VAL | RCHR_REG | RAND_REG | NOT_REG | SEED_REG
        | FSEEK_REG | FSEEK_VAL | FOPEN_REG | FOPEN_VAL | PRTD_AREG | MEMP_AREG | PRT_AREG
        | PRTC_AREG | RSTR_AREG | IPOLL_AREG => 2,
        ADD_REG_REG | ADD_REG_VAL | SUB_REG_REG | SUB_REG_VAL | CPY_REG_REG | CPY_REG_VAL
        | SWP_AREG_AREG | SWP_REG_REG | JMP_ADDR | JE_ADDR | JNE_ADDR | JL_ADDR | JG_ADDR
        | OVER_ADDR | CMP_AREG_AREG | CPY_AREG_AREG | NOVER_ADDR | CMP_REG_REG | CMP_REG_VAL
        | MEMR_ADDR | MEMW_ADDR | CALL_ADDR | PRTS_STR | FSKIP_REG_REG | FSKIP_REG_VAL
        | FSKIP_VAL_REG | FSKIP_VAL_VAL | ARG_REG_VAL | ARG_REG_REG | MEMP_ADDR
        | FILER_REG_AREG | FILER_VAL_AREG | FILEW_REG_AREG | FILEW_VAL_AREG | IPOLL_ADDR
        | RSTR_ADDR | AND_REG_VAL | AND_REG_REG | AND_REG_AREG | OR_REG_AREG | XOR_REG_AREG
        | OR_REG_VAL | OR_REG_REG | XOR_REG_REG | XOR_REG_VAL | FCHK_REG_AREG | FCHK_VAL_AREG
        | ADD_REG_AREG | SUB_REG_AREG | CPY_REG_AREG | CMP_REG_AREG | FILEW_REG_REG
        | FILEW_REG_VAL | FILEW_VAL_REG | FILEW_VAL_VAL => 3,
        CMP_AREG_ADDR | CPY_AREG_ADDR | CMP_AREG_REG_REG | CMP_REG_REG_AREG | CPY_REG_REG_AREG
        | FCHK_REG_ADDR | FCHK_VAL_ADDR | CPY_AREG_REG_REG | FILER_REG_ADDR | FILEW_VAL_ADDR
        | FILER_VAL_ADDR | FILEW_REG_ADDR => 4,
        LD_AREG_DATA_REG_REG | LD_AREG_DATA_REG_VAL | LD_AREG_DATA_VAL_REG
        | LD_AREG_DATA_VAL_VAL => 6,
        _ => panic!("Unknown opcode: {:02X}", opcode),
    }
}

pub fn get_addr_byte_offset(opcode: u8) -> Option<usize> {
    match opcode {
        JMP_ADDR | JE_ADDR | JL_ADDR | JNE_ADDR | RSTR_ADDR | JG_ADDR | OVER_ADDR | NOVER_ADDR
        | CALL_ADDR | MEMR_ADDR | MEMW_ADDR | IPOLL_ADDR | PRTS_STR | MEMP_ADDR => Some(1),
        FCHK_VAL_ADDR | FCHK_REG_ADDR | LD_AREG_DATA_VAL_VAL | CPY_AREG_ADDR | CMP_AREG_ADDR
        | FILEW_VAL_ADDR | FILER_VAL_ADDR | FILER_REG_ADDR | LD_AREG_DATA_VAL_REG
        | LD_AREG_DATA_REG_REG | LD_AREG_DATA_REG_VAL => Some(2),
        _ => None,
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
            | FCHK_VAL_AREG
            | FCHK_VAL_ADDR
            | FCHK_REG_AREG
            | FCHK_REG_ADDR
            | IPOLL_AREG
            | IPOLL_ADDR
    )
}

#[rustfmt::skip]
#[allow(dead_code)]
pub const ALL_OPS: [u8; 108] = [
    ADD_REG_REG, ADD_REG_VAL, ADD_REG_AREG,
    SUB_REG_REG, SUB_REG_VAL, SUB_REG_AREG,
    AND_REG_REG, AND_REG_VAL, AND_REG_AREG,
    OR_REG_REG, OR_REG_VAL, OR_REG_AREG,
    XOR_REG_REG, XOR_REG_VAL, XOR_REG_AREG,
    INC_REG, DEC_REG,
    CPY_REG_REG,
    CPY_REG_VAL,
    CPY_AREG_AREG,
    CPY_AREG_ADDR,
    CPY_AREG_REG_REG,
    CPY_REG_REG_AREG,
    CPY_REG_AREG,
    CMP_AREG_AREG,
    CMP_AREG_ADDR,
    CMP_REG_REG_AREG,
    CMP_REG_REG,
    CMP_AREG_REG_REG,
    CMP_REG_VAL,
    CMP_REG_AREG,
    JMP_ADDR, JMP_AREG,
    JE_ADDR, JE_AREG,
    JNE_ADDR, JNE_AREG,
    JL_ADDR, JL_AREG,
    JG_ADDR, JG_AREG,
    OVER_ADDR, OVER_AREG,
    NOVER_ADDR, NOVER_AREG,
    MEMR_ADDR, MEMR_AREG,
    MEMW_ADDR, MEMW_AREG,
    CALL_ADDR, CALL_AREG,
    RET,
    PUSH_REG, PUSH_VAL,
    POP_REG,
    PRT_REG, PRT_VAL, PRT_AREG,
    PRTLN,
    PRTC_REG, PRTC_VAL, PRTC_AREG,
    FOPEN_VAL, FOPEN_REG,
    FILER_REG_ADDR, FILER_REG_AREG,
    FILEW_REG_ADDR, FILEW_REG_AREG,
    FSEEK_REG, FSEEK_VAL,
    FSKIP_VAL_REG, FSKIP_REG_REG,
    FCHK_REG_ADDR, FCHK_REG_AREG,
    FCHK_VAL_ADDR, FCHK_VAL_AREG,
    FILER_VAL_ADDR, FILER_VAL_AREG,
    FILEW_VAL_ADDR, FILEW_VAL_AREG,
    NOP,
    HALT,
    ARG_REG_VAL, ARG_REG_REG,
    IPOLL_ADDR, IPOLL_AREG,
    RCHR_REG,
    RSTR_AREG, RSTR_ADDR,
    SWP_REG_REG, SWP_AREG_AREG,
    TIME,
    RAND_REG,
    SEED_REG,
    NOT_REG,
    LD_AREG_DATA_REG_REG,
    LD_AREG_DATA_REG_VAL,
    LD_AREG_DATA_VAL_REG,
    LD_AREG_DATA_VAL_VAL,
    MEMP_ADDR, MEMP_AREG,
    PRTD_AREG,
    PRTS_STR,
    DEBUG,
    FILEW_REG_REG, FILEW_REG_VAL, FILEW_VAL_REG, FILEW_VAL_VAL
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

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
            get_byte_count(*op);
        }
    }

    #[test]
    fn check_jump_ops_have_addr_offsets() {
        for op in ALL_OPS.iter() {
            if is_jump_op(*op) {
                get_addr_byte_offset(*op);
            }
        }
    }
}
