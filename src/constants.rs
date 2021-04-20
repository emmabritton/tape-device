pub mod hardware {
    pub const RAM_SIZE: usize = 0xFFFF;
    pub const DATA_REG_SIZE: usize = 4;
    pub const ADDR_REG_SIZE: usize = 2;

    pub const REG_ACC: u8 = 0x01;
    pub const REG_D0: u8 = 0x02;
    pub const REG_D1: u8 = 0x03;
    pub const REG_D2: u8 = 0x04;
    pub const REG_D3: u8 = 0x05;
    pub const REG_A0: u8 = 0x06;
    pub const REG_A1: u8 = 0x07;
}

pub mod compare {
    pub const EQUAL: u8 = 0;
    pub const LESSER: u8 = 1;
    pub const GREATER: u8 = 2;
}

pub mod system {
    pub const TAPE_HEADER_1: u8 = 0xFD;
    pub const TAPE_HEADER_2: u8 = 0xA0;

    pub const PRG_VERSION: u8 = 8;

    pub const MAX_PRG_SIZE: u16 = 21845;
}

pub mod code {
    pub const OP_ADD_REG_REG: u8 = 0x04;
    pub const OP_ADD_REG_VAL: u8 = 0x01;
    pub const OP_SUB_REG_REG: u8 = 0x02;
    pub const OP_SUB_REG_VAL: u8 = 0x03;

    pub const OP_INC: u8 = 0x05;
    pub const OP_DEC: u8 = 0x06;

    pub const OP_COPY_REG_VAL: u8 = 0x10;
    pub const OP_MEM_READ: u8 = 0x11;
    pub const OP_COPY_REG_REG: u8 = 0x12;
    pub const OP_MEM_WRITE: u8 = 0x13;
    pub const OP_MEM_READ_REG: u8 = 0x14;
    pub const OP_MEM_WRITE_REG: u8 = 0x15;

    pub const OP_JMP: u8 = 0x20;
    //JMP is last CMP equal
    pub const OP_JE: u8 = 0x21;
    //JMP is last CMP not equal
    pub const OP_JNE: u8 = 0x22;
    //JMP is last CMP less than
    pub const OP_JL: u8 = 0x23;
    //JMP is last CMP greater than
    pub const OP_JG: u8 = 0x24;
    pub const OP_CMP_REG_REG: u8 = 0x2A;
    pub const OP_CMP_REG_VAL: u8 = 0x2B;
    pub const OP_JMP_REG: u8 = 0x25;
    //JMP is last CMP equal
    pub const OP_JE_REG: u8 = 0x26;
    //JMP is last CMP not equal
    pub const OP_JNE_REG: u8 = 0x27;
    //JMP is last CMP less than
    pub const OP_JL_REG: u8 = 0x28;
    //JMP is last CMP greater than
    pub const OP_JG_REG: u8 = 0x29;

    //Reads and prints [ACC] bytes as chars from tape to @ADDR
    pub const OP_PRINT_DAT: u8 = 0x30;
    //Print number
    pub const OP_PRINT_REG: u8 = 0x31;
    //Print number
    pub const OP_PRINT_VAL: u8 = 0x32;
    pub const OP_PRINT_LN: u8 = 0x33;
    //Reads and prints [ACC] bytes as chars from mem at @ADDR
    pub const OP_PRINT_MEM: u8 = 0x34;
    //Reads and prints [ACC] bytes as chars from mem at REG
    pub const OP_PRINT_MEM_REG: u8 = 0x35;

    //Open file, loads length into [D0][D1][D2][D3]
    //If the length is 1023 bytes then regs set to [00][00][03][FF]
    pub const OP_OPEN_FILE: u8 = 0x40;
    //Reads [ACC] bytes from the file to @ADDR
    //Sets [ACC] to read byte count
    pub const OP_READ_FILE: u8 = 0x41;
    //Skips [REG] bytes from the file
    //Sets [ACC] to skipped byte count
    pub const OP_SKIP_FILE: u8 = 0x42;
    //Moves cursor to [D0][D1][D2][D3] bytes in file
    pub const OP_SEEK_FILE: u8 = 0x43;
    //Reads [ACC] bytes from memory starting at @ADDR
    //Sets [ACC] to written byte count
    pub const OP_WRITE_FILE: u8 = 0x44;
    //Reads [ACC] bytes from the file to REG
    //Sets [ACC] to read byte count
    pub const OP_READ_FILE_REG: u8 = 0x45;
    //Reads [ACC] bytes from memory starting at REG
    //Sets [ACC] to written byte count
    pub const OP_WRITE_FILE_REG: u8 = 0x46;

    pub const OP_OVERFLOW: u8 = 0x50;
    pub const OP_NOT_OVERFLOW: u8 = 0x51;
    pub const OP_OVERFLOW_REG: u8 = 0x52;
    pub const OP_NOT_OVERFLOW_REG: u8 = 0x53;

    pub const OP_LOAD_ADDR_HIGH: u8 = 0x60;
    pub const OP_LOAD_ADDR_LOW: u8 = 0x61;
    pub const OP_LOAD_ADDR_HIGH_VAL: u8 = 0x62;
    pub const OP_LOAD_ADDR_LOW_VAL: u8 = 0x63;

    pub const OP_CALL_ADDR: u8 = 0x70;
    pub const OP_RETURN: u8 = 0x71;
    pub const OP_PUSH_REG: u8 = 0x72;
    pub const OP_POP_REG: u8 = 0x73;
    pub const OP_PUSH_VAL: u8 = 0x74;
    pub const OP_CALL_REG: u8 = 0x75;

    //No op
    //Used to mark empty lines/comments as PC = line num
    pub const OP_NOP: u8 = 0xFF;

    pub const OP_HALT: u8 = 0xFE;

    pub const ALIGNMENT_PADDING: u8 = 0xF0;
}
