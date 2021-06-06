pub mod internals;

use crate::device::internals::Device;
use crate::printer::StdoutPrinter;
use crate::tape_reader::read_tape;
use anyhow::Result;

pub fn start(path: &str, input_paths: Vec<&str>) -> Result<()> {
    let tape = read_tape(path)?;

    println!("Running {} v{}", tape.name, tape.version);

    let mut device = Device::new(
        tape.ops,
        tape.strings,
        tape.data,
        input_paths.iter().map(|str| str.to_string()).collect(),
        StdoutPrinter::new(),
    );
    device.run();

    Ok(())
}
