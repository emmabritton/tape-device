use crate::constants::code::*;
use crate::language::parser::ops::Op;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref OPS: [Op; 37] = [
        //CPY reg reg, reg val, areg areg, areg label|addr, areg reg reg, reg reg areg
        //Copy value from 2nd param to 1st
        Op::new_reg_complex("CPY", CPY_REG_REG, CPY_REG_VAL, CPY_AREG_AREG, CPY_AREG_ADDR, CPY_AREG_REG_REG, CPY_REG_REG_AREG),
        //ADD reg reg|val
        //Add 1st and 2nd params and store in ACC
        Op::new_reg_val("ADD", ADD_REG_REG, ADD_REG_VAL),
        //ADD reg reg|val
        //Subtract 2nd param from 1st and store in ACC
        Op::new_reg_val("SUB", SUB_REG_REG, SUB_REG_VAL),
        //CMP reg reg, reg val, areg areg, areg label|addr, areg reg reg, reg reg areg
        //Compare values in 1st and 2nd params, store result in ACC (0 = Equal, 1 = Lesser, 2 = Greater)
        Op::new_reg_complex("CMP", CMP_REG_REG, CMP_REG_VAL, CMP_AREG_AREG, CMP_AREG_ADDR, CMP_AREG_REG_REG, CMP_REG_REG_AREG),
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
        //FOPEN
        //Opens supplied input/data file or crashes, saves length to [D0][D1][D2][D3]
        Op::new_none("FOPEN", FOPEN),
        //NOP
        //Does nothing
        Op::new_none("NOP", NOP),
        //RET
        //Return from subroutine
        Op::new_none("RET", RET),
        //FSEEK
        //Move file cursor to [D0][D1][D2][D3]
        Op::new_none("FSEEK", FSEEK),
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
        //FILER addr|addr_reg
        //Read ACC bytes from file cursor and write to 1st param in memory, sets read byte count in ACC
        Op::new_mem("FILER", FILER_ADDR, FILER_AREG),
        //FILEW addr|addr_reg
        //Write ACC bytes starting at 1st param in memory to file cursor, sets written byte count in ACC
        Op::new_mem("FILEW", FILEW_ADDR, FILEW_AREG),
        //PRT reg|val
        //Prints value in 1st param
        Op::new_regval("PRT", PRT_REG, PRT_VAL),
        //PRTLN
        //Prints new line
        Op::new_none("PRTLN", PRTLN),
        //PRTC reg|val
        //Prints value in 1st param as ASCII
        Op::new_regval("PRTC", PRTC_REG, PRTC_VAL),
        //FSKIP reg|val
        //Move file cursor forward by number of bytes set by 1st param
        Op::new_regval("FSKIP", FSKIP_REG, FSKIP_VAL),
        //PRTD key
        //Prints string named by 1st param
        Op::new_string("PRTD", PRTD_STR),
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
        //FCHK addr_reg|addr
        //Jump to 1st param if input file is available
        Op::new_jmp("FCHK", FCHK_ADDR, FCHK_AREG),
        //PSTR addr_reg|addr
        //Print ACC chars from 1st param in memory
        Op::new_mem("PSTR", PSTR_ADDR, PSTR_AREG),
        //RSTR addr_reg|addr
        //Read up to chars keyboard (until return is pressed or 255 entered) starting at 1st param in memory
        Op::new_mem("RSTR", RSTR_ADDR, RSTR_AREG),
        //RCHR reg
        //Read one char from keyboard into 1st param
        Op::new_single_reg("RCHR", RCHR_REG),
    ];
}
