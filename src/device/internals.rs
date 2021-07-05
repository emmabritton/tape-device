use crate::constants::code::*;
use crate::constants::hardware::*;
use crate::constants::{compare, get_byte_count, is_jump_op};
use crate::device::internals::RunResult::{Breakpoint, EoF, ProgError};
use crate::printer::{Printer, RcBox};
use anyhow::{Error, Result};
use chrono::{Local, Timelike};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use random_fast_rng::{FastRng, Random};
use std::cmp::Ordering;
use std::fs::{File, OpenOptions};
use std::io::{stdin, Read, Seek, SeekFrom, Write};
use std::ops::{BitAnd, BitOr, BitXor, Not};
use std::time::Duration;

const SP_MAX: u16 = u16::MAX;
const KEY_CODE_RETURN: u8 = 10;

pub struct Device {
    mem: [u8; RAM_SIZE],
    tape_ops: Vec<u8>,
    tape_strings: Vec<u8>,
    tape_data: Vec<u8>,
    data_files: Vec<String>,
    flags: Flags,
    pc: u16,
    acc: u8,
    sp: u16,
    fp: u16,
    data_reg: [u8; DATA_REG_COUNT],
    addr_reg: [u16; ADDR_REG_COUNT],
    files: Vec<Option<File>>,
    breakpoints: Vec<u16>,
    printer: RcBox<dyn Printer>,
    rng: FastRng,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Dump {
    pub pc: u16,
    pub acc: u8,
    pub sp: u16,
    pub fp: u16,
    pub data_reg: [u8; DATA_REG_COUNT],
    pub addr_reg: [u16; ADDR_REG_COUNT],
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
        strings: Vec<u8>,
        data: Vec<u8>,
        data_files: Vec<String>,
        printer: RcBox<dyn Printer>,
    ) -> Self {
        let mut files = Vec::with_capacity(data_files.len());
        for _ in 0..data_files.len() {
            files.push(None);
        }
        Device {
            mem: [0; RAM_SIZE],
            flags: Flags::default(),
            acc: 0,
            data_reg: [0; DATA_REG_COUNT],
            addr_reg: [0; ADDR_REG_COUNT],
            pc: 0,
            sp: SP_MAX,
            fp: SP_MAX,
            breakpoints: vec![],
            tape_ops: ops,
            tape_strings: strings,
            tape_data: data,
            data_files,
            files,
            printer,
            rng: FastRng::new(),
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
        self.printer.borrow_mut().newline();
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
                    "PC: {:4} SP: {:4X} FP: {:4X} Overflowed: {}",
                    dump.pc, dump.sp, dump.fp, dump.overflow
                ));
                self.elog(&format!(
                    "Stack ({:4X}..FFFF): {:?}",
                    dump.sp,
                    &self.mem[dump.sp as usize..0xFFFF]
                ));
                false
            }
        };
    }

    fn cond_jump(&mut self, should_jump: bool, addr: u16, opcode: u8) {
        if should_jump {
            self.jump(addr)
        } else {
            self.pc += get_byte_count(opcode) as u16;
        }
    }

    fn try_execute(&mut self) -> Result<bool> {
        let idx = self.pc as usize;
        let op = self.tape_ops[idx];
        match op {
            NOP => {}
            ADD_REG_REG => self.add(
                self.get_reg_content(self.tape_ops[idx + 1])?,
                self.get_reg_content(self.tape_ops[idx + 2])?,
            ),
            ADD_REG_VAL => self.add(
                self.get_reg_content(self.tape_ops[idx + 1])?,
                self.tape_ops[idx + 2],
            ),
            SUB_REG_REG => self.sub(
                self.get_reg_content(self.tape_ops[idx + 1])?,
                self.get_reg_content(self.tape_ops[idx + 2])?,
            ),
            SUB_REG_VAL => self.sub(
                self.get_reg_content(self.tape_ops[idx + 1])?,
                self.tape_ops[idx + 2],
            ),
            MEMR_ADDR => self.load(
                REG_ACC,
                self.get_mem(addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2])),
            )?,
            MEMR_AREG => self.load(
                REG_ACC,
                self.get_mem(self.get_addr_reg_content(self.tape_ops[idx + 1])?),
            )?,
            CPY_REG_VAL => self.load(self.tape_ops[idx + 1], self.tape_ops[idx + 2])?,
            CPY_REG_REG => self.load(
                self.tape_ops[idx + 1],
                self.get_reg_content(self.tape_ops[idx + 2])?,
            )?,
            CPY_AREG_REG_REG => self.copy_addr_reg(
                self.tape_ops[idx + 1],
                self.tape_ops[idx + 2],
                self.tape_ops[idx + 3],
            )?,
            CPY_REG_REG_AREG => self.load_addr_reg(
                self.tape_ops[idx + 3],
                self.tape_ops[idx + 1],
                self.tape_ops[idx + 2],
            )?,
            CPY_AREG_ADDR => self.copy_addr_reg_val(
                self.tape_ops[idx + 1],
                addr(self.tape_ops[idx + 2], self.tape_ops[idx + 3]),
            )?,
            MEMW_ADDR => self.store(addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2])),
            MEMW_AREG => self.store(self.get_addr_reg_content(self.tape_ops[idx + 1])?),
            JMP_AREG => self.jump(self.get_addr_reg_content(self.tape_ops[idx + 1])?),
            JE_AREG => self.cond_jump(
                self.acc == compare::EQUAL,
                self.get_addr_reg_content(self.tape_ops[idx + 1])?,
                JE_AREG,
            ),
            JL_AREG => self.cond_jump(
                self.acc == compare::LESSER,
                self.get_addr_reg_content(self.tape_ops[idx + 1])?,
                JL_AREG,
            ),
            JG_AREG => self.cond_jump(
                self.acc == compare::GREATER,
                self.get_addr_reg_content(self.tape_ops[idx + 1])?,
                JG_AREG,
            ),
            JNE_AREG => self.cond_jump(
                self.acc != compare::EQUAL,
                self.get_addr_reg_content(self.tape_ops[idx + 1])?,
                JNE_AREG,
            ),
            OVER_AREG => self.cond_jump(
                self.flags.overflow,
                self.get_addr_reg_content(self.tape_ops[idx + 1])?,
                OVER_AREG,
            ),
            NOVER_AREG => self.cond_jump(
                !self.flags.overflow,
                self.get_addr_reg_content(self.tape_ops[idx + 1])?,
                NOVER_AREG,
            ),
            JMP_ADDR => self.jump(addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2])),
            JE_ADDR => self.cond_jump(
                self.acc == compare::EQUAL,
                addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]),
                JE_ADDR,
            ),
            JL_ADDR => self.cond_jump(
                self.acc == compare::LESSER,
                addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]),
                JL_ADDR,
            ),
            JG_ADDR => self.cond_jump(
                self.acc == compare::GREATER,
                addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]),
                JG_ADDR,
            ),
            JNE_ADDR => self.cond_jump(
                self.acc != compare::EQUAL,
                addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]),
                JNE_ADDR,
            ),
            OVER_ADDR => self.cond_jump(
                self.flags.overflow,
                addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]),
                OVER_ADDR,
            ),
            NOVER_ADDR => self.cond_jump(
                !self.flags.overflow,
                addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]),
                NOVER_ADDR,
            ),
            INC_REG => self.change(self.tape_ops[idx + 1], 1)?,
            DEC_REG => self.change(self.tape_ops[idx + 1], -1)?,
            CMP_REG_REG => self.compare(
                self.get_reg_content(self.tape_ops[idx + 1])?,
                self.get_reg_content(self.tape_ops[idx + 2])?,
            ),
            CMP_REG_VAL => self.compare(
                self.get_reg_content(self.tape_ops[idx + 1])?,
                self.tape_ops[idx + 2],
            ),
            CMP_AREG_ADDR => self.compare_16(
                self.get_addr_reg_content(self.tape_ops[idx + 1])?,
                addr(self.tape_ops[idx + 2], self.tape_ops[idx + 3]),
            ),
            CMP_AREG_AREG => self.compare_16(
                self.get_addr_reg_content(self.tape_ops[idx + 1])?,
                self.get_addr_reg_content(self.tape_ops[idx + 2])?,
            ),
            CMP_AREG_REG_REG => self.compare_16(
                self.get_addr_reg_content(self.tape_ops[idx + 1])?,
                addr(
                    self.get_reg_content(self.tape_ops[idx + 2])?,
                    self.get_reg_content(self.tape_ops[idx + 3])?,
                ),
            ),
            CMP_REG_REG_AREG => self.compare_16(
                addr(
                    self.get_reg_content(self.tape_ops[idx + 1])?,
                    self.get_reg_content(self.tape_ops[idx + 2])?,
                ),
                self.get_addr_reg_content(self.tape_ops[idx + 3])?,
            ),
            PRT_REG => self.print(self.get_reg_content(self.tape_ops[idx + 1])?),
            PRT_VAL => self.print(self.tape_ops[idx + 1]),
            PRTC_REG => self.printc(self.get_reg_content(self.tape_ops[idx + 1])?),
            PRTC_VAL => self.printc(self.tape_ops[idx + 1]),
            PRT_AREG => self
                .print(self.get_data_content(self.get_addr_reg_content(self.tape_ops[idx + 1])?)?),
            PRTC_AREG => self
                .printc(self.get_data_content(self.get_addr_reg_content(self.tape_ops[idx + 1])?)?),
            PRTLN => {
                self.printer.borrow_mut().newline();
            }
            PRTS_STR => {
                self.print_tape_string(addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]))
            }
            FOPEN_REG => self.open_file(self.get_reg_content(self.tape_ops[idx + 1])? as usize)?,
            FILER_REG_ADDR => self.read_file(
                self.get_reg_content(self.tape_ops[idx + 1])? as usize,
                addr(self.tape_ops[idx + 2], self.tape_ops[idx + 3]),
            )?,
            FILER_REG_AREG => self.read_file(
                self.get_reg_content(self.tape_ops[idx + 1])? as usize,
                self.get_addr_reg_content(self.tape_ops[idx + 2])?,
            )?,
            FILEW_REG_AREG => self.write_file(
                self.get_reg_content(self.tape_ops[idx + 1])? as usize,
                self.get_addr_reg_content(self.tape_ops[idx + 2])?,
            )?,
            FILEW_REG_ADDR => self.write_file(
                self.get_reg_content(self.tape_ops[idx + 1])? as usize,
                addr(self.tape_ops[idx + 2], self.tape_ops[idx + 3]),
            )?,

            FSEEK_REG => self.seek_file(self.get_reg_content(self.tape_ops[idx + 1])? as usize)?,
            FSKIP_REG_REG => self.skip_file(
                self.get_reg_content(self.tape_ops[idx + 1])? as usize,
                self.get_reg_content(self.tape_ops[idx + 2])?,
            )?,
            FSKIP_REG_VAL => self.skip_file(
                self.get_reg_content(self.tape_ops[idx + 1])? as usize,
                self.tape_ops[idx + 2],
            )?,
            FOPEN_VAL => self.open_file(self.tape_ops[idx + 1] as usize)?,
            FILER_VAL_ADDR => self.read_file(
                self.tape_ops[idx + 1] as usize,
                addr(self.tape_ops[idx + 2], self.tape_ops[idx + 3]),
            )?,
            FILER_VAL_AREG => self.read_file(
                self.tape_ops[idx + 1] as usize,
                self.get_addr_reg_content(self.tape_ops[idx + 2])?,
            )?,
            FILEW_VAL_AREG => self.write_file(
                self.tape_ops[idx + 1] as usize,
                self.get_addr_reg_content(self.tape_ops[idx + 2])?,
            )?,
            FILEW_VAL_ADDR => self.write_file(
                self.tape_ops[idx + 1] as usize,
                addr(self.tape_ops[idx + 2], self.tape_ops[idx + 3]),
            )?,
            FILEW_REG_REG => self.write_file_value(
                self.get_reg_content(self.tape_ops[idx + 1])? as usize,
                self.get_reg_content(self.tape_ops[idx + 2])?,
            )?,
            FILEW_REG_VAL => self.write_file_value(
                self.get_reg_content(self.tape_ops[idx + 1])? as usize,
                self.tape_ops[idx + 2],
            )?,
            FILEW_VAL_REG => self.write_file_value(
                self.tape_ops[idx + 1] as usize,
                self.get_reg_content(self.tape_ops[idx + 2])?,
            )?,
            FILEW_VAL_VAL => {
                self.write_file_value(self.tape_ops[idx + 1] as usize, self.tape_ops[idx + 2])?
            }
            FSEEK_VAL => self.seek_file(self.tape_ops[idx + 1] as usize)?,
            FSKIP_VAL_REG => self.skip_file(
                self.tape_ops[idx + 1] as usize,
                self.get_reg_content(self.tape_ops[idx + 2])?,
            )?,
            FSKIP_VAL_VAL => {
                self.skip_file(self.tape_ops[idx + 1] as usize, self.tape_ops[idx + 2])?
            }
            HALT => return Ok(false),
            PUSH_VAL => self.stack_push(self.tape_ops[idx + 1]),
            PUSH_REG => self.stack_push_reg(self.tape_ops[idx + 1])?,
            POP_REG => self.stack_pop(self.tape_ops[idx + 1])?,
            ARG_REG_VAL => self.stack_arg(self.tape_ops[idx + 1], self.tape_ops[idx + 2])?,
            ARG_REG_REG => self.stack_arg(
                self.tape_ops[idx + 1],
                self.get_reg_content(self.tape_ops[idx + 2])?,
            )?,
            RET => self.stack_return(),
            CALL_ADDR => {
                self.stack_call(addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]), false)
            }
            CALL_AREG => self.stack_call(self.get_addr_reg_content(self.tape_ops[idx + 1])?, true),
            SWP_REG_REG | SWP_AREG_AREG => {
                self.swap(self.tape_ops[idx + 1], self.tape_ops[idx + 2])?
            }
            IPOLL_ADDR => {
                self.poll_input(addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]), false)?
            }
            IPOLL_AREG => {
                self.poll_input(self.get_addr_reg_content(self.tape_ops[idx + 1])?, true)?
            }
            RCHR_REG => self.read_char(self.tape_ops[idx + 1])?,
            RSTR_ADDR => self.read_string(addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]))?,
            RSTR_AREG => self.read_string(self.get_addr_reg_content(self.tape_ops[idx + 1])?)?,
            MEMP_ADDR => self.print_string(addr(self.tape_ops[idx + 1], self.tape_ops[idx + 2]))?,
            MEMP_AREG => self.print_string(self.get_addr_reg_content(self.tape_ops[idx + 1])?)?,
            FCHK_REG_ADDR => self.cond_jump(
                self.files.len() > self.get_reg_content(self.tape_ops[idx + 1])? as usize,
                addr(self.tape_ops[idx + 2], self.tape_ops[idx + 3]),
                FCHK_REG_ADDR,
            ),
            FCHK_REG_AREG => self.cond_jump(
                self.files.len() > self.get_reg_content(self.tape_ops[idx + 1])? as usize,
                self.get_addr_reg_content(self.tape_ops[idx + 2])?,
                FCHK_REG_AREG,
            ),
            FCHK_VAL_ADDR => self.cond_jump(
                self.files.len() > self.tape_ops[idx + 1] as usize,
                addr(self.tape_ops[idx + 2], self.tape_ops[idx + 3]),
                FCHK_VAL_ADDR,
            ),
            FCHK_VAL_AREG => self.cond_jump(
                self.files.len() > self.tape_ops[idx + 1] as usize,
                self.get_addr_reg_content(self.tape_ops[idx + 2])?,
                FCHK_VAL_AREG,
            ),
            TIME => self.get_time(),
            RAND_REG => self.rand(self.tape_ops[idx + 1])?,
            SEED_REG => self.seed(self.tape_ops[idx + 1])?,
            AND_REG_REG => self.bit_and(
                self.get_reg_content(self.tape_ops[idx + 1])?,
                self.get_reg_content(self.tape_ops[idx + 2])?,
            ),
            AND_REG_VAL => self.bit_and(
                self.get_reg_content(self.tape_ops[idx + 1])?,
                self.tape_ops[idx + 2],
            ),
            OR_REG_REG => self.bit_or(
                self.get_reg_content(self.tape_ops[idx + 1])?,
                self.get_reg_content(self.tape_ops[idx + 2])?,
            ),
            OR_REG_VAL => self.bit_or(
                self.get_reg_content(self.tape_ops[idx + 1])?,
                self.tape_ops[idx + 2],
            ),
            XOR_REG_REG => self.bit_xor(
                self.get_reg_content(self.tape_ops[idx + 1])?,
                self.get_reg_content(self.tape_ops[idx + 2])?,
            ),
            XOR_REG_VAL => self.bit_xor(
                self.get_reg_content(self.tape_ops[idx + 1])?,
                self.tape_ops[idx + 2],
            ),
            NOT_REG => self.bit_not(self.get_reg_content(self.tape_ops[idx + 1])?),
            LD_AREG_DATA_VAL_VAL => self.load_data_addr(
                self.tape_ops[idx + 1],
                addr(self.tape_ops[idx + 2], self.tape_ops[idx + 3]),
                self.tape_ops[idx + 4],
                self.tape_ops[idx + 5],
            )?,
            LD_AREG_DATA_VAL_REG => self.load_data_addr(
                self.tape_ops[idx + 1],
                addr(self.tape_ops[idx + 2], self.tape_ops[idx + 3]),
                self.tape_ops[idx + 4],
                self.get_reg_content(self.tape_ops[idx + 5])?,
            )?,
            LD_AREG_DATA_REG_VAL => self.load_data_addr(
                self.tape_ops[idx + 1],
                addr(self.tape_ops[idx + 2], self.tape_ops[idx + 3]),
                self.get_reg_content(self.tape_ops[idx + 4])?,
                self.tape_ops[idx + 5],
            )?,
            LD_AREG_DATA_REG_REG => self.load_data_addr(
                self.tape_ops[idx + 1],
                addr(self.tape_ops[idx + 2], self.tape_ops[idx + 3]),
                self.get_reg_content(self.tape_ops[idx + 4])?,
                self.get_reg_content(self.tape_ops[idx + 5])?,
            )?,
            CPY_REG_AREG => self.load_data(self.tape_ops[idx + 1], self.tape_ops[idx + 2])?,
            CMP_REG_AREG => self.compare_data(
                self.get_reg_content(self.tape_ops[idx + 1])?,
                self.tape_ops[idx + 2],
            )?,
            PRTD_AREG => self.print_data(self.tape_ops[idx + 1])?,
            DEBUG => {
                let dump = self.dump();
                self.elog(&format!(
                    "ACC: {:02X}  D0: {:02X}  D1: {:02X}  D2: {:02X}  D3: {:02X} A0: {:04X} A1: {:04X}",
                    dump.acc, dump.data_reg[0], dump.data_reg[1], dump.data_reg[2], dump.data_reg[3], dump.addr_reg[0], dump.addr_reg[1]
                ));
                self.elog(&format!(
                    "PC: {:4} SP: {:4X} FP: {:4X} Overflowed: {}",
                    dump.pc, dump.sp, dump.fp, dump.overflow
                ));
                self.elog(&format!(
                    "Stack ({:4X}..FFFF): {:?}",
                    dump.sp,
                    &self.mem[dump.sp as usize..0xFFFF]
                ));
            }
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
        assert_eq!(self.get_reg_content(reg).unwrap(), value);
    }

    ///Returns value of addr reg for testing/debugging
    #[allow(dead_code)]
    fn assert_addr_reg(&self, reg: u8, value: u16) {
        assert_eq!(self.get_addr_reg_content(reg).unwrap(), value);
    }

    //Accessors

    fn get_reg_content(&self, id: u8) -> Result<u8> {
        return match id {
            REG_ACC => Ok(self.acc),
            REG_D0 => Ok(self.data_reg[0]),
            REG_D1 => Ok(self.data_reg[1]),
            REG_D2 => Ok(self.data_reg[2]),
            REG_D3 => Ok(self.data_reg[3]),
            _ => Err(Error::msg(format!("Invalid data register: {:02X}", id))),
        };
    }

    fn get_addr_reg_content(&self, id: u8) -> Result<u16> {
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

    fn poll_input(&mut self, addr: u16, from_areg: bool) -> Result<()> {
        if crossterm::event::poll(Duration::from_millis(100))? {
            self.jump(addr)
        } else if from_areg {
            self.pc += 2;
        } else {
            self.pc += 3;
        }

        Ok(())
    }

    fn read_string(&mut self, addr: u16) -> Result<()> {
        let mut char = [0_u8; 1];
        let mut read_count = 0;
        stdin().read_exact(&mut char)?;
        while read_count < 255 && char[0] != KEY_CODE_RETURN {
            let mem = addr as usize + read_count;
            self.mem[mem] = char[0];
            stdin().read_exact(&mut char)?;
            read_count += 1;
        }
        self.acc = read_count as u8;

        Ok(())
    }

    fn print_string(&mut self, addr: u16) -> Result<()> {
        let start = addr as usize;
        let end = (addr + self.acc as u16) as usize;
        self.log(
            String::from_utf8_lossy(&self.mem[start..end])
                .to_string()
                .as_str(),
        );
        Ok(())
    }

    fn read_char(&mut self, reg: u8) -> Result<()> {
        let mut char = [0_u8; 1];
        crossterm::terminal::enable_raw_mode()?;
        let mut event = crossterm::event::read()?;
        loop {
            if let Event::Key(key) = event {
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                    crossterm::terminal::disable_raw_mode()?;
                    std::process::exit(1);
                }
                match key.code {
                    KeyCode::Enter => {
                        char[0] = 10;
                        break;
                    }
                    KeyCode::Char(chr) => {
                        char[0] = chr as u8;
                        break;
                    }
                    KeyCode::Esc => {
                        char[0] = 27;
                        break;
                    }
                    _ => {}
                }
            }
            event = crossterm::event::read()?;
        }
        crossterm::terminal::disable_raw_mode()?;

        match reg {
            REG_ACC => self.acc = char[0],
            REG_D0 => self.data_reg[0] = char[0],
            REG_D1 => self.data_reg[1] = char[0],
            REG_D2 => self.data_reg[2] = char[0],
            REG_D3 => self.data_reg[3] = char[0],
            _ => return Err(Error::msg(format!("Invalid data register: {:02X}", reg))),
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

    fn open_file(&mut self, file_num: usize) -> Result<()> {
        if self.files[file_num].is_some() {
            return Err(Error::msg(format!("File {} already open", file_num)));
        }
        if self.data_files.len() <= file_num {
            return Err(Error::msg(format!("File {} not provided", file_num)));
        }
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.data_files[file_num])?;
        let pos = file
            .seek(SeekFrom::End(0))
            .expect("Unable to get file length");
        self.data_reg[3] = (pos & 0xFF) as u8;
        self.data_reg[2] = (pos.rotate_right(8) & 0xFF) as u8;
        self.data_reg[1] = (pos.rotate_right(16) & 0xFF) as u8;
        self.data_reg[0] = (pos.rotate_right(24) & 0xFF) as u8;
        file.seek(SeekFrom::Start(0))
            .expect("Unable to reset file cursor");
        self.files.insert(file_num, Some(file));

        Ok(())
    }

    fn seek_file(&mut self, file_num: usize) -> Result<()> {
        match &mut self.files[file_num] {
            None => Err(Error::msg(format!("File {} not open", file_num))),
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

    fn read_file(&mut self, file_num: usize, addr: u16) -> Result<()> {
        match &mut self.files[file_num] {
            None => Err(Error::msg(format!("File {} not open", file_num))),
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

    fn write_file(&mut self, file_num: usize, addr: u16) -> Result<()> {
        match &mut self.files[file_num] {
            None => Err(Error::msg(format!("File {} not open", file_num))),
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

    fn write_file_value(&mut self, file_num: usize, value: u8) -> Result<()> {
        match &mut self.files[file_num] {
            None => Err(Error::msg(format!("File {} not open", file_num))),
            Some(file) => match file.write(&[value]) {
                Ok(count) => {
                    self.acc = count as u8;

                    file.flush()?;
                    Ok(())
                }
                Err(err) => Err(Error::from(err)),
            },
        }
    }

    fn skip_file(&mut self, file_num: usize, val: u8) -> Result<()> {
        match &mut self.files[file_num] {
            None => Err(Error::msg(format!("File {} not open", file_num))),
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

    fn load_data_addr(&mut self, areg: u8, addr: u16, offset1: u8, offset2: u8) -> Result<()> {
        if (addr as usize + offset1 as usize) >= self.tape_data.len() {
            return Err(Error::msg(format!(
                "Data access out of bounds {}, max {}",
                addr + offset1 as u16,
                self.tape_data[addr as usize]
            )));
        }
        let subarray_count = self.tape_data[addr as usize];
        if offset1 > subarray_count {
            return Err(Error::msg(format!(
                "Data subarray access out of bounds {}, max {}",
                offset1, subarray_count
            )));
        }
        let mut subarray_addr = 0;
        if offset1 > 0 {
            subarray_addr += 1;
            for offset in 0..offset1 {
                subarray_addr += self.tape_data[addr as usize + offset as usize] as u16;
            }
        }
        let data_addr = addr + subarray_addr + offset2 as u16;
        if data_addr as usize >= self.tape_data.len() {
            return Err(Error::msg(format!(
                "Data byte access out of bounds {}, max {}",
                data_addr,
                self.tape_data.len()
            )));
        }
        match areg {
            REG_A0 => self.addr_reg[0] = data_addr,
            REG_A1 => self.addr_reg[1] = data_addr,
            _ => return Err(Error::msg(format!("Invalid addr register: {:02X}", areg))),
        }
        Ok(())
    }

    fn print_data(&mut self, areg: u8) -> Result<()> {
        let addr = match areg {
            REG_A0 => self.addr_reg[0],
            REG_A1 => self.addr_reg[1],
            _ => return Err(Error::msg(format!("Invalid addr register: {:02X}", areg))),
        } as usize;
        for i in 0..self.acc as usize {
            self.log(&format!("{}", self.tape_data[addr + i] as char));
        }
        Ok(())
    }

    fn print(&mut self, val: u8) {
        self.log(&format!("{}", val));
    }

    fn bit_and(&mut self, lhs: u8, rhs: u8) {
        self.acc = lhs.bitand(rhs);
    }

    fn bit_or(&mut self, lhs: u8, rhs: u8) {
        self.acc = lhs.bitor(rhs);
    }

    fn bit_xor(&mut self, lhs: u8, rhs: u8) {
        self.acc = lhs.bitxor(rhs);
    }

    fn bit_not(&mut self, value: u8) {
        self.acc = value.not();
    }

    fn print_tape_string(&mut self, data_addr: u16) {
        let length = self.tape_strings[data_addr as usize] as usize;
        let str_start = (data_addr + 1) as usize;
        for i in 0..length {
            let chr_addr = str_start + i;
            self.log(&format!("{}", self.tape_strings[chr_addr] as char));
        }
    }

    fn printc(&mut self, val: u8) {
        self.log(&format!("{}", val as char));
    }

    fn get_time(&mut self) {
        let time = Local::now();
        let hour = time.hour() as u8;
        let minute = time.minute() as u8;
        let second = time.second() as u8;
        self.data_reg[0] = second;
        self.data_reg[1] = minute;
        self.data_reg[2] = hour;
    }

    fn seed(&mut self, reg: u8) -> Result<()> {
        let value = match reg {
            REG_ACC => self.acc,
            REG_D0 => self.data_reg[0],
            REG_D1 => self.data_reg[1],
            REG_D2 => self.data_reg[2],
            REG_D3 => self.data_reg[3],
            _ => return Err(Error::msg(format!("Invalid register: {:02X}", reg))),
        };
        self.rng = FastRng::seed(value as u64, value.not() as u64);
        Ok(())
    }

    fn rand(&mut self, reg: u8) -> Result<()> {
        let num = self.rng.get_u8();
        match reg {
            REG_ACC => self.acc = num,
            REG_D0 => self.data_reg[0] = num,
            REG_D1 => self.data_reg[1] = num,
            REG_D2 => self.data_reg[2] = num,
            REG_D3 => self.data_reg[3] = num,
            _ => return Err(Error::msg(format!("Invalid register: {:02X}", reg))),
        };
        Ok(())
    }

    fn swap(&mut self, reg1: u8, reg2: u8) -> Result<()> {
        let is_addr_reg = |reg: u8| reg == REG_A0 || reg == REG_A1;
        let data_reg_idx = |reg: u8| match reg {
            REG_D0 => 0,
            REG_D1 => 1,
            REG_D2 => 2,
            REG_D3 => 3,
            _ => panic!("Impossible state, {}", reg),
        };

        if is_addr_reg(reg1) && is_addr_reg(reg2) {
            self.addr_reg.swap(0, 1);
        } else if !is_addr_reg(reg1) && !is_addr_reg(reg2) {
            if reg1 != REG_ACC && reg2 != REG_ACC {
                self.data_reg.swap(data_reg_idx(reg1), data_reg_idx(reg2))
            } else if reg1 == REG_ACC && reg2 == REG_ACC {
                //do nothing
            } else if reg1 == REG_ACC {
                let tmp = self.acc;
                self.acc = self.data_reg[data_reg_idx(reg2)];
                self.data_reg[data_reg_idx(reg2)] = tmp;
            } else if reg2 == REG_ACC {
                let tmp = self.acc;
                self.acc = self.data_reg[data_reg_idx(reg1)];
                self.data_reg[data_reg_idx(reg1)] = tmp;
            }
        } else {
            return Err(Error::msg("Invalid registers, mix of data and address"));
        }

        Ok(())
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

    fn compare(&mut self, lhs: u8, rhs: u8) {
        match lhs.cmp(&rhs) {
            Ordering::Less => self.acc = compare::LESSER,
            Ordering::Equal => self.acc = compare::EQUAL,
            Ordering::Greater => self.acc = compare::GREATER,
        }
    }

    fn compare_data(&mut self, lhs: u8, areg: u8) -> Result<()> {
        let data_addr = match areg {
            REG_A0 => self.addr_reg[0],
            REG_A1 => self.addr_reg[1],
            _ => return Err(Error::msg(format!("Invalid addr register: {:02X}", areg))),
        };
        let rhs = self.get_data_content(data_addr)?;
        match lhs.cmp(&rhs) {
            Ordering::Less => self.acc = compare::LESSER,
            Ordering::Equal => self.acc = compare::EQUAL,
            Ordering::Greater => self.acc = compare::GREATER,
        }
        Ok(())
    }

    fn compare_16(&mut self, lhs: u16, rhs: u16) {
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

    fn load_data(&mut self, dest: u8, areg: u8) -> Result<()> {
        let data_addr = match areg {
            REG_A0 => self.addr_reg[0],
            REG_A1 => self.addr_reg[1],
            _ => return Err(Error::msg(format!("Invalid addr register: {:02X}", areg))),
        };
        let data = self.get_data_content(data_addr)?;
        match dest {
            REG_ACC => self.acc = data,
            REG_D0 => self.data_reg[0] = data,
            REG_D1 => self.data_reg[1] = data,
            REG_D2 => self.data_reg[2] = data,
            REG_D3 => self.data_reg[3] = data,
            _ => return Err(Error::msg(format!("Invalid register: {:02X}", dest))),
        }
        Ok(())
    }

    fn get_data_content(&self, addr: u16) -> Result<u8> {
        if addr as usize >= self.tape_data.len() {
            return Err(Error::msg(format!(
                "Data byte access out of bounds {}, max {}",
                addr,
                self.tape_data.len()
            )));
        }
        Ok(self.tape_data[addr as usize])
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
            self.stack_push(self.get_reg_content(reg)?);
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
        let addr = self.fp.saturating_add(offset.saturating_add(3) as u16) as usize;
        let addr_second = self.fp.saturating_add((offset.saturating_add(4)) as u16) as usize;
        if addr >= SP_MAX as usize
            || ((reg == REG_A0 || reg == REG_A1) && addr_second >= SP_MAX as usize)
        {
            return Err(Error::msg(format!(
                "Attempted to access argument beyond memory {}, max {}",
                addr,
                SP_MAX - 1
            )));
        }
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
        if !device.step() {
            eprintln!("{}", device.printer.borrow().error_output());
            panic!("step for {}", name);
        }
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
        let mut device = Device::new(ops, vec![], vec![], vec![],printer.clone());

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
        let mut device = Device::new(ops, vec![], vec![], vec![],printer.clone());

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
        let mut device = Device::new(ops, vec![], vec![], vec![],printer.clone());

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

    #[test]
    #[rustfmt::skip]
    fn test_swapping() {
        let ops = vec![
            CPY_REG_VAL, REG_D0, 10,
            SWP_REG_REG, REG_D0, REG_D1,
            SWP_REG_REG, REG_D1, REG_D2,
            SWP_REG_REG, REG_D2, REG_D3,
            SWP_REG_REG, REG_D3, REG_ACC,
            CPY_AREG_ADDR, REG_A0, 0xF, 0xFF,
            SWP_AREG_AREG, REG_A0, REG_A1
        ];

        let printer = DebugPrinter::new();
        let mut device = Device::new(ops, vec![], vec![], vec![],printer.clone());

        assert_eq!(device.dump(), Dump::default());
        assert_step_device("PUSH D0 10", &mut device, Dump {pc: 3,data_reg:[10,0,0,0],..Dump::default()});
        assert_step_device("SWP D0 D1", &mut device, Dump {pc: 6,data_reg:[0,10,0,0],..Dump::default()});
        assert_step_device("SWP D1 D2", &mut device, Dump {pc: 9,data_reg:[0,0,10,0],..Dump::default()});
        assert_step_device("SWP D2 D3", &mut device, Dump {pc: 12,data_reg:[0,0,0,10],..Dump::default()});
        assert_step_device("SWP D3 ACC", &mut device, Dump {pc: 15,acc:10,..Dump::default()});
        assert_step_device("CPY A0 @0xFFF", &mut device, Dump {pc: 19,acc:10,addr_reg:[4095, 0],..Dump::default()});
        assert_step_device("SWP A0 A1", &mut device, Dump {pc: 22,acc:10,addr_reg:[0, 4095],..Dump::default()});
    }

    #[test]
    #[rustfmt::skip]
    fn test_stack_pushpop() {
        let ops = vec![
            PUSH_VAL, 5, 
            CPY_REG_VAL, REG_ACC, 10, 
            PUSH_REG, REG_ACC, 
            POP_REG, REG_D1,
            CALL_ADDR, 0,15,
            POP_REG, REG_D3,
            HALT,
            /*method @15*/ARG_REG_VAL,REG_D2,1,
            RET,
        ];
        let printer = DebugPrinter::new();
        let mut device = Device::new(ops, vec![], vec![],vec![], printer.clone());

        assert_eq!(device.dump(), Dump::default());
        device.assert_mem(SP_MAX-1 , 0);
        device.assert_mem(SP_MAX-2, 0);
        
        assert_step_device("PUSH 5", &mut device, Dump {pc: 2,sp:SP_MAX - 1,..Dump::default()});
        device.assert_mem(SP_MAX-1, 5);
        assert_step_device("CPY ACC 10", &mut device, Dump {pc: 5,sp:SP_MAX - 1,acc:10,..Dump::default()});
        assert_step_device("PUSH ACC", &mut device, Dump {pc: 7,sp:SP_MAX - 2,acc:10,..Dump::default()});
        device.assert_mem(SP_MAX-1, 5);
        device.assert_mem(SP_MAX - 2, 10);
        assert_step_device("POP D1", &mut device, Dump {pc: 9,sp:SP_MAX - 1,acc:10,data_reg:[0,10,0,0],..Dump::default()});
        device.assert_mem(SP_MAX-1, 5);
        assert_step_device("CALL @13", &mut device, Dump {pc: 15,sp:SP_MAX - 5,acc:10,data_reg:[0,10,0,0],fp:65530,..Dump::default()});
        device.assert_mem(SP_MAX-1, 5);
        device.assert_mem(SP_MAX-2, 0xff);
        device.assert_mem(SP_MAX-3, 0xff);
        device.assert_mem(SP_MAX-4, 0);
        device.assert_mem(SP_MAX-5, 12);
        assert_step_device("ARG D2 0", &mut device, Dump {pc: 18,sp:SP_MAX - 5,acc:10,data_reg:[0,10,5,0],fp:65530,..Dump::default()});
        assert_step_device("RET", &mut device, Dump {pc: 12,sp:SP_MAX - 1,acc:10,data_reg:[0,10,5,0],fp:SP_MAX,..Dump::default()});
        device.assert_mem(SP_MAX-1, 5);
        device.assert_mem(SP_MAX-2, 0xff);
        device.assert_mem(SP_MAX-3, 0xff);
        device.assert_mem(SP_MAX-4, 0);
        device.assert_mem(SP_MAX-5, 12);
        assert_step_device("POP D3", &mut device, Dump {pc: 14,sp:SP_MAX,acc:10,data_reg:[0,10,5,5],fp:SP_MAX,..Dump::default()});
        device.assert_mem(SP_MAX-1, 5);

        assert_eq!(printer.borrow().output(), String::new());
        assert_eq!(printer.borrow().error_output(), String::new());
    }

    #[test]
    #[rustfmt::skip]
    fn test_simple_data() {
        let ops = vec![
            LD_AREG_DATA_VAL_VAL, REG_A0, 0, 0, 0, 0,
            CPY_REG_AREG, REG_ACC, REG_A0,
            LD_AREG_DATA_VAL_VAL, REG_A0, 0, 0, 0, 1,
            CPY_REG_AREG, REG_ACC, REG_A0,
            LD_AREG_DATA_VAL_VAL, REG_A0, 0, 0, 1, 0,
            CPY_REG_AREG, REG_ACC, REG_A0,
            PRTC_AREG, REG_A0,
            PRTC_REG, REG_ACC,
            PRT_AREG, REG_A0,
            PRT_REG, REG_ACC
        ];

        let printer = DebugPrinter::new();
        let mut device = Device::new(ops, vec![], vec![1, 2, 97, 15], vec![], printer.clone());

        assert_eq!(device.dump(), Dump::default());

        assert_step_device("LD A0 @0 0 0", &mut device, Dump { pc: 6, ..Dump::default() });
        assert_step_device("CPY ACC A0", &mut device, Dump { pc: 9, acc: 1, ..Dump::default() });
        assert_step_device("LD A0 @0 0 1", &mut device, Dump { pc: 15, acc: 1, addr_reg: [1, 0], ..Dump::default() });
        assert_step_device("CPY ACC A0", &mut device, Dump { pc: 18, acc: 2, addr_reg: [1, 0], ..Dump::default() });
        assert_step_device("LD A0 @0 1 0", &mut device, Dump { pc: 24, acc: 2, addr_reg: [2, 0], ..Dump::default() });
        assert_step_device("CPY ACC A0", &mut device, Dump { pc: 27, acc: 97, addr_reg: [2, 0], ..Dump::default() });
        assert_step_device("PRTC A0", &mut device, Dump { pc: 29, acc: 97, addr_reg: [2, 0], ..Dump::default() });
        assert_step_device("PRTC ACC", &mut device, Dump { pc: 31, acc: 97, addr_reg: [2, 0], ..Dump::default() });
        assert_step_device("PRT A0", &mut device, Dump { pc: 33, acc: 97, addr_reg: [2, 0], ..Dump::default() });
        assert_step_device("PRT ACC", &mut device, Dump { pc: 35, acc: 97, addr_reg: [2, 0], ..Dump::default() });

        assert_eq!(printer.borrow().output(), String::from("aa9797"));
        assert_eq!(printer.borrow().error_output(), String::new());
    }

    #[test]
    #[rustfmt::skip]
    fn test_complex_data() {
        let ops = vec![
            LD_AREG_DATA_VAL_VAL, REG_A0, 0, 0, 0, 0,
            CPY_REG_AREG, REG_ACC, REG_A0,
            LD_AREG_DATA_VAL_VAL, REG_A0, 0, 0, 0, 1,
            CPY_REG_AREG, REG_ACC, REG_A0,
            LD_AREG_DATA_VAL_VAL, REG_A0, 0, 5, 0, 0,
            CPY_REG_AREG, REG_ACC, REG_A0,
            LD_AREG_DATA_VAL_VAL, REG_A0, 0, 5, 0, 1,
            CPY_REG_AREG, REG_ACC, REG_A0,
            LD_AREG_DATA_VAL_VAL, REG_A0, 0, 5, 0, 2,
            CPY_REG_AREG, REG_ACC, REG_A0,
            LD_AREG_DATA_VAL_VAL, REG_A0, 0, 12, 0, 0,
            CPY_REG_AREG, REG_ACC, REG_A0,
            LD_AREG_DATA_VAL_VAL, REG_A0, 0, 12, 0, 1,
            CPY_REG_AREG, REG_ACC, REG_A0,
            LD_AREG_DATA_VAL_VAL, REG_A0, 0, 12, 0, 2,
            CPY_REG_AREG, REG_ACC, REG_A0,
            LD_AREG_DATA_VAL_VAL, REG_A1, 0, 0, 1, 1,
            PRTC_AREG, REG_A1,
            LD_AREG_DATA_VAL_VAL, REG_A1, 0, 0, 1, 2,
            PRT_AREG, REG_A1,
            LD_AREG_DATA_VAL_VAL, REG_A1, 0, 12, 2, 0,
            PRTC_AREG, REG_A1,
            LD_AREG_DATA_VAL_VAL, REG_A1, 0, 12, 2, 4,
            PRTC_AREG, REG_A1,
            LD_AREG_DATA_VAL_VAL, REG_A0, 0, 12, 0, 1,
            CPY_REG_AREG, REG_ACC, REG_A0,
            LD_AREG_DATA_VAL_VAL, REG_A0, 0, 12, 1, 0,
            PRTD_AREG, REG_A0
        ];

        let printer = DebugPrinter::new();
        //.data
        //text=["abc"]
        //lists=[[20,30],[50,51]]
        //chars=[['h','e','l','l','o'],['w','o','r','l','d']]
        let mut device = Device::new(ops,
                                     vec![],
                                     vec![1, 3, 97, 98, 99, 2, 2, 2, 20, 30, 50, 51, 2, 5, 5, 104, 101, 108, 108, 111, 119, 111, 114, 108, 100],
                                     vec![],
                                     printer.clone());

        assert_eq!(device.dump(), Dump::default());

        assert_step_device("LD A0 text 0 0", &mut device, Dump { pc: 6, ..Dump::default() });
        assert_step_device("CPY ACC A0", &mut device, Dump { pc: 9, acc: 1, ..Dump::default() });
        assert_step_device("LD A0 text 0 1", &mut device, Dump { pc: 15, acc: 1, addr_reg: [1, 0], ..Dump::default() });
        assert_step_device("CPY ACC A0", &mut device, Dump { pc: 18, acc: 3, addr_reg: [1, 0], ..Dump::default() });

        assert_step_device("LD A0 lists 0 0", &mut device, Dump { pc: 24, acc: 3, addr_reg: [5, 0], ..Dump::default() });
        assert_step_device("CPY ACC A0", &mut device, Dump { pc: 27, acc: 2, addr_reg: [5, 0], ..Dump::default() });
        assert_step_device("LD A0 lists 0 1", &mut device, Dump { pc: 33, acc: 2, addr_reg: [6, 0], ..Dump::default() });
        assert_step_device("CPY ACC A0", &mut device, Dump { pc: 36, acc: 2, addr_reg: [6, 0], ..Dump::default() });
        assert_step_device("LD A0 lists 0 2", &mut device, Dump { pc: 42, acc: 2, addr_reg: [7, 0], ..Dump::default() });
        assert_step_device("CPY ACC A0", &mut device, Dump { pc: 45, acc: 2, addr_reg: [7, 0], ..Dump::default() });

        assert_step_device("LD A0 chars 0 0", &mut device, Dump { pc: 51, acc: 2, addr_reg: [12, 0], ..Dump::default() });
        assert_step_device("CPY ACC A0", &mut device, Dump { pc: 54, acc: 2, addr_reg: [12, 0], ..Dump::default() });
        assert_step_device("LD A0 chars 0 1", &mut device, Dump { pc: 60, acc: 2, addr_reg: [13, 0], ..Dump::default() });
        assert_step_device("CPY ACC A0", &mut device, Dump { pc: 63, acc: 5, addr_reg: [13, 0], ..Dump::default() });
        assert_step_device("LD A0 chars 0 2", &mut device, Dump { pc: 69, acc: 5, addr_reg: [14, 0], ..Dump::default() });
        assert_step_device("CPY ACC A0", &mut device, Dump { pc: 72, acc: 5, addr_reg: [14, 0], ..Dump::default() });

        assert_step_device("LD A1 text 1 1", &mut device, Dump { pc: 78, acc: 5, addr_reg: [14, 3], ..Dump::default() });
        assert_step_device("PRTC A1", &mut device, Dump { pc: 80, acc: 5, addr_reg: [14, 3], ..Dump::default() });
        assert_step_device("LD A1 text 1 2", &mut device, Dump { pc: 86, acc: 5, addr_reg: [14, 4], ..Dump::default() });
        assert_step_device("PRT A1", &mut device, Dump { pc: 88, acc: 5, addr_reg: [14, 4], ..Dump::default() });

        assert_step_device("LD A1 chars 2 0", &mut device, Dump { pc: 94, acc: 5, addr_reg: [14, 20], ..Dump::default() });
        assert_step_device("PRTC A1", &mut device, Dump { pc: 96, acc: 5, addr_reg: [14, 20], ..Dump::default() });
        assert_step_device("LD A1 chars 2 4", &mut device, Dump { pc: 102, acc: 5, addr_reg: [14, 24], ..Dump::default() });
        assert_step_device("PRT A1", &mut device, Dump { pc: 104, acc: 5, addr_reg: [14, 24], ..Dump::default() });

        assert_step_device("LD A0 chars 0 1", &mut device, Dump { pc: 110, acc: 5, addr_reg: [13, 24], ..Dump::default() });
        assert_step_device("CPY ACC A0", &mut device, Dump { pc: 113, acc: 5, addr_reg: [13, 24], ..Dump::default() });
        assert_step_device("LD A0 chars 1 0", &mut device, Dump { pc: 119, acc: 5, addr_reg: [15, 24], ..Dump::default() });
        assert_step_device("PRTD A0", &mut device, Dump { pc: 121, acc: 5, addr_reg: [15, 24], ..Dump::default() });

        assert_eq!(printer.borrow().output(), String::from("b99wdhello"));
        assert_eq!(printer.borrow().error_output(), String::new());
    }
}
