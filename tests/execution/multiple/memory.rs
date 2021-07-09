use crate::{assert_memory, assert_specific_output, assert_step_device, setup};
use tape_device::constants::code::{
    MEMP_ADDR, MEMP_AREG, MEMR_ADDR, MEMR_AREG, MEMW_ADDR, MEMW_AREG,
};
use tape_device::constants::hardware::{REG_A0, REG_A1};
use tape_device::device::Dump;

#[test]
#[rustfmt::skip]
fn test_multiple_memory_ops() {
    let ops = vec![
        MEMW_ADDR, 0, 0,
        MEMW_AREG, REG_A0,
        MEMR_ADDR, 0, 10,
        MEMR_AREG, REG_A1,
        MEMP_ADDR, 0, 40,
        MEMP_AREG, REG_A1,
    ];
    let mut device = setup(ops);

    let empty_memory = vec![0; 65535];
    assert_memory(&device, 0, &empty_memory);

    assert_step_device("MEMW @0", &mut device, Dump { pc: 3, ..Default::default() });
    assert_memory(&device, 0, &[0]);
    device.acc = 5;
    assert_step_device("MEMW A0", &mut device, Dump { pc: 5, acc: 5, ..Default::default() });
    assert_memory(&device, 0, &[5, 0, 0, 0, 0]);
    assert_step_device("MEMR @0", &mut device, Dump { pc: 8, ..Default::default() });
    assert_step_device("MEMR A1", &mut device, Dump { pc: 10, acc: 5, ..Default::default() });
    assert_memory(&device, 0, &[5, 0, 0, 0, 0]);
    device.mem[40] = b'H';
    device.mem[41] = b'e';
    device.mem[42] = b'l';
    device.mem[43] = b'l';
    device.mem[44] = b'o';
    device.addr_reg = [40, 42];
    device.acc = 5;
    assert_step_device("MEMP @40", &mut device, Dump { pc: 13, acc: 5, addr_reg: [40, 42], ..Default::default()});
    assert_step_device("MEMP A1", &mut device, Dump { pc: 15, acc: 5, addr_reg: [40, 42], ..Default::default()});

    assert_specific_output(device, "Hellollo\u{0}\u{0}");
}
