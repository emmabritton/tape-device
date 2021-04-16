mod internals;

use crate::device::internals::Device;
use crate::printer::StdoutPrinter;
use crate::tape_reader::read_tape;
use anyhow::Result;

pub fn start(path: &str, input_path: Option<&str>, debug_pc: bool) -> Result<()> {
    let tape = read_tape(path)?;

    println!("Running {} v{}", tape.name, tape.version);

    let mut device = Device::new(
        tape.ops,
        tape.data,
        input_path.map(|str| str.to_string()),
        StdoutPrinter::boxed(),
    );
    device.debug.pc = debug_pc;
    device.run();

    Ok(())
}
