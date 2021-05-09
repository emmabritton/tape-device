pub mod internals;

use crate::device::internals::Device;
use crate::printer::StdoutPrinter;
use crate::tape_reader::read_tape;
use anyhow::Result;
use std::path::Path;

pub fn start(path: &str, input_path: Option<&str>) -> Result<()> {
    let tape = read_tape(path)?;

    if let Some(input_file) = input_path {
        if !Path::new(input_file).exists() {
            eprintln!("No file found at {}", input_file);
            return Ok(());
        }
    }

    println!("Running {} v{}", tape.name, tape.version);

    let mut device = Device::new(
        tape.ops,
        tape.data,
        input_path.map(|str| str.to_string()),
        StdoutPrinter::new(),
    );
    device.run();

    Ok(())
}
