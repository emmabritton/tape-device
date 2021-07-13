pub mod internals;
mod piped_device;
mod std_device;

use crate::constants::hardware::{ADDR_REG_COUNT, DATA_REG_COUNT, RAM_SIZE};
use crate::device::piped_device::PipedDevice;
use crate::device::std_device::StdDevice;
use crate::tape_reader::read_tape;
use anyhow::Result;

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
