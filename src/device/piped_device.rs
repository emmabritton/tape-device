use crate::device::comm::Output;
use crate::device::internals::{Device, RunResult};
use std::io::{stdin, stdout, Write};
use std::mem::swap;
use crate::device::piped_device::prefix::{OUTPUT_STR, OUTPUT_BP_HIT, OUTPUT_ERR, OUTPUT_REQ_KEY, OUTPUT_REQ_STR, OUTPUT_END, OUTPUT_CRASH};

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
        let stdin = stdin();
        let mut stdout = stdout();
        loop {
            match self.last_run_result {
                RunResult::Pause => self.last_run_result = self.device.step(true),
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
                    Output::OutputStd(text) => {
                        stdout.write(&[OUTPUT_STR]).expect("Writing to stdout");
                        stdout.write(text.as_bytes()).expect("Writing to stdout");
                    }
                    Output::OutputErr(text) => {
                        stdout.write(&[OUTPUT_ERR]).expect("Writing to stdout");
                        stdout.write(text.as_bytes()).expect("Writing to stdout");
                    }
                    Output::BreakpointHit(byte) => {
                        stdout.write(&[OUTPUT_BP_HIT]).expect("Writing to stdout");
                        stdout.write(&byte.to_be_bytes()).expect("Writing to stdout");
                    }
                }
            }
        }
    }
}
