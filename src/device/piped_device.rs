use crate::device::comm::Output;
use crate::device::internals::{Device, RunResult};
use std::io::{stdin, stdout, Write, Read};
use std::mem::swap;
use crate::device::piped_device::prefix::*;
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

    pub const INPUT_STEP: u8 = b'e';
    pub const INPUT_DUMP: u8 = b'd';
}

pub struct PipedDevice {
    device: Device,
    last_run_result: RunResult,
}

impl PipedDevice {
    pub fn new(ops: Vec<u8>, strings: Vec<u8>, data: Vec<u8>, data_files: Vec<String>) -> Self {
        PipedDevice {
            device: Device::new(ops, strings, data, data_files),
            last_run_result: RunResult::Pause,
        }
    }
}

impl PipedDevice {
    pub fn run(&mut self) {
        let mut stdout = stdout();
        loop {
            self.process_input();

            match self.last_run_result {
                RunResult::Pause => {/* do nothing*/},
                RunResult::Breakpoint => {/*handled below*/},
                RunResult::Halt | RunResult::EoF => {
                    stdout.write(&[OUTPUT_END]).expect("Writing to stdout");
                },
                RunResult::ProgError => {
                    stdout.write(&[OUTPUT_CRASH]).expect("Writing to stdout");
                },
                RunResult::CharInputRequested => {
                    stdout.write(&[OUTPUT_REQ_KEY]).expect("Writing to stdout");
                },
                RunResult::StringInputRequested => {
                    stdout.write(&[OUTPUT_REQ_STR]).expect("Writing to stdout");
                },
            }

            let mut msgs = vec![];
            swap(&mut self.device.output, &mut msgs);
            for output in msgs {
                match output {
                    Output::OutputStd(text) => self.send_text(OUTPUT_STR, text),
                    Output::OutputErr(text) => self.send_text(OUTPUT_ERR, text),
                    Output::BreakpointHit(byte) => {
                        stdout.write(&[OUTPUT_BP_HIT]).expect("Writing to stdout");
                        stdout.write(&byte.to_be_bytes()).expect("Writing to stdout");
                    }
                }
            }

            sleep(Duration::from_millis(10));
        }
    }

    fn send_text(&self, cmd: u8, text: String) {
        for chunk in text.into_bytes().chunks(255) {
            stdout().write(&[cmd, chunk.len() as u8]).expect("Writing to stdout");
            stdout().write(chunk).expect("Writing to stdout");
            stdout().flush().expect("Flushing stdout");
        }
    }

    fn process_input(&mut self) {
        let mut byte = [0_u8; 1];
        stdin().read_exact(&mut byte).expect("Reading from stdin");
        match byte[0] {
            INPUT_STEP => self.last_run_result = self.device.step(false),
            INPUT_DUMP => {
                let json = serde_json::to_string(&self.device.dump()).expect("Creating dump json");
                self.send_text(OUTPUT_DUMP, json);
            },
            _ => {}
        }
    }
}
