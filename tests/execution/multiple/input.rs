use crate::{setup, assert_step_device, assert_no_output, assert_memory};
use tape_device::constants::code::{IPOLL_ADDR, RCHR_REG, RSTR_AREG, HALT, IPOLL_AREG, RSTR_ADDR};
use tape_device::device::Dump;
use tape_device::constants::hardware::{REG_ACC, REG_A0, REG_A1};
use tape_device::device::internals::RunResult;

#[test]
#[rustfmt::skip]
fn test_input() {
    let ops = vec![
        IPOLL_ADDR, 255, 255,
        RCHR_REG, REG_ACC,
        RSTR_AREG, REG_A0,
        IPOLL_ADDR, 0, 11,
        HALT,
        RSTR_AREG, REG_A1,
        IPOLL_AREG, REG_A1,
        HALT,
        RSTR_ADDR, 0, 100,
    ];
    let mut device = setup(ops);

    assert_step_device("IPOLL @xFFFF", &mut device, Dump { pc: 3, ..Default::default() });

    assert_eq!(device.step(true), RunResult::CharInputRequested, "RCHR ACC");
    assert_eq!(device.dump(), Dump { pc: 3, ..Default::default() });
    device.keyboard_buffer = vec![b'a'];
    assert_step_device("RCHR ACC", &mut device, Dump { pc: 5, acc: 97, ..Default::default() });
    assert_eq!(device.keyboard_buffer, Vec::<u8>::new());

    assert_eq!(device.step(true), RunResult::StringInputRequested, "RSTR A0");
    assert_eq!(device.dump(), Dump { pc: 5, acc: 97, ..Default::default() });
    assert_eq!(device.keyboard_buffer, Vec::<u8>::new());

    device.keyboard_buffer = vec![b'H', b'i'];
    assert_step_device("RSTR A0", &mut device, Dump { pc: 7, acc: 2, ..Default::default() });
    assert_eq!(device.keyboard_buffer, Vec::<u8>::new());
    assert_memory(&device, 0, &[b'H', b'i']);

    device.addr_reg[1] = 16;
    device.keyboard_buffer = vec![b'T', b'e', b's', b't'];
    assert_step_device("IPOLL @11", &mut device, Dump { pc: 11, acc: 2, addr_reg: [0, 16], ..Default::default() });
    assert_step_device("RSTR A1", &mut device, Dump { pc: 13, acc: 4, addr_reg: [0, 16], ..Default::default() });
    assert_eq!(device.keyboard_buffer, Vec::<u8>::new());
    assert_memory(&device, 0, &[b'H', b'i']);
    assert_memory(&device, 16, &[b'T', b'e', b's', b't']);

    device.keyboard_buffer = vec![b'F', b'i', b'n'];
    assert_step_device("IPOLL A1", &mut device, Dump { pc: 16, acc: 4, addr_reg: [0, 16], ..Default::default() });
    assert_step_device("RSTR @100", &mut device, Dump { pc: 19, acc: 3, addr_reg: [0, 16], ..Default::default() });
    assert_eq!(device.keyboard_buffer, Vec::<u8>::new());
    assert_memory(&device, 0, &[b'H', b'i']);
    assert_memory(&device, 16, &[b'T', b'e', b's', b't']);
    assert_memory(&device, 100, &[b'F', b'i', b'n']);


    assert_no_output(device);
}