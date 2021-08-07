use crate::device::comm::Output;
use crate::device::input::{read_char, read_str};
use crate::device::internals::{Device, RunResult};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::ExecutableCommand;
use std::io::stdout;
use std::mem::swap;

pub struct StdDevice {
    device: Device,
    last_run_result: RunResult,
}

impl StdDevice {
    pub fn new(ops: Vec<u8>, strings: Vec<u8>, data: Vec<u8>, data_files: Vec<String>) -> Self {
        StdDevice {
            device: Device::new(ops, strings, data, data_files),
            last_run_result: RunResult::Pause,
        }
    }
}

impl StdDevice {
    pub fn run(&mut self) {
        loop {
            match self.last_run_result {
                RunResult::Pause => self.last_run_result = self.device.step(true),
                RunResult::Breakpoint => panic!("Encountered and stopped for breakpoint"),
                RunResult::EoF => return,
                RunResult::ProgError => return,
                RunResult::Halt => return,
                RunResult::CharInputRequested => {
                    let chr = read_char().expect("Error reading input (char)");
                    self.device.keyboard_buffer.push(chr);
                    self.last_run_result = RunResult::Pause;
                }
                RunResult::StringInputRequested => {
                    let input = read_str();
                    self.device.keyboard_buffer.extend_from_slice(&input);
                    self.last_run_result = RunResult::Pause;
                }
            }

            let mut msgs = vec![];
            swap(&mut self.device.output, &mut msgs);
            for output in msgs {
                match output {
                    Output::OutputStd(text) => {
                        stdout()
                            .execute(ResetColor)
                            .expect("Error setting foreground color")
                            .execute(Print(text))
                            .expect("Error printing output");
                    }
                    Output::OutputErr(text) => {
                        stdout()
                            .execute(SetForegroundColor(Color::Red))
                            .expect("Error setting foreground color")
                            .execute(Print(text))
                            .expect("Error printing error output")
                            .execute(ResetColor)
                            .expect("Error setting foreground color");
                    }
                    Output::BreakpointHit(_) => panic!("Encountered and stopped for breakpoint"),
                }
            }
        }
    }
}
