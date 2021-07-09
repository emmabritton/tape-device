use crate::{assert_no_output, assert_step_device, setup};
use tape_device::constants::code::{
    AND_REG_AREG, AND_REG_REG, AND_REG_VAL, NOT_REG, OR_REG_AREG, OR_REG_REG, OR_REG_VAL,
    XOR_REG_AREG, XOR_REG_REG,
};
use tape_device::constants::hardware::{REG_A0, REG_A1, REG_ACC, REG_D0, REG_D1, REG_D2, REG_D3};
use tape_device::device::Dump;

#[test]
#[rustfmt::skip]
fn test_multiple_logic_ops() {
    let ops = vec![
        AND_REG_REG, REG_D0, REG_D1,
        AND_REG_VAL, REG_D3, 41,
        AND_REG_AREG, REG_ACC, REG_A0,
        XOR_REG_REG, REG_D3, REG_ACC,
        XOR_REG_REG, REG_ACC, REG_ACC,
        XOR_REG_AREG, REG_D3, REG_A0,
        OR_REG_REG, REG_D0, REG_D1,
        OR_REG_VAL, REG_D3, 40,
        OR_REG_AREG, REG_ACC, REG_A1,
        NOT_REG, REG_D2
    ];
    let mut device = setup(ops);
    device.tape_data = vec![1, 1, 32];
    device.data_reg = [10, 20, 30, 40];
    device.addr_reg = [2, 0];

    assert_step_device("AND D0 D1", &mut device, Dump { pc: 3, acc: 0, data_reg: [10, 20, 30, 40], addr_reg: [2, 0], ..Default::default() });
    assert_step_device("AND D0 41", &mut device, Dump { pc: 6, acc: 40, data_reg: [10, 20, 30, 40], addr_reg: [2, 0], ..Default::default() });
    assert_step_device("AND ACC A0", &mut device, Dump { pc: 9, acc: 32, data_reg: [10, 20, 30, 40], addr_reg: [2, 0], ..Default::default() });
    assert_step_device("XOR D3 ACC", &mut device, Dump { pc: 12, acc: 8, data_reg: [10, 20, 30, 40], addr_reg: [2, 0], ..Default::default() });
    assert_step_device("XOR ACC ACC", &mut device, Dump { pc: 15, acc: 0, data_reg: [10, 20, 30, 40], addr_reg: [2, 0], ..Default::default() });
    assert_step_device("XOR D3 A0", &mut device, Dump { pc: 18, acc: 8, data_reg: [10, 20, 30, 40], addr_reg: [2, 0], ..Default::default() });
    assert_step_device("OR D0 D1", &mut device, Dump { pc: 21, acc: 30, data_reg: [10, 20, 30, 40], addr_reg: [2, 0], ..Default::default() });
    assert_step_device("OR D3 40", &mut device, Dump { pc: 24, acc: 40, data_reg: [10, 20, 30, 40], addr_reg: [2, 0], ..Default::default() });
    assert_step_device("OR ACC A1", &mut device, Dump { pc: 27, acc: 41, data_reg: [10, 20, 30, 40], addr_reg: [2, 0], ..Default::default() });
    assert_step_device("NOT D2", &mut device, Dump { pc: 29, acc: 225, data_reg: [10, 20, 30, 40], addr_reg: [2, 0], ..Default::default() });

    assert_no_output(device);
}
