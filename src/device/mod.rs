mod debug_device;
mod input;
pub mod internals;
mod piped_device;
mod std_device;
mod util;

use crate::constants::hardware::{ADDR_REG_COUNT, DATA_REG_COUNT, RAM_SIZE};
use crate::device::debug_device::{setup_terminal, shutdown_terminal, DebugDevice};
use crate::device::piped_device::PipedDevice;
use crate::device::std_device::StdDevice;
use crate::tape_reader::read_tape;
use anyhow::Result;
use std::fs::read_to_string;

pub fn start(path: &str, input_paths: Vec<&str>) -> Result<()> {
    let tape = read_tape(path)?;

    println!("Running {} v{}", tape.name, tape.version);

    let mut device = StdDevice::new(
        tape.ops,
        tape.strings,
        tape.data,
        input_paths.iter().map(|str| str.to_string()).collect(),
    );
    device.run();

    Ok(())
}

pub fn start_piped(path: &str, input_paths: Vec<&str>) -> Result<()> {
    let tape = read_tape(path)?;

    let mut device = PipedDevice::new(
        tape.ops,
        tape.strings,
        tape.data,
        input_paths.iter().map(|str| str.to_string()).collect(),
    );
    device.run();

    Ok(())
}

pub fn start_debug(path: &str, debug_path: &str, input_paths: Vec<&str>) -> Result<()> {
    let tape = read_tape(path)?;
    let debug_info_text = read_to_string(debug_path).expect("Unable to read debug info file");
    let debug_info = serde_json::from_str(&debug_info_text).expect("Unable to parse debug info");

    let mut device = DebugDevice::new(
        tape.ops,
        tape.strings,
        tape.data,
        debug_info,
        input_paths.iter().map(|str| str.to_string()).collect(),
    );

    setup_terminal()?;
    device.run()?;
    shutdown_terminal();

    Ok(())
}

pub mod comm {
    pub enum Output {
        OutputStd(String),
        OutputErr(String),
        BreakpointHit(u16),
    }
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
            sp: RAM_SIZE as u16,
            fp: RAM_SIZE as u16,
            data_reg: [0, 0, 0, 0],
            addr_reg: [0, 0],
            overflow: false,
        }
    }
}
