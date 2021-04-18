use crate::common::Instruction;
use crate::constants::code::*;
use crate::constants::compare;
use crate::constants::hardware::*;
use crate::device::internals::RunResult::{Breakpoint, EoF, ProgError};
use crate::printer::Printer;
use anyhow::{Error, Result};
use std::cmp::Ordering;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

pub struct Device {
    mem: [u8; RAM_SIZE],
    tape_ops: Vec<Instruction>,
    tape_data: Vec<u8>,
    input_data: Option<String>,
    flags: Flags,
    pub debug: Debug,
    pc: u16,
    acc: u8,
    reg: [u8; REG_SIZE],
    file: Option<File>,
    breakpoints: Vec<u16>,
    printer: Box<dyn Printer>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum RunResult {
    Breakpoint,
    EoF,
    ProgError,
}

impl Device {
    pub fn new(
        ops: Vec<Instruction>,
        data: Vec<u8>,
        input_file: Option<String>,
        printer: Box<dyn Printer>,
    ) -> Self {
        Device {
            mem: [0; RAM_SIZE],
            flags: Flags::default(),
            acc: 0,
            reg: [0; REG_SIZE],
            pc: 0,
            debug: Debug::default(),
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
pub struct Debug {
    pub pc: bool,
}

#[derive(Debug, Default)]
pub struct Flags {
    overflow: bool,
}

impl Device {
    pub fn run(&mut self) -> RunResult {
        loop {
            if self.breakpoints.contains(&self.pc) {
                self.clear_breakpoint(self.pc);
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

    pub fn step(&mut self) -> bool {
        if self.pc as usize >= self.tape_ops.len() {
            panic!("Tried to execute EoF")
        }
        self.execute(self.tape_ops[self.pc as usize])
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

    fn execute(&mut self, instruction: Instruction) -> bool {
        if self.debug.pc {
            self.printer.print(&format!("{}", self.pc));
        }
        return match self.try_execute(instruction) {
            Ok(continue_running) => continue_running,
            Err(err) => {
                self.printer
                    .eprint(&format!("\nFatal error at line {}:", self.pc + 1));
                self.printer.eprint(&format!("{}", err));
                self.printer.eprint(&format!("\nInstructions:"));
                for i in self.pc.saturating_sub(2)..self.pc {
                    self.printer.eprint(&format!(
                        "{:4}   {:02X} {:02X} {:02X}",
                        i + 1,
                        self.tape_ops[i as usize][0],
                        self.tape_ops[i as usize][1],
                        self.tape_ops[i as usize][2]
                    ));
                }
                self.printer.eprint(&format!(
                    "{:4} > {:02X} {:02X} {:02X} <",
                    self.pc + 1,
                    instruction[0],
                    instruction[1],
                    instruction[2]
                ));
                for i in (self.pc as usize)
                    .saturating_add(1)
                    .min(self.tape_ops.len())
                    ..(self.pc as usize)
                        .saturating_add(3)
                        .min(self.tape_ops.len())
                {
                    self.printer.eprint(&format!(
                        "{:4}   {:02X} {:02X} {:02X}",
                        i + 1,
                        self.tape_ops[i as usize][0],
                        self.tape_ops[i as usize][1],
                        self.tape_ops[i as usize][2]
                    ));
                }
                let (acc, reg, pc) = self.dump();
                self.printer.eprint(&format!("\nDump:"));
                self.printer.eprint(&format!(
                    "ACC: {:02X}  D0: {:02X}  D1: {:02X}  D2: {:02X}  D3: {:02X}",
                    acc, reg[0], reg[1], reg[2], reg[3]
                ));
                self.printer
                    .eprint(&format!("PC: {:4}  File open: {}", pc, self.file.is_some()));
                false
            }
        };
    }

    fn try_execute(&mut self, instruction: Instruction) -> Result<bool> {
        match instruction[0] {
            OP_NOP => self.pc += 1,
            OP_ADD_REG_REG => {
                self.add(self.get_reg(instruction[1])?, self.get_reg(instruction[2])?)
            }
            OP_ADD_REG_VAL => self.add(self.get_reg(instruction[1])?, instruction[2]),
            OP_SUB_REG_REG => {
                self.sub(self.get_reg(instruction[1])?, self.get_reg(instruction[2])?)
            }
            OP_SUB_REG_VAL => self.sub(self.get_reg(instruction[1])?, instruction[2]),
            OP_COPY_REG_VAL => self.load(instruction[1], instruction[2])?,
            OP_MEM_READ => self.load(REG_ACC, self.get_mem(addr(&instruction, 1)))?,
            OP_COPY_REG_REG => self.load(instruction[1], self.get_reg(instruction[2])?)?,
            OP_MEM_WRITE => self.store(addr(&instruction, 1)),
            OP_JMP => self.jump(addr(&instruction, 1)),
            OP_JE => {
                if self.acc == compare::EQUAL {
                    self.jump(addr(&instruction, 1))
                } else {
                    self.pc += 1;
                }
            }
            OP_JL => {
                if self.acc == compare::LESSER {
                    self.jump(addr(&instruction, 1))
                } else {
                    self.pc += 1;
                }
            }
            OP_JG => {
                if self.acc == compare::GREATER {
                    self.jump(addr(&instruction, 1))
                } else {
                    self.pc += 1;
                }
            }
            OP_JNE => {
                if self.acc != compare::EQUAL {
                    self.jump(addr(&instruction, 1))
                } else {
                    self.pc += 1;
                }
            }
            OP_OVERFLOW => {
                if self.flags.overflow {
                    self.jump(addr(&instruction, 1))
                } else {
                    self.pc += 1;
                }
            }
            OP_NOT_OVERFLOW => {
                if !self.flags.overflow {
                    self.jump(addr(&instruction, 1))
                } else {
                    self.pc += 1;
                }
            }
            OP_INC => self.change(instruction[1], 1)?,
            OP_DEC => self.change(instruction[1], -1)?,
            OP_CMP_REG_REG => {
                self.compare(self.get_reg(instruction[1])?, self.get_reg(instruction[2])?)
            }
            OP_CMP_REG_VAL => self.compare(self.get_reg(instruction[1])?, instruction[2]),
            OP_PRINT_REG => self.print(self.get_reg(instruction[1])?),
            OP_PRINT_VAL => self.print(instruction[1]),
            OP_PRINT_LN => {
                self.printer.newline();
                self.pc += 1;
            }
            OP_PRINT_DAT => self.print_dat(addr(&instruction, 1)),
            OP_PRINT_MEM => self.print_mem(addr(&instruction, 1)),
            OP_OPEN_FILE => self.open_file()?,
            OP_READ_FILE => self.read_file(addr(&instruction, 1))?,
            OP_WRITE_FILE => self.write_file(addr(&instruction, 1))?,
            OP_SEEK_FILE => self.seek_file()?,
            OP_SKIP_FILE => self.skip_file(self.get_reg(instruction[1])?)?,
            OP_HALT => return Ok(false),
            _ => {
                return Err(Error::msg(format!(
                    "Unknown instruction: {:02X}",
                    instruction[0]
                )))
            }
        }
        Ok(true)
    }

    #[allow(dead_code)]
    fn dump(&self) -> (u8, [u8; REG_SIZE], u16) {
        (self.acc, self.reg, self.pc)
    }

    #[allow(dead_code)]
    fn assert_mem(&self, addr: u16, value: u8) {
        assert_eq!(self.mem[addr as usize], value);
    }

    #[allow(dead_code)]
    fn assert_reg(&self, reg: u8, value: u8) {
        assert_eq!(self.get_reg(reg).unwrap(), value);
    }

    //Accessors

    fn get_reg(&self, id: u8) -> Result<u8> {
        return match id {
            REG_ACC => Ok(self.acc),
            REG_D0 => Ok(self.reg[0]),
            REG_D1 => Ok(self.reg[1]),
            REG_D2 => Ok(self.reg[2]),
            REG_D3 => Ok(self.reg[3]),
            _ => Err(Error::msg(format!("Invalid register: {:02X}", id))),
        };
    }

    fn get_mem(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    //Operations

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
            self.reg[3] = (pos & 0xFF) as u8;
            self.reg[2] = (pos.rotate_right(8) & 0xFF) as u8;
            self.reg[1] = (pos.rotate_right(16) & 0xFF) as u8;
            self.reg[0] = (pos.rotate_right(24) & 0xFF) as u8;
            file.seek(SeekFrom::Start(0))
                .expect("Unable to reset file cursor");
            self.file = Some(file);
            self.pc += 1;
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
                    self.reg[0],
                    self.reg[1],
                    self.reg[2],
                    self.reg[3],
                ]);
                match file.seek(SeekFrom::Start(addr)) {
                    Ok(_) => {
                        self.pc += 1;
                        Ok(())
                    }
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
                        self.pc += 1;
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
                        self.pc += 1;
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
                        self.pc += 1;
                        Ok(())
                    }
                    Err(err) => Err(Error::from(err)),
                }
            }
        }
    }

    fn print(&mut self, val: u8) {
        self.printer.print(&format!("{}", val));
        self.pc += 1;
    }

    fn print_dat(&mut self, data_addr: u16) {
        let length = self.tape_data[data_addr as usize] as usize;
        let str_start = (data_addr + 1) as usize;
        for i in 0..length {
            let chr_addr = str_start + i;
            self.printer
                .print(&format!("{}", self.tape_data[chr_addr] as char));
        }
        self.pc += 1;
    }

    fn print_mem(&mut self, addr: u16) {
        let initial = self.acc as usize;
        while self.acc > 0 {
            let chr = self.mem[addr as usize + (initial - self.acc as usize)] as char;
            self.printer.print(&format!("{}", chr));
            self.acc -= 1;
        }
        self.acc = initial as u8;
        self.pc += 1;
    }

    fn change(&mut self, id: u8, diff: isize) -> Result<()> {
        let update = |value: u8| {
            if diff < 1 {
                let (value, overflowed) = value.overflowing_sub(1);
                self.flags.overflow = overflowed;
                value
            } else {
                let (value, overflowed) = value.overflowing_add(1);
                self.flags.overflow = overflowed;
                value
            }
        };
        match id {
            REG_ACC => self.acc = update(self.acc),
            REG_D0 => self.reg[0] = update(self.reg[0]),
            REG_D1 => self.reg[1] = update(self.reg[1]),
            REG_D2 => self.reg[2] = update(self.reg[2]),
            REG_D3 => self.reg[3] = update(self.reg[3]),
            _ => return Err(Error::msg(format!("Invalid register: {:02X}", id))),
        }
        self.pc += 1;
        Ok(())
    }

    fn compare(&mut self, lhs: u8, rhs: u8) {
        match lhs.cmp(&rhs) {
            Ordering::Less => self.acc = compare::LESSER,
            Ordering::Equal => self.acc = compare::EQUAL,
            Ordering::Greater => self.acc = compare::GREATER,
        }
        self.pc += 1;
    }

    fn add(&mut self, lhs: u8, rhs: u8) {
        let (value, overflowed) = lhs.overflowing_add(rhs);
        self.flags.overflow = overflowed;
        self.acc = value;
        self.pc += 1;
    }

    fn sub(&mut self, lhs: u8, rhs: u8) {
        let (value, overflowed) = lhs.overflowing_sub(rhs);
        self.flags.overflow = overflowed;
        self.acc = value;
        self.pc += 1;
    }

    fn load(&mut self, dest: u8, value: u8) -> Result<()> {
        match dest {
            REG_ACC => self.acc = value,
            REG_D0 => self.reg[0] = value,
            REG_D1 => self.reg[1] = value,
            REG_D2 => self.reg[2] = value,
            REG_D3 => self.reg[3] = value,
            _ => return Err(Error::msg(format!("Invalid register: {:02X}", dest))),
        }
        self.pc += 1;
        Ok(())
    }

    fn store(&mut self, addr: u16) {
        self.mem[addr as usize] = self.acc;
        self.pc += 1;
    }

    fn jump(&mut self, addr: u16) {
        self.pc = addr;
    }
}

fn addr(arr: &Instruction, start: usize) -> u16 {
    u16::from_be_bytes([arr[start], arr[start + 1]])
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::hardware::REG_ACC;
    use crate::printer::StdoutPrinter;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_handling() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_string_lossy();
        println!("Testing with {}", path);

        let mut device = Device::new(
            vec![
                [OP_COPY_REG_VAL, REG_ACC, 10],
                [OP_MEM_WRITE, 0, 0xF0],
                [OP_COPY_REG_VAL, REG_ACC, 11],
                [OP_MEM_WRITE, 0, 0xF1],
                [OP_COPY_REG_VAL, REG_ACC, 12],
                [OP_MEM_WRITE, 0, 0xF2],
                [OP_COPY_REG_VAL, REG_ACC, 13],
                [OP_MEM_WRITE, 0, 0xF3],
                [OP_COPY_REG_VAL, REG_ACC, 14],
                [OP_MEM_WRITE, 0, 0xF4],
                [OP_OPEN_FILE, 0, 0],
                [OP_COPY_REG_VAL, REG_ACC, 1],
                [OP_WRITE_FILE, 0, 0xF0],
                [OP_COPY_REG_VAL, REG_ACC, 1],
                [OP_WRITE_FILE, 0, 0xF0],
                [OP_COPY_REG_VAL, REG_ACC, 1],
                [OP_WRITE_FILE, 0, 0xF0],
                [OP_COPY_REG_VAL, REG_ACC, 1],
                [OP_WRITE_FILE, 0, 0xF0],
                [OP_SEEK_FILE, 0, 0],
                [OP_COPY_REG_VAL, REG_ACC, 4],
                [OP_READ_FILE, 0, 0],
            ],
            vec![],
            Some(path.to_string()),
            StdoutPrinter::boxed(),
        );

        let result = device.run();

        let dump = device.dump();
        assert_eq!(result, EoF);
        assert_eq!(dump.2, 22);
        assert_eq!(dump.0, 4);
        device.assert_mem(0, 10);
        device.assert_mem(1, 10);
        device.assert_mem(2, 10);
        device.assert_mem(3, 10);
        device.assert_mem(4, 0);
    }

    #[test]
    fn test_simple() {
        let mut device = Device::new(
            vec![
                [OP_COPY_REG_VAL, REG_D0, 10],
                [OP_COPY_REG_VAL, REG_D2, 20],
                [OP_COPY_REG_VAL, REG_D3, 30],
                [OP_ADD_REG_REG, REG_D0, REG_D1],
            ],
            vec![],
            None,
            StdoutPrinter::boxed(),
        );
        device.run();
    }

    #[test]
    fn test_halt() {
        let mut device = Device::new(
            vec![
                [OP_COPY_REG_VAL, REG_D1, 1],
                [OP_HALT, 0, 0],
                [OP_COPY_REG_VAL, REG_D1, 2],
            ],
            vec![],
            None,
            StdoutPrinter::boxed(),
        );
        device.run();

        device.assert_reg(REG_D1, 1);
    }

    #[test]
    fn test_jmp() {
        let mut device = Device::new(
            vec![
                [OP_COPY_REG_VAL, REG_D1, 3],
                [OP_SUB_REG_VAL, REG_D1, 3],
                [OP_COPY_REG_REG, REG_D1, REG_ACC],
                [OP_CMP_REG_VAL, REG_D1, 0],
                [OP_JE, 0, 6],
                [OP_HALT, 0, 0],
                [OP_COPY_REG_VAL, REG_D0, 2],
            ],
            vec![],
            None,
            StdoutPrinter::boxed(),
        );
        device.run();

        device.assert_reg(REG_D0, 2);
    }

    #[test]
    fn test_loop() {
        let mut device = Device::new(
            vec![
                [OP_COPY_REG_VAL, REG_D0, 0],
                [OP_INC, REG_D0, 0],
                [OP_CMP_REG_VAL, REG_D0, 5],
                [OP_JNE, 0, 1],
            ],
            vec![],
            None,
            StdoutPrinter::boxed(),
        );
        device.run();

        device.assert_reg(REG_D0, 5);
    }

    #[test]
    fn integration_test() {
        let mut device = Device::new(vec![], vec![], None, StdoutPrinter::boxed());
        for i in 0..RAM_SIZE {
            device.assert_mem(i as u16, 0);
        }
        device.assert_reg(REG_ACC, 0);
        device.assert_reg(REG_D0, 0);
        device.assert_reg(REG_D1, 0);

        device.execute([OP_COPY_REG_VAL, REG_ACC, 0x01]);
        device.execute([OP_COPY_REG_VAL, REG_D0, 0x08]);
        device.execute([OP_COPY_REG_VAL, REG_D1, 0x10]);

        for i in 0..RAM_SIZE {
            device.assert_mem(i as u16, 0);
        }
        device.assert_reg(REG_ACC, 1);
        device.assert_reg(REG_D0, 8);
        device.assert_reg(REG_D1, 16);

        device.execute([OP_COPY_REG_REG, REG_D0, REG_ACC]);

        device.assert_reg(REG_ACC, 1);
        device.assert_reg(REG_D0, 1);

        device.execute([OP_MEM_WRITE, 0x00, 0x00]);

        device.assert_mem(0, 1);

        device.execute([OP_COPY_REG_VAL, REG_D2, 0x04]);
        device.execute([OP_COPY_REG_VAL, REG_D3, 0x04]);
        device.execute([OP_ADD_REG_REG, REG_D2, REG_D3]);

        device.execute([OP_MEM_WRITE, 0x12, 0x34]);

        device.assert_mem(0x1234, 8);

        device.execute([OP_SUB_REG_REG, REG_ACC, REG_D3]);

        device.assert_reg(REG_ACC, 4);
        device.assert_reg(REG_D2, 4);
        device.assert_reg(REG_D3, 4);

        device.execute([OP_COPY_REG_VAL, REG_D0, 10]);
        device.execute([OP_ADD_REG_VAL, REG_D0, 10]);

        device.assert_reg(REG_ACC, 20);

        device.execute([OP_INC, REG_ACC, 0]);

        device.assert_reg(REG_ACC, 21);

        device.execute([OP_DEC, REG_D2, 0]);
        device.execute([OP_DEC, REG_D2, 0]);

        device.assert_reg(REG_D2, 2);
    }
}
