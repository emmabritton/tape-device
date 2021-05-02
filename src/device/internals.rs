use crate::constants::code::*;
use crate::constants::hardware::*;
use crate::constants::{compare, get_byte_count, is_jump_op};
use crate::device::internals::RunResult::{Breakpoint, EoF, ProgError};
use crate::printer::{Printer, RcBox};
use anyhow::{Error, Result};
use std::cmp::Ordering;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

const SP_MAX: u16 = u16::MAX;

pub struct Device {
    mem: [u8; RAM_SIZE],
    tape_ops: Vec<u8>,
    tape_data: Vec<u8>,
    input_data: Option<String>,
    flags: Flags,
    pc: u16,
    acc: u8,
    sp: u16,
    fp: u16,
    data_reg: [u8; DATA_REG_SIZE],
    addr_reg: [u16; ADDR_REG_SIZE],
    file: Option<File>,
    breakpoints: Vec<u16>,
    printer: RcBox<dyn Printer>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Dump {
    pub pc: u16,
    pub acc: u8,
    pub sp: u16,
    pub fp: u16,
    pub data_reg: [u8; DATA_REG_SIZE],
    pub addr_reg: [u16; ADDR_REG_SIZE],
    pub overflow: bool,
}

impl Default for Dump {
    fn default() -> Self {
        Dump {
            pc: 0,
            acc: 0,
            sp: SP_MAX,
            fp: SP_MAX,
            data_reg: [0, 0, 0, 0],
            addr_reg: [0, 0],
            overflow: false,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum RunResult {
    ///Breakpoint hit
    Breakpoint,
    ///End of program
    EoF,
    ///Program error or HALT
    ProgError,
}

impl Device {
    pub fn new(
        ops: Vec<u8>,
        data: Vec<u8>,
        input_file: Option<String>,
        printer: RcBox<dyn Printer>,
    ) -> Self {
        Device {
            mem: [0; RAM_SIZE],
            flags: Flags::default(),
            acc: 0,
            data_reg: [0; DATA_REG_SIZE],
            addr_reg: [0; ADDR_REG_SIZE],
            pc: 0,
            sp: SP_MAX,
            fp: SP_MAX,
            breakpoints: vec![],
            tape_ops: ops,
            tape_data: data,
            input_data: input_file,
            file: None,
            printer,
        }
    }
}

#[derive(Debug, Default)]
pub struct Flags {
    overflow: bool,
}

impl Device {
    pub fn run(&mut self) -> RunResult {
        loop {
            if self.breakpoints.contains(&self.pc) {
                return Breakpoint;
            }
            if self.pc as usize >= self.tape_ops.len() {
                return EoF;
            }
            if !self.step() {
                return ProgError;
            }
        }
    }

    ///Execute next instruction
    ///Returns false if an error occurs or HALT is found, true otherwise
    pub fn step(&mut self) -> bool {
        if self.pc as usize >= self.tape_ops.len() {
            panic!("Tried to execute EoF")
        }
        self.execute()
    }

    fn log(&mut self, msg: &str) {
        self.printer.borrow_mut().print(msg);
    }

    fn elog(&mut self, msg: &str) {
        self.printer.borrow_mut().eprint(msg);
    }

    #[allow(dead_code)]
    pub fn set_breakpoint(&mut self, line: u16) {
        self.breakpoints.push(line);
    }

    #[allow(dead_code)]
    pub fn clear_breakpoint(&mut self, line: u16) {
        self.breakpoints = self
            .breakpoints
            .iter()
            .filter(|&val| val != &line)
            .cloned()
            .collect();
    }

    fn execute(&mut self) -> bool {
        return match self.try_execute() {
            Ok(continue_running) => continue_running,
            Err(err) => {
                self.elog(&format!("\nFatal error at byte {}:", self.pc));
                self.elog(&format!("{}", err));
                self.elog("\nInstructions:");
                let mut output = String::new();
                let mut idx = 0;
                let start = self.pc.saturating_sub(9);
                let end = self.pc.saturating_add(9).min(self.tape_ops.len() as u16);
                for i in start..end {
                    output.push_str(&format!("{:02X} ", self.tape_ops[i as usize]));
                    if i < self.pc {
                        idx += 3;
                    }
                }
                self.elog(&output);
                self.elog(&format!("{1: >0$}", idx + 2, "^^"));
                let dump = self.dump();
                self.elog("\nDump:");
                self.elog(&format!(
                    "ACC: {:02X}  D0: {:02X}  D1: {:02X}  D2: {:02X}  D3: {:02X} A0: {:04X} A1: {:04X}",
                    dump.acc, dump.data_reg[0], dump.data_reg[1], dump.data_reg[2], dump.data_reg[3], dump.addr_reg[0], dump.addr_reg[1]
                ));
                self.elog(&format!(
                    "PC: {:4} SP: {:4X} File open: {}",
                    dump.pc,
                    dump.sp,
                    self.file.is_some()
                ));
                false
            }
        };
    }

    fn cond_jump(&mut self, should_jump: bool, addr: u16, from_areg: bool) {
        if should_jump {
            self.jump(addr)
        } else if from_areg {
            self.pc += 2;
        } else {
            self.pc += 3;
        }
    }

    fn try_execute(&mut self) -> Result<bool> {
        let idx = self.pc as usize;
        let op = self.tape_ops[idx];
        match op {
            NOP => {}
            ADD_REG_REG => self.add(
                self.get_reg(self.tape_ops[idx + 1])?,
                self.get_reg(self.tape_ops[idx + 2])?,
            ),
            ADD_REG_VAL => self.add(
                self.get_reg(self.tape_ops[idx + 1])?,
                self.tape_ops[idx + 2],
            ),
            SUB_REG_REG => self.sub(
                self.get_reg(self.tape_ops[idx + 1])?,
                self.get_reg(self.tape_ops[idx + 2])?,
            ),
            SUB_REG_VAL => self.sub(
                self.get_reg(self.tape_ops[idx + 1])?,
                self.tape_ops[idx + 2],
            ),
            CPY_REG_VAL => self.load(self.tape_ops[idx + 1], self.tape_ops[idx + 2])?,
            MEMR_ADDR => self.load(
                REG_ACC,
                self.get_mem(addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2])),
            )?,
            MEMR_AREG => self.load(
                REG_ACC,
                self.get_mem(self.get_addr_reg(self.tape_ops[idx + 1])?),
            )?,
            CPY_REG_REG => self.load(
                self.tape_ops[idx + 1],
                self.get_reg(self.tape_ops[idx + 2])?,
            )?,
            MEMW_ADDR => self.store(addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2])),
            MEMW_AREG => self.store(self.get_addr_reg(self.tape_ops[idx + 1])?),
            JMP_AREG => self.jump(self.get_addr_reg(self.tape_ops[idx + 1])?),
            JE_AREG => self.cond_jump(
                self.acc == compare::EQUAL,
                self.get_addr_reg(self.tape_ops[idx + 1])?,
                true,
            ),
            JL_AREG => self.cond_jump(
                self.acc == compare::LESSER,
                self.get_addr_reg(self.tape_ops[idx + 1])?,
                true,
            ),
            JG_AREG => self.cond_jump(
                self.acc == compare::GREATER,
                self.get_addr_reg(self.tape_ops[idx + 1])?,
                true,
            ),
            JNE_AREG => self.cond_jump(
                self.acc != compare::EQUAL,
                self.get_addr_reg(self.tape_ops[idx + 1])?,
                true,
            ),
            OVER_AREG => self.cond_jump(
                self.flags.overflow,
                self.get_addr_reg(self.tape_ops[idx + 1])?,
                true,
            ),
            NOVER_AREG => self.cond_jump(
                !self.flags.overflow,
                self.get_addr_reg(self.tape_ops[idx + 1])?,
                true,
            ),
            JMP_ADDR => self.jump(addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2])),
            JE_ADDR => self.cond_jump(
                self.acc == compare::EQUAL,
                addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]),
                false,
            ),
            JL_ADDR => self.cond_jump(
                self.acc == compare::LESSER,
                addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]),
                false,
            ),
            JG_ADDR => self.cond_jump(
                self.acc == compare::GREATER,
                addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]),
                false,
            ),
            JNE_ADDR => self.cond_jump(
                self.acc != compare::EQUAL,
                addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]),
                false,
            ),
            OVER_ADDR => self.cond_jump(
                self.flags.overflow,
                addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]),
                false,
            ),
            NOVER_ADDR => self.cond_jump(
                !self.flags.overflow,
                addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]),
                false,
            ),
            INC_REG => self.change(self.tape_ops[idx + 1], 1)?,
            DEC_REG => self.change(self.tape_ops[idx + 1], -1)?,
            CMP_REG_REG => self.compare(
                self.get_reg(self.tape_ops[idx + 1])?,
                self.get_reg(self.tape_ops[idx + 2])?,
            ),
            CMP_REG_VAL => self.compare(
                self.get_reg(self.tape_ops[idx + 1])?,
                self.tape_ops[idx + 2],
            ),
            PRT_REG => self.print(self.get_reg(self.tape_ops[idx + 1])?),
            PRT_VAL => self.print(self.tape_ops[idx + 1]),
            PRTC_REG => self.printc(self.get_reg(self.tape_ops[idx + 1])?),
            PRTC_VAL => self.printc(self.tape_ops[idx + 1]),
            PRTLN => {
                self.printer.borrow_mut().newline();
            }
            PRTD_STR => self.print_dat(addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2])),
            FOPEN => self.open_file()?,
            FILER_ADDR => self.read_file(addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]))?,
            FILER_AREG => self.read_file(self.get_addr_reg(self.tape_ops[idx + 1])?)?,
            FILEW_AREG => self.write_file(self.get_addr_reg(self.tape_ops[idx + 1])?)?,
            FILEW_ADDR => self.write_file(addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]))?,
            FSEEK => self.seek_file()?,
            FSKIP_REG => self.skip_file(self.get_reg(self.tape_ops[idx + 1])?)?,
            FSKIP_VAL => self.skip_file(self.tape_ops[idx + 1])?,
            HALT => return Ok(false),
            PUSH_VAL => self.stack_push(self.tape_ops[idx + 1]),
            PUSH_REG => self.stack_push_reg(self.tape_ops[idx + 1])?,
            POP_REG => self.stack_pop(self.tape_ops[idx + 1])?,
            ARG_REG_VAL => self.stack_arg(self.tape_ops[idx + 1], self.tape_ops[idx + 2])?,
            ARG_REG_REG => self.stack_arg(
                self.tape_ops[idx + 1],
                self.get_reg(self.tape_ops[idx + 2])?,
            )?,
            RET => self.stack_return(),
            CALL_ADDR => {
                self.stack_call(addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]), false)
            }
            CALL_AREG => self.stack_call(self.get_addr_reg(self.tape_ops[idx + 1])?, true),
            SWPAR => self.swap_addr_reg(),
            CMPAR => self.cmp_addr_reg(),
            LDA0_REG_REG => self.load_addr_reg(
                REG_A0,
                self.get_reg(self.tape_ops[idx + 1])?,
                self.get_reg(self.tape_ops[idx + 2])?,
            )?,
            LDA1_REG_REG => self.load_addr_reg(
                REG_A1,
                self.get_reg(self.tape_ops[idx + 1])?,
                self.get_reg(self.tape_ops[idx + 2])?,
            )?,
            CPY_A0_REG_REG => self.copy_addr_reg(
                REG_A0,
                self.get_reg(self.tape_ops[idx + 1])?,
                self.get_reg(self.tape_ops[idx + 2])?,
            )?,
            CPY_A1_REG_REG => self.copy_addr_reg(
                REG_A1,
                self.get_reg(self.tape_ops[idx + 1])?,
                self.get_reg(self.tape_ops[idx + 2])?,
            )?,
            CPY_A0_ADDR => self
                .copy_addr_reg_val(REG_A0, addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]))?,
            CPY_A1_ADDR => self
                .copy_addr_reg_val(REG_A1, addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]))?,
            _ => {
                return Err(Error::msg(format!(
                    "Unknown instruction: {:02X}",
                    self.tape_ops[idx]
                )))
            }
        }
        if !is_jump_op(self.tape_ops[idx]) {
            let op_size = get_byte_count(self.tape_ops[idx]) as u16;
            self.pc += op_size;
        }
        Ok(true)
    }

    ///Returns contents of registers for testing/debugging
    #[allow(dead_code)]
    pub fn dump(&self) -> Dump {
        Dump {
            pc: self.pc,
            acc: self.acc,
            sp: self.sp,
            fp: self.fp,
            data_reg: self.data_reg,
            addr_reg: self.addr_reg,
            overflow: self.flags.overflow,
        }
    }

    ///Returns value at memory for testing/debugging
    #[allow(dead_code)]
    fn assert_mem(&self, addr: u16, value: u8) {
        assert_eq!(self.mem[addr as usize], value);
    }

    ///Returns value of sp for testing/debugging
    #[allow(dead_code)]
    fn assert_sp(&self, value: u16) {
        assert_eq!(self.sp, value);
    }

    ///Returns value of pc for testing/debugging
    #[allow(dead_code)]
    fn assert_pc(&self, value: u16) {
        assert_eq!(self.pc, value);
    }

    ///Returns value of data reg for testing/debugging
    #[allow(dead_code)]
    fn assert_data_reg(&self, reg: u8, value: u8) {
        assert_eq!(self.get_reg(reg).unwrap(), value);
    }

    ///Returns value of addr reg for testing/debugging
    #[allow(dead_code)]
    fn assert_addr_reg(&self, reg: u8, value: u16) {
        assert_eq!(self.get_addr_reg(reg).unwrap(), value);
    }

    //Accessors

    fn get_reg(&self, id: u8) -> Result<u8> {
        return match id {
            REG_ACC => Ok(self.acc),
            REG_D0 => Ok(self.data_reg[0]),
            REG_D1 => Ok(self.data_reg[1]),
            REG_D2 => Ok(self.data_reg[2]),
            REG_D3 => Ok(self.data_reg[3]),
            _ => Err(Error::msg(format!("Invalid data register: {:02X}", id))),
        };
    }

    fn get_addr_reg(&self, id: u8) -> Result<u16> {
        return match id {
            REG_A0 => Ok(self.addr_reg[0]),
            REG_A1 => Ok(self.addr_reg[1]),
            _ => Err(Error::msg(format!("Invalid address register: {:02X}", id))),
        };
    }

    fn get_mem(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    //Operations

    fn load_addr_reg(&mut self, addr_reg: u8, reg1: u8, reg2: u8) -> Result<()> {
        let bytes = match addr_reg {
            REG_A0 => self.addr_reg[0],
            REG_A1 => self.addr_reg[1],
            _ => {
                return Err(Error::msg(format!(
                    "Invalid addr register: {:02X}",
                    addr_reg
                )))
            }
        }
        .to_be_bytes();
        match reg1 {
            REG_ACC => self.acc = bytes[0],
            REG_D0 => self.data_reg[0] = bytes[0],
            REG_D1 => self.data_reg[1] = bytes[0],
            REG_D2 => self.data_reg[2] = bytes[0],
            REG_D3 => self.data_reg[3] = bytes[0],
            _ => return Err(Error::msg(format!("Invalid data register: {:02X}", reg1))),
        }
        match reg2 {
            REG_ACC => self.acc = bytes[1],
            REG_D0 => self.data_reg[0] = bytes[1],
            REG_D1 => self.data_reg[1] = bytes[1],
            REG_D2 => self.data_reg[2] = bytes[1],
            REG_D3 => self.data_reg[3] = bytes[1],
            _ => return Err(Error::msg(format!("Invalid data register: {:02X}", reg2))),
        }

        Ok(())
    }

    fn copy_addr_reg(&mut self, addr_reg: u8, reg1: u8, reg2: u8) -> Result<()> {
        let byte0 = match reg1 {
            REG_ACC => self.acc,
            REG_D0 => self.data_reg[0],
            REG_D1 => self.data_reg[1],
            REG_D2 => self.data_reg[2],
            REG_D3 => self.data_reg[3],
            _ => return Err(Error::msg(format!("Invalid data register: {:02X}", reg1))),
        };
        let byte1 = match reg2 {
            REG_ACC => self.acc,
            REG_D0 => self.data_reg[0],
            REG_D1 => self.data_reg[1],
            REG_D2 => self.data_reg[2],
            REG_D3 => self.data_reg[3],
            _ => return Err(Error::msg(format!("Invalid data register: {:02X}", reg2))),
        };
        let addr = u16::from_be_bytes([byte0, byte1]);
        match addr_reg {
            REG_A0 => self.addr_reg[0] = addr,
            REG_A1 => self.addr_reg[1] = addr,
            _ => {
                return Err(Error::msg(format!(
                    "Invalid addr register: {:02X}",
                    addr_reg
                )))
            }
        }

        Ok(())
    }

    fn copy_addr_reg_val(&mut self, addr_reg: u8, addr: u16) -> Result<()> {
        match addr_reg {
            REG_A0 => self.addr_reg[0] = addr,
            REG_A1 => self.addr_reg[1] = addr,
            _ => {
                return Err(Error::msg(format!(
                    "Invalid addr register: {:02X}",
                    addr_reg
                )))
            }
        }

        Ok(())
    }

    fn open_file(&mut self) -> Result<()> {
        if self.file.is_some() {
            return Err(Error::msg("File already open"));
        }
        return if let Some(path) = &self.input_data {
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(true)
                .open(path)?;
            let pos = file
                .seek(SeekFrom::End(0))
                .expect("Unable to get file length");
            self.data_reg[3] = (pos & 0xFF) as u8;
            self.data_reg[2] = (pos.rotate_right(8) & 0xFF) as u8;
            self.data_reg[1] = (pos.rotate_right(16) & 0xFF) as u8;
            self.data_reg[0] = (pos.rotate_right(24) & 0xFF) as u8;
            file.seek(SeekFrom::Start(0))
                .expect("Unable to reset file cursor");
            self.file = Some(file);

            Ok(())
        } else {
            Err(Error::msg("No input file path set"))
        };
    }

    fn seek_file(&mut self) -> Result<()> {
        match &mut self.file {
            None => Err(Error::msg("No file open")),
            Some(file) => {
                let addr = u64::from_be_bytes([
                    0,
                    0,
                    0,
                    0,
                    self.data_reg[0],
                    self.data_reg[1],
                    self.data_reg[2],
                    self.data_reg[3],
                ]);
                match file.seek(SeekFrom::Start(addr)) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(Error::from(err)),
                }
            }
        }
    }

    fn read_file(&mut self, addr: u16) -> Result<()> {
        match &mut self.file {
            None => Err(Error::msg("No file open")),
            Some(file) => {
                let mut buffer = vec![0_u8; self.acc as usize];
                match file.read(&mut buffer) {
                    Ok(count) => {
                        #[allow(clippy::needless_range_loop)] //looks better this way
                        for i in 0..count {
                            let mem_addr = addr as usize + i;
                            self.mem[mem_addr] = buffer[i];
                        }
                        self.acc = count as u8;

                        Ok(())
                    }
                    Err(err) => Err(Error::from(err)),
                }
            }
        }
    }

    fn write_file(&mut self, addr: u16) -> Result<()> {
        match &mut self.file {
            None => Err(Error::msg("No file open")),
            Some(file) => {
                let buffer = &self.mem[(addr as usize)..((addr + self.acc as u16) as usize)];
                match file.write(buffer) {
                    Ok(count) => {
                        self.acc = count as u8;

                        file.flush()?;
                        Ok(())
                    }
                    Err(err) => Err(Error::from(err)),
                }
            }
        }
    }

    fn skip_file(&mut self, val: u8) -> Result<()> {
        match &mut self.file {
            None => Err(Error::msg("No file open")),
            Some(file) => {
                let mut buffer = vec![0_u8; val as usize];
                match file.read(&mut buffer) {
                    Ok(count) => {
                        self.acc = count as u8;

                        Ok(())
                    }
                    Err(err) => Err(Error::from(err)),
                }
            }
        }
    }

    fn print(&mut self, val: u8) {
        self.log(&format!("{}", val));
    }

    fn print_dat(&mut self, data_addr: u16) {
        let length = self.tape_data[data_addr as usize] as usize;
        let str_start = (data_addr + 1) as usize;
        for i in 0..length {
            let chr_addr = str_start + i;
            self.log(&format!("{}", self.tape_data[chr_addr] as char));
        }
    }

    fn printc(&mut self, val: u8) {
        self.log(&format!("{}", val as char));
    }

    fn swap_addr_reg(&mut self) {
        self.addr_reg.swap(0, 1);
    }

    fn change(&mut self, id: u8, diff: isize) -> Result<()> {
        let update = |value: u8| {
            if diff < 1 {
                value.overflowing_sub(1)
            } else {
                value.overflowing_add(1)
            }
        };
        let update16 = |value: u16| {
            if diff < 1 {
                value.overflowing_sub(1)
            } else {
                value.overflowing_add(1)
            }
        };
        match id {
            REG_ACC => (self.acc, self.flags.overflow) = update(self.acc),
            REG_D0 => (self.data_reg[0], self.flags.overflow) = update(self.data_reg[0]),
            REG_D1 => (self.data_reg[1], self.flags.overflow) = update(self.data_reg[1]),
            REG_D2 => (self.data_reg[2], self.flags.overflow) = update(self.data_reg[2]),
            REG_D3 => (self.data_reg[3], self.flags.overflow) = update(self.data_reg[3]),
            REG_A0 => (self.addr_reg[0], self.flags.overflow) = update16(self.addr_reg[0]),
            REG_A1 => (self.addr_reg[1], self.flags.overflow) = update16(self.addr_reg[1]),
            _ => return Err(Error::msg(format!("Invalid register: {:02X}", id))),
        }

        Ok(())
    }

    fn cmp_addr_reg(&mut self) {
        match self.addr_reg[0].cmp(&self.addr_reg[1]) {
            Ordering::Less => self.acc = compare::LESSER,
            Ordering::Equal => self.acc = compare::EQUAL,
            Ordering::Greater => self.acc = compare::GREATER,
        }
    }

    fn compare(&mut self, lhs: u8, rhs: u8) {
        match lhs.cmp(&rhs) {
            Ordering::Less => self.acc = compare::LESSER,
            Ordering::Equal => self.acc = compare::EQUAL,
            Ordering::Greater => self.acc = compare::GREATER,
        }
    }

    fn add(&mut self, lhs: u8, rhs: u8) {
        let (value, overflowed) = lhs.overflowing_add(rhs);
        self.flags.overflow = overflowed;
        self.acc = value;
    }

    fn sub(&mut self, lhs: u8, rhs: u8) {
        let (value, overflowed) = lhs.overflowing_sub(rhs);
        self.flags.overflow = overflowed;
        self.acc = value;
    }

    fn load(&mut self, dest: u8, value: u8) -> Result<()> {
        match dest {
            REG_ACC => self.acc = value,
            REG_D0 => self.data_reg[0] = value,
            REG_D1 => self.data_reg[1] = value,
            REG_D2 => self.data_reg[2] = value,
            REG_D3 => self.data_reg[3] = value,
            _ => return Err(Error::msg(format!("Invalid register: {:02X}", dest))),
        }
        Ok(())
    }

    fn store(&mut self, addr: u16) {
        self.mem[addr as usize] = self.acc;
    }

    fn jump(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn sp_add(&mut self, value: u8) {
        self.sp = self.sp.saturating_sub(1);
        self.mem[self.sp as usize] = value;
    }

    fn sp_remove(&mut self) -> u8 {
        let value = self.mem[self.sp as usize];
        self.sp = self.sp.saturating_add(1).min(SP_MAX);
        value
    }

    fn stack_push(&mut self, value: u8) {
        self.sp_add(value);
    }

    fn stack_push_reg(&mut self, reg: u8) -> Result<()> {
        if matches!(reg, REG_ACC | REG_D0 | REG_D1 | REG_D2 | REG_D3) {
            self.stack_push(self.get_reg(reg)?);
        } else if reg == REG_A0 {
            let bytes = self.addr_reg[0].to_be_bytes();
            self.sp_add(bytes[0]);
            self.sp_add(bytes[1]);
        } else if reg == REG_A1 {
            let bytes = self.addr_reg[1].to_be_bytes();
            self.sp_add(bytes[0]);
            self.sp_add(bytes[1]);
        } else {
            return Err(Error::msg(format!("Invalid register: {:02X}", reg)));
        }

        Ok(())
    }

    fn stack_pop(&mut self, reg: u8) -> Result<()> {
        match reg {
            REG_ACC => self.acc = self.sp_remove(),
            REG_D0 => self.data_reg[0] = self.sp_remove(),
            REG_D1 => self.data_reg[1] = self.sp_remove(),
            REG_D2 => self.data_reg[2] = self.sp_remove(),
            REG_D3 => self.data_reg[3] = self.sp_remove(),
            REG_A0 => self.addr_reg[0] = u16::from_be_bytes([self.sp_remove(), self.sp_remove()]),
            REG_A1 => self.addr_reg[1] = u16::from_be_bytes([self.sp_remove(), self.sp_remove()]),
            _ => return Err(Error::msg(format!("Invalid register: {:02X}", reg))),
        }

        Ok(())
    }

    fn stack_arg(&mut self, reg: u8, offset: u8) -> Result<()> {
        let addr = self.fp.saturating_add(offset as u16) as usize;
        let addr_second = self.fp.saturating_add((offset.saturating_add(1)) as u16) as usize;
        match reg {
            REG_ACC => self.acc = self.mem[addr],
            REG_D0 => self.data_reg[0] = self.mem[addr],
            REG_D1 => self.data_reg[1] = self.mem[addr],
            REG_D2 => self.data_reg[2] = self.mem[addr],
            REG_D3 => self.data_reg[3] = self.mem[addr],
            REG_A0 => {
                self.addr_reg[0] = u16::from_be_bytes([self.mem[addr_second], self.mem[addr]])
            }
            REG_A1 => {
                self.addr_reg[1] = u16::from_be_bytes([self.mem[addr_second], self.mem[addr]])
            }
            _ => return Err(Error::msg(format!("Invalid register: {:02X}", reg))),
        }
        Ok(())
    }

    ///Stack explanation
    //Stack is written downwards from top of memory, so @FFFF is 0
    //FP points to the last byte of the previous frame (starts at FFFF)
    //SP points to the last byte of the current frame, which is also the end of the stack (starts at FFFF)
    //When CALLing the current FP and PC+2|3 is PUSHed to the stack
    //PC is +2|3 as that's the next instruction
    //
    //PC is then set the CALL addr
    //FP is set to SP
    //Arguments/parameters are stored relative to the FP and must be manually PUSHed and POPed by caller
    //and are access via ARG reg pos e.g. arg d0 1 gets FP+1 and sets in D0
    //So if CALLing calc with 5 the code would be
    //
    //push 5
    //call calc
    //pop acc
    //
    //Start:   SP=0 FP=0 PC=0 Stack=
    //Push:    SP=1 FP=0 PC=1 Stack=05 [param]
    //Call:    SP=5 FP=5 PC=? Stack=0500000005 [param][fp hb][fp lb][pc hb][pc lb]
    //Return:  SP=1 FP=1 PC=5 Stack=05 [param]
    //Pop:     SP=0 FP=0 PC=7 Stack=

    fn stack_call(&mut self, addr: u16, from_reg: bool) {
        let bytes = self.fp.to_be_bytes();
        self.sp_add(bytes[0]);
        self.sp_add(bytes[1]);

        let offset = if from_reg { 2 } else { 3 };
        let bytes = (self.pc.wrapping_add(offset)).to_be_bytes();
        self.sp_add(bytes[0]);
        self.sp_add(bytes[1]);

        self.pc = addr;
        self.fp = self.sp;
    }

    fn stack_return(&mut self) {
        let mut bytes = [0; 2];
        bytes[1] = self.sp_remove();
        bytes[0] = self.sp_remove();
        self.pc = u16::from_be_bytes(bytes);

        bytes[1] = self.sp_remove();
        bytes[0] = self.sp_remove();
        self.fp = u16::from_be_bytes(bytes);

        while self.fp < self.sp {
            self.sp_remove();
        }
    }
}

fn addr(byte1: u8, byte2: u8) -> u16 {
    u16::from_be_bytes([byte1, byte2])
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::compare::{EQUAL, LESSER};
    use crate::constants::hardware::REG_ACC;
    use crate::printer::DebugPrinter;

    fn assert_step_device(name: &str, device: &mut Device, dump: Dump) {
        assert!(device.step(), "step for {}", name);
        assert_eq!(device.dump(), dump, "dump for {}", name);
    }

    #[test]
    #[rustfmt::skip]
    fn test_simple() {
        let ops = vec![
            PRT_REG,
            REG_D0,
            INC_REG,
            REG_D0,
            CMP_REG_VAL,
            REG_D0,
            2,
            JNE_ADDR,
            0,
            0,
            HALT,
        ];
        let printer = DebugPrinter::new();
        let mut device = Device::new(ops, vec![], None, printer.clone());

        assert_eq!(device.dump(), Dump::default());

        assert_step_device("PRT D0", &mut device, Dump { pc: 2, ..Dump::default() }, );
        assert_eq!(printer.borrow().output(), String::from("0"));

        assert_step_device("INC D0", &mut device, Dump { pc: 4, data_reg: [1, 0, 0, 0], ..Dump::default() }, );
        assert_eq!(printer.borrow().output(), String::from("0"));

        assert_step_device("CMP D0 2", &mut device, Dump { pc: 7, acc: LESSER, data_reg: [1, 0, 0, 0], ..Dump::default() }, );
        assert_eq!(printer.borrow().output(), String::from("0"));

        assert_step_device("JNE @0", &mut device, Dump { pc: 0, acc: LESSER, data_reg: [1, 0, 0, 0], ..Dump::default() }, );
        assert_eq!(printer.borrow().output(), String::from("0"));

        assert_step_device("PRT D0", &mut device, Dump { pc: 2, acc: LESSER, data_reg: [1, 0, 0, 0], ..Dump::default() }, );
        assert_eq!(printer.borrow().output(), String::from("01"));

        assert_step_device("INC D0", &mut device, Dump { pc: 4, acc: LESSER, data_reg: [2, 0, 0, 0], ..Dump::default() }, );
        assert_eq!(printer.borrow().output(), String::from("01"));

        assert_step_device("CMP D0 2", &mut device, Dump { pc: 7, acc: EQUAL, data_reg: [2, 0, 0, 0], ..Dump::default() }, );
        assert_eq!(printer.borrow().output(), String::from("01"));

        assert_step_device("JNE @0", &mut device, Dump { pc: 10, acc: EQUAL, data_reg: [2, 0, 0, 0], ..Dump::default() }, );
        assert_eq!(printer.borrow().output(), String::from("01"));

        //HALT
        assert!(!device.step());
        assert_eq!(device.dump(), Dump { pc: 10, acc: EQUAL, data_reg: [2, 0, 0, 0], ..Dump::default() });
        assert_eq!(printer.borrow().output(), String::from("01"));
        assert_eq!(printer.borrow().error_output(), String::new());
    }

    #[test]
    #[rustfmt::skip]
    fn test_overflow_flag() {
        let ops = vec![
            DEC_REG, REG_D0, DEC_REG, REG_D0, INC_REG, REG_D0, INC_REG, REG_D0, DEC_REG, REG_A0,
            DEC_REG, REG_A0,
        ];
        let printer = DebugPrinter::new();
        let mut device = Device::new(ops, vec![], None, printer.clone());

        assert_eq!(device.dump(), Dump::default());

        assert_step_device("DEC D0", &mut device, Dump { pc: 2, data_reg: [255, 0, 0, 0], overflow: true, ..Dump::default() }, );
        assert_step_device("DEC D0", &mut device, Dump { pc: 4, data_reg: [254, 0, 0, 0], ..Dump::default() }, );
        assert_step_device("INC D0", &mut device, Dump { pc: 6, data_reg: [255, 0, 0, 0], ..Dump::default() },);
        assert_step_device("INC D0", &mut device, Dump { pc: 8, overflow: true, ..Dump::default() }, );
        assert_step_device("DEC A0", &mut device, Dump { pc: 10, addr_reg: [65535, 0], overflow: true, ..Dump::default() },);
        assert_step_device("DEC A0", &mut device, Dump { pc: 12, addr_reg: [65534, 0], ..Dump::default() }, );

        assert_eq!(printer.borrow().output(), String::new());
        assert_eq!(printer.borrow().error_output(), String::new());
    }

    #[test]
    #[rustfmt::skip]
    fn test_registers() {
        let ops = vec![
            INC_REG, REG_A0, INC_REG, REG_A1, INC_REG, REG_D0, INC_REG, REG_D1, INC_REG, REG_D2, INC_REG, REG_D3, INC_REG, REG_ACC,
            DEC_REG, REG_A0, DEC_REG, REG_A1, DEC_REG, REG_D0, DEC_REG, REG_D1, DEC_REG, REG_D2, DEC_REG, REG_D3, DEC_REG, REG_ACC,
            CPY_REG_VAL, REG_D0, 5,CPY_REG_VAL, REG_D1, 5, CPY_REG_VAL, REG_D2, 5,CPY_REG_VAL, REG_D3, 5,CPY_REG_VAL, REG_ACC, 5
        ];
        let printer = DebugPrinter::new();
        let mut device = Device::new(ops, vec![], None, printer.clone());

        assert_eq!(device.dump(), Dump::default());

        assert_step_device("INC A0", &mut device,Dump{pc:2, addr_reg:[1,0],..Dump::default()});
        assert_step_device("INC A1", &mut device,Dump{pc:4, addr_reg:[1,1],..Dump::default()});
        assert_step_device("INC D0", &mut device,Dump{pc:6, addr_reg:[1,1], data_reg: [1,0,0,0],..Dump::default()});
        assert_step_device("INC D1", &mut device,Dump{pc:8, addr_reg:[1,1], data_reg: [1,1,0,0],..Dump::default()});
        assert_step_device("INC D2", &mut device,Dump{pc:10, addr_reg:[1,1], data_reg: [1,1,1,0],..Dump::default()});
        assert_step_device("INC D3", &mut device,Dump{pc:12, addr_reg:[1,1], data_reg: [1,1,1,1], ..Dump::default()});
        assert_step_device("INC ACC", &mut device,Dump{pc:14, addr_reg:[1,1], data_reg: [1,1,1,1], acc: 1,..Dump::default()});
        assert_step_device("DEC A0", &mut device,Dump{pc:16, addr_reg:[0,1], data_reg: [1,1,1,1], acc: 1,..Dump::default()});
        assert_step_device("DEC A1", &mut device,Dump{pc:18, data_reg: [1,1,1,1], acc: 1,..Dump::default()});
        assert_step_device("DEC D0", &mut device,Dump{pc:20, data_reg: [0,1,1,1], acc: 1,..Dump::default()});
        assert_step_device("DEC D1", &mut device,Dump{pc:22, data_reg: [0,0,1,1], acc: 1,..Dump::default()});
        assert_step_device("DEC D2", &mut device,Dump{pc:24, data_reg: [0,0,0,1], acc: 1,..Dump::default()});
        assert_step_device("DEC D3", &mut device,Dump{pc:26, acc: 1,..Dump::default()});
        assert_step_device("DEC ACC", &mut device,Dump{pc:28,..Dump::default()});
        assert_step_device("CPY D0 5", &mut device,Dump{pc:31, data_reg:[5,0,0,0],..Dump::default()});
        assert_step_device("CPY D1 5", &mut device,Dump{pc:34, data_reg:[5,5,0,0],..Dump::default()});
        assert_step_device("CPY D2 5", &mut device,Dump{pc:37, data_reg:[5,5,5,0],..Dump::default()});
        assert_step_device("CPY D3 5", &mut device,Dump{pc:40, data_reg:[5,5,5,5],..Dump::default()});
        assert_step_device("CPY ACC 5", &mut device,Dump{pc:43, data_reg:[5,5,5,5], acc:5,..Dump::default()});

        assert_eq!(printer.borrow().output(), String::new());
        assert_eq!(printer.borrow().error_output(), String::new());
    }
}
