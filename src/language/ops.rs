use crate::constants::code::*;
use crate::language::parser::ops::Op;
use lazy_static::lazy_static;

//To add new operation the following files must be updated:
//language/ops.rs
//constants.rs
//decompiler/mod.rs
//device/internal.rs
//language.md

lazy_static! {
    pub static ref OPS: [Op; 48] = [
        //CPY reg reg, reg val, areg areg, areg label|addr, areg reg reg, reg reg areg, reg areg
        //Copy value from 2nd param to 1st
        Op::new_reg_complex("CPY", CPY_REG_REG, CPY_REG_VAL, CPY_AREG_AREG, CPY_AREG_ADDR, CPY_AREG_REG_REG, CPY_REG_REG_AREG, CPY_REG_AREG),
        //ADD reg reg|val|addr_reg
        //Add 1st and 2nd params and store in ACC (addr_reg must point to data)
        Op::new_reg_val("ADD", ADD_REG_REG, ADD_REG_VAL, ADD_REG_AREG),
        //ADD reg reg|val|addr_reg
        //Subtract 2nd param from 1st and store in ACC (addr_reg must point to data)
        Op::new_reg_val("SUB", SUB_REG_REG, SUB_REG_VAL, SUB_REG_AREG),
        //CMP reg reg, reg val, areg areg, areg label|addr, areg reg reg, reg reg areg, reg areg
        //Compare values in 1st and 2nd params, store result in ACC (0 = Equal, 1 = Lesser, 2 = Greater)
        Op::new_reg_complex("CMP", CMP_REG_REG, CMP_REG_VAL, CMP_AREG_AREG, CMP_AREG_ADDR, CMP_AREG_REG_REG, CMP_REG_REG_AREG, CMP_REG_AREG),
        //LD areg data_key (reg reg, reg val, val reg, val val)
        //Load address of indexed data (params 2 to 4) into 1st param
        Op::new_data("LD", LD_AREG_DATA_REG_REG, LD_AREG_DATA_REG_VAL, LD_AREG_DATA_VAL_REG, LD_AREG_DATA_VAL_VAL),
        //JMP addr|lbl|addr_reg
        //Jump to instruction at 1st param
        Op::new_jmp("JMP", JMP_ADDR, JMP_AREG),
        //JE addr|lbl|addr_reg
        //Jump to instruction at 1st param if ACC == 0 (Equal)
        Op::new_jmp("JE", JE_ADDR, JE_AREG),
        //JNE addr|lbl|addr_reg
        //Jump to instruction at 1st param if ACC != 0 (Equal)
        Op::new_jmp("JNE", JNE_ADDR, JNE_AREG),
        //JG addr|lbl|addr_reg
        //Jump to instruction at 1st param if ACC == 2 (Greater)
        Op::new_jmp("JG", JG_ADDR, JG_AREG),
        //JL addr|lbl|addr_reg
        //Jump to instruction at 1st param if ACC == 1 (Lesser)
        Op::new_jmp("JL", JL_ADDR, JL_AREG),
        //OVER addr|lbl|addr_reg
        //Jump to instruction at 1st param if overflow flag is set
        Op::new_jmp("OVER", OVER_ADDR, OVER_AREG),
        //NOVER addr|lbl|addr_reg
        //Jump to instruction at 1st param if overflow flag is not set
        Op::new_jmp("NOVER", NOVER_ADDR, NOVER_AREG),
        //HALT
        //Stop program execution
        Op::new_none("HALT", HALT),
        //FOPEN reg|val
        //Opens input/data file <num> or crashes, saves length to [D0][D1][D2][D3]
        Op::new_regval("FOPEN", FOPEN_REG, FOPEN_VAL),
        //NOP
        //Does nothing
        Op::new_none("NOP", NOP),
        //RET
        //Return from subroutine
        Op::new_none("RET", RET),
        //FSEEK reg|val
        //Move file <num> cursor to [D0][D1][D2][D3]
        Op::new_regval("FSEEK", FSEEK_REG, FSEEK_VAL),
        //SWP reg reg, areg areg
        //Swaps contents of 1st param and 2nd param
        Op::new_either_reg_reg("SWP", SWP_REG_REG, SWP_AREG_AREG),
        //INC reg|addr_reg
        //Increment 1st param by 1
        Op::new_single_reg("INC", INC_REG),
        //DEC reg|addr_reg
        //Decrement 1st param by 1
        Op::new_single_reg("DEC", DEC_REG),
        //MEMR addr|addr_reg
        //Read byte at 1st param in memory and store in ACC
        Op::new_mem("MEMR", MEMR_ADDR, MEMR_AREG),
        //MEMW addr|addr_reg
        //Write byte in ACC and write to byte at 1st param in memory
        Op::new_mem("MEMW", MEMW_ADDR, MEMW_AREG),
        //FILER reg|val addr|addr_reg
        //Read ACC bytes from file <num> cursor and write to 1st param in memory, sets read byte count in ACC
        Op::new_file_mem("FILER", FILER_REG_ADDR, FILER_REG_AREG, FILER_VAL_ADDR, FILER_VAL_AREG),
        //FILEW reg|val addr|addr_reg
        //Write ACC bytes starting at 1st param in memory to file <num> cursor, sets written byte count in ACC
        Op::new_file_mem_value("FILEW", FILEW_REG_ADDR, FILEW_REG_AREG, FILEW_VAL_ADDR, FILEW_VAL_AREG, FILEW_REG_REG, FILEW_REG_VAL, FILEW_VAL_REG, FILEW_VAL_VAL),
        //PRT reg|val|addr_reg
        //Prints value in 1st param (addr_reg must point to data)
        Op::new_regvaldata("PRT", PRT_REG, PRT_VAL, PRT_AREG),
        //PRTLN
        //Prints new line
        Op::new_none("PRTLN", PRTLN),
        //PRTC reg|val|addr_reg
        //Prints value in 1st param as ASCII (addr_reg must point to data)
        Op::new_regvaldata("PRTC", PRTC_REG, PRTC_VAL, PRTC_AREG),
        //FSKIP reg|val reg|val
        //Move file <num> cursor forward by number of bytes set by 1st param
        Op::new_regval_regval("FSKIP", FSKIP_REG_REG, FSKIP_REG_VAL, FSKIP_VAL_REG, FSKIP_VAL_VAL),
        //PRTS key
        //Prints string named by 1st param
        Op::new_string("PRTS", PRTS_STR),
        //PRTD addr_reg
        //Prints ACC bytes from data starting at by 1st param
        Op::new_areg("PRTD", PRTD_AREG),
        //CALL addr|lbl|addr_reg
        //Jump to 1st param, setup stack to allow RET
        Op::new_jmp("CALL", CALL_ADDR, CALL_AREG),
        //PUSH addr_reg|reg|val
        //Push 1st param in to stack
        Op::new_addrregval("PUSH", PUSH_REG, PUSH_VAL),
        //POP addr_reg|reg
        //Pop value from stack to 1st param
        Op::new_single_reg("POP", POP_REG),
        //ARG addr_reg|reg reg|val
        //Read from value from stack 2nd param bytes before the FP and save to 1st param
        Op::new_addrreg_regval("ARG", ARG_REG_REG, ARG_REG_VAL),
        //IPOLL addr_reg|addr
        //Jump to 1st param if at least one char can be read from keyboard
        Op::new_jmp("IPOLL", IPOLL_ADDR, IPOLL_AREG),
        //FCHK reg|val addr_reg|addr
        //Jump to 1st param if input file <num> is available
        Op::new_regval_jmp("FCHK", FCHK_REG_ADDR, FCHK_REG_AREG, FCHK_VAL_ADDR, FCHK_VAL_AREG),
        //MEMP addr_reg|addr
        //Print ACC chars from 1st param in memory or data
        Op::new_mem("MEMP", MEMP_ADDR, MEMP_AREG),
        //MEMC addr_reg addr_reg
        //Copy ACC bytes from 1st param in data to 2nd param in memory
        Op::new_areg_areg("MEMC", MEMC_AREG_AREG),
        //RSTR addr_reg|addr
        //Read up to chars keyboard (until return is pressed or 255 entered) starting at 1st param in memory
        Op::new_mem("RSTR", RSTR_ADDR, RSTR_AREG),
        //RCHR reg
        //Read one char from keyboard into 1st param
        Op::new_single_reg("RCHR", RCHR_REG),
        //RAND reg
        //Generate a pseudorandom number and put in 1st param
        Op::new_single_reg("RAND", RAND_REG),
        //SEED reg
        //Set the seed for the rng
        Op::new_single_reg("SEED", SEED_REG),
        //TIME
        //Populates D0 with seconds, D1 with minutes, D2 with hours
        Op::new_none("TIME", TIME),
        //AND reg reg|val|addr_reg
        //and bits of 1st and 2nd params and store in ACC (addr_reg must point to data)
        Op::new_reg_val("AND", AND_REG_REG, AND_REG_VAL, AND_REG_AREG),
        //OR reg reg|val|addr_reg
        //or bits of 1st and 2nd params and store in ACC (addr_reg must point to data)
        Op::new_reg_val("OR", OR_REG_REG, OR_REG_VAL, OR_REG_AREG),
        //XOR reg reg|val|addr_reg
        //xor bits of 1st and 2nd params and store in ACC (addr_reg must point to data)
        Op::new_reg_val("XOR", XOR_REG_REG, XOR_REG_VAL, XOR_REG_AREG),
        //NOT reg|addr_reg
        //not bits of 1st param (addr_reg must point to data)
        Op::new_single_reg("NOT", NOT_REG),
        //DEBUG
        //Prints dump from system
        Op::new_none("DEBUG", DEBUG),
    ];
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_for_missing_ops() {
        assert_eq!(MNEMONICS.len(), OPS.len())
    }
}
