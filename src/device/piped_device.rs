use crate::device::comm::Output;
use crate::device::internals::{Device, RunResult};
use crate::device::piped_device::prefix::*;
use std::io::{stdin, stdout, Read, Write};
use std::mem::swap;
use std::thread::sleep;
use std::time::Duration;

/// PipedDevice
/// Can be used by external programs to host the Device so that it can used inside other programs such as debuggers
///
/// The Device can be communicated with over stdin and stdout using bytes
/// Instructions and user input are sent over stdin
/// Program output and diagnostics are sent over stdout
///
/// The format for both channels is <cmd><content len><content>
/// Program output: o,11,H,e,l,l,o, ,w,o,r,l,d
/// Error output:   e,12,C,r,a,s,h,e,d,\n,P,C,:, ,1
/// Set breakpoint: b,2,30

mod prefix {
    pub const OUTPUT_STR: u8 = b'o';
    pub const OUTPUT_ERR: u8 = b'e';
    pub const OUTPUT_BP_HIT: u8 = b'h';
    pub const OUTPUT_REQ_STR: u8 = b't';
    pub const OUTPUT_REQ_KEY: u8 = b'k';
    pub const OUTPUT_END: u8 = b'f';
    pub const OUTPUT_CRASH: u8 = b'c';
    pub const OUTPUT_DUMP: u8 = b'd';
    pub const OUTPUT_MEMORY: u8 = b'm';

    pub const INPUT_STEP: u8 = b'e';
    pub const INPUT_STEP_FORCE: u8 = b'f';
    pub const INPUT_DUMP: u8 = b'd';
    pub const INPUT_BP_SET: u8 = b'b';
    pub const INPUT_BP_CLEAR: u8 = b'c';
    pub const INPUT_MEMORY: u8 = b'm';
    pub const INPUT_CHAR: u8 = b'k';
    pub const INPUT_STRING: u8 = b't';
}

pub struct PipedDevice {
    device: Device,
}

impl PipedDevice {
    pub fn new(ops: Vec<u8>, strings: Vec<u8>, data: Vec<u8>, data_files: Vec<String>) -> Self {
        PipedDevice {
            device: Device::new(ops, strings, data, data_files),
        }
    }
}

impl PipedDevice {
    pub fn run(&mut self) {
        loop {
            self.process_input();

            let mut msgs = vec![];
            swap(&mut self.device.output, &mut msgs);
            for output in msgs {
                match output {
                    Output::OutputStd(text) => self.send_text(OUTPUT_STR, text),
                    Output::OutputErr(text) => self.send_text(OUTPUT_ERR, text),
                    Output::BreakpointHit(byte) => {
                        stdout()
                            .write_all(&[OUTPUT_BP_HIT])
                            .expect("Writing to stdout");
                        stdout()
                            .write_all(&byte.to_be_bytes())
                            .expect("Writing to stdout");
                    }
                }
            }

            sleep(Duration::from_millis(10));
        }
    }

    fn step(&mut self, ignore_breakpoints: bool) {
        match self.device.step(ignore_breakpoints) {
            RunResult::Pause => { /* do nothing*/ }
            RunResult::Breakpoint => { /*handled below*/ }
            RunResult::Halt | RunResult::EoF => {
                stdout()
                    .write_all(&[OUTPUT_END])
                    .expect("Writing to stdout");
            }
            RunResult::ProgError => {
                stdout()
                    .write_all(&[OUTPUT_CRASH])
                    .expect("Writing to stdout");
            }
            RunResult::CharInputRequested => {
                stdout()
                    .write_all(&[OUTPUT_REQ_KEY])
                    .expect("Writing to stdout");
            }
            RunResult::StringInputRequested => {
                stdout()
                    .write_all(&[OUTPUT_REQ_STR])
                    .expect("Writing to stdout");
            }
        }
    }

    fn send_text(&self, cmd: u8, text: String) {
        for chunk in text.into_bytes().chunks(255) {
            stdout()
                .write_all(&[cmd, chunk.len() as u8])
                .expect("Writing to stdout");
            stdout().write_all(chunk).expect("Writing to stdout");
            stdout().flush().expect("Flushing stdout");
        }
    }

    fn process_input(&mut self) {
        match read_u8() {
            INPUT_STEP => self.step(false),
            INPUT_STEP_FORCE => self.step(true),
            INPUT_DUMP => {
                let dump = self.device.dump();
                stdout()
                    .write_all(&[OUTPUT_DUMP])
                    .expect("Writing to stdout");
                write_u16(dump.pc);
                write_u16(dump.addr_reg[0]);
                write_u16(dump.addr_reg[1]);
                write_u16(dump.sp);
                write_u16(dump.fp);
                let overflow_byte = if dump.overflow { 1 } else { 0 };
                stdout()
                    .write_all(&[
                        dump.acc,
                        dump.data_reg[0],
                        dump.data_reg[1],
                        dump.data_reg[2],
                        dump.data_reg[3],
                        overflow_byte,
                    ])
                    .expect("Writing to stdout");
                stdout().flush().expect("Writing to stdout");
            }
            INPUT_BP_SET => {
                let addr = read_u16();
                self.device.breakpoints.push(addr);
            }
            INPUT_BP_CLEAR => {
                let addr = read_u16();
                self.device.breakpoints.retain(|value| value != &addr);
            }
            INPUT_MEMORY => {
                let start = read_u16();
                let end = read_u16();
                let mem = &self.device.mem[start as usize..end as usize];
                stdout()
                    .write_all(&[OUTPUT_MEMORY])
                    .expect("Writing to stdout");
                write_u16(start);
                write_u16(end);
                write_u16(mem.len() as u16);
                stdout().write_all(mem).expect("Writing to stdout");
                stdout().flush().expect("Writing to stdout");
            }
            INPUT_CHAR => {
                let chr = read_u8();
                self.device.keyboard_buffer.push(chr);
            }
            INPUT_STRING => {
                let len = read_u8() as usize;
                let mut bytes = vec![];
                for _ in 0..len {
                    bytes.push(read_u8())
                }
                self.device.keyboard_buffer.extend_from_slice(&bytes);
            }
            _ => {}
        }
    }
}

fn read_u8() -> u8 {
    let mut addr = [0_u8; 1];
    stdin().read_exact(&mut addr).expect("Reading from stdin");
    addr[0]
}

fn read_u16() -> u16 {
    let mut addr = [0_u8; 2];
    stdin().read_exact(&mut addr).expect("Reading from stdin");
    u16::from_be_bytes(addr)
}

fn write_u16(value: u16) {
    stdout()
        .write_all(&value.to_be_bytes())
        .expect("Writing to stdout");
    stdout().flush().expect("Writing to stdout");
}
