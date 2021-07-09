use crate::{assert_no_output, assert_step_device, setup};
use tape_device::constants::code::{HALT, JE_ADDR, JMP_ADDR, JMP_AREG};
use tape_device::constants::compare::EQUAL;
use tape_device::constants::hardware::REG_A0;
use tape_device::device::Dump;

#[test]
#[rustfmt::skip]
fn test_multiple_jump_ops() {
    let ops = vec![
        JMP_ADDR, 0, 4,
        HALT,
        JMP_AREG, REG_A0,
        HALT,
        JE_ADDR, 0, 10,

    ];
    let mut device = setup(ops);

    assert_step_device("JMP @4", &mut device, Dump { pc: 4, ..Default::default() });

    device.addr_reg = [7,0];
    device.acc = EQUAL;

    assert_no_output(device);
}
