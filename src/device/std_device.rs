use crate::device::comm::Output;
use crate::device::internals::{Device, RunResult};
use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use crossterm::style::{Color, Print, SetForegroundColor};
use crossterm::ExecutableCommand;
use std::io::{stdin, stdout};
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
            }
            let mut msgs = vec![];
            swap(&mut self.device.output, &mut msgs);
            for output in msgs {
                match output {
                    Output::OutputStd(text) => {
                        stdout()
                            .execute(SetForegroundColor(Color::White))
                            .expect("Error setting foreground color")
                            .execute(Print(text))
                            .expect("Error printing output");
                    }
                    Output::OutputErr(text) => {
                        stdout()
                            .execute(SetForegroundColor(Color::Red))
                            .expect("Error setting foreground color")
                            .execute(Print(text))
                            .expect("Error printing error output");
                    }
                    Output::RequestInputChr => {
                        let chr = self.read_char().expect("Error reading input (char)");
                        self.device.keyboard_buffer.push(chr);
                    }
                    Output::RequestInputStr => {
                        let mut line = String::new();
                        let len = stdin()
                            .read_line(&mut line)
                            .expect("Error reading input (str)");
                        let bytes = if len > 255 {
                            line.into_bytes().iter().take(255).cloned().collect()
                        } else {
                            line.into_bytes()
                        };
                        self.device.keyboard_buffer.extend_from_slice(&bytes);
                    }
                    Output::BreakpointHit(_) => panic!("Encountered and stopped for breakpoint"),
                    Output::Status(_) => panic!("Unexpected system dump"),
                    Output::OutputMem(_, _) => panic!("Unexpected system memory"),
                }
            }
        }
    }

    fn read_char(&self) -> Result<u8> {
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
                    KeyCode::Backspace => {
                        char[0] = 8;
                        break;
                    }
                    KeyCode::Tab => {
                        char[0] = 9;
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
        Ok(char[0])
    }
}
