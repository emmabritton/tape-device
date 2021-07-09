mod execution;
mod parsing;

extern crate tape_device;

use tape_device::device::comm::Output;
use tape_device::device::comm::Output::OutputErr;
use tape_device::device::internals::{Device, RunResult};
use tape_device::device::Dump;

fn setup(ops: Vec<u8>) -> Device {
    let device = Device::new(ops, vec![], vec![], vec![]);

    assert_eq!(device.dump(), Dump::default());

    device
}

fn assert_no_output(device: Device) {
    for output in device.output {
        match output {
            Output::OutputStd(msg) => panic!("Expected no output but got: {}", msg),
            Output::OutputErr(msg) => panic!("Expected no error output but got: {}", msg),
            _ => {}
        }
    }
}

fn assert_specific_output(device: Device, msg: &str) {
    let mut output_text = String::new();
    for output in device.output {
        match output {
            Output::OutputStd(msg) => output_text.push_str(&msg),
            Output::OutputErr(msg) => panic!("Expected no error output but got: {}", msg),
            _ => {}
        }
    }
    assert_eq!(output_text.as_str(), msg);
}

fn assert_step_device(name: &str, device: &mut Device, dump: Dump) {
    let result = device.step(true);
    if result == RunResult::ProgError {
        for output in &device.output {
            if let OutputErr(text) = output {
                eprintln!("{}", text);
            }
        }
        panic!("step for {}", name);
    }
    assert_eq!(result, RunResult::Pause);
    assert_eq!(device.dump(), dump, "dump for {}", name);
}

fn assert_memory(device: &Device, start: usize, target: &[u8]) {
    assert_eq!(&device.mem[start..(start + target.len())], target);
}
