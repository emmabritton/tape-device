use crate::{assert_no_output, assert_step_device, setup};
use tape_device::constants::code::{
    CMP_AREG_ADDR, CMP_AREG_AREG, CMP_AREG_REG_REG, CMP_REG_AREG, CMP_REG_REG, CMP_REG_REG_AREG,
    CMP_REG_VAL,
};
use tape_device::constants::compare::{EQUAL, GREATER, LESSER};
use tape_device::constants::hardware::{REG_A0, REG_A1, REG_ACC, REG_D0, REG_D1, REG_D2, REG_D3};
use tape_device::device::Dump;

#[test]
#[rustfmt::skip]
fn test_multiple_compare_ops() {
    let ops = vec![
        CMP_REG_VAL, REG_D0, 10,
        CMP_REG_VAL, REG_D1, 10,
        CMP_REG_VAL, REG_D2, 10,
        CMP_REG_REG, REG_D0, REG_D2,
        CMP_REG_REG, REG_D1, REG_D2,
        CMP_REG_REG, REG_D2, REG_D2,
        CMP_AREG_AREG, REG_A0, REG_A1,
        CMP_AREG_ADDR, REG_A0, 0, 200,
        CMP_AREG_ADDR, REG_A0, 1, 244,
        CMP_AREG_ADDR, REG_A0, 4, 0,
        CMP_AREG_REG_REG, REG_A0, REG_D3, REG_D1,
        CMP_AREG_REG_REG, REG_A0, REG_D1, REG_D3,
        CMP_REG_REG_AREG, REG_D3, REG_D1, REG_A0,
        CMP_REG_REG_AREG, REG_D1, REG_D3, REG_A0,
        CMP_AREG_REG_REG, REG_A1, REG_D3, REG_D1,
        CMP_REG_AREG, REG_D1, REG_A1,
        CMP_REG_VAL, REG_ACC, 10,
    ];
    let mut device = setup(ops);
    device.tape_data = vec![1, 1, 50];
    device.data_reg[1] = 20;
    device.data_reg[2] = 10;
    device.addr_reg[0] = 500;
    device.addr_reg[1] = 2;

    assert_step_device("CMP D0 10", &mut device, Dump { pc: 3, acc: LESSER, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });
    assert_step_device("CMP D1 10", &mut device, Dump { pc: 6, acc: GREATER, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });
    assert_step_device("CMP D2 10", &mut device, Dump { pc: 9, acc: EQUAL, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });
    assert_step_device("CMP D0 D2", &mut device, Dump { pc: 12, acc: LESSER, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });
    assert_step_device("CMP D1 D2", &mut device, Dump { pc: 15, acc: GREATER, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });
    assert_step_device("CMP D2 D2", &mut device, Dump { pc: 18, acc: EQUAL, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });
    assert_step_device("CMP A0 A1", &mut device, Dump { pc: 21, acc: GREATER, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });
    assert_step_device("CMP A0 @200", &mut device, Dump { pc: 25, acc: GREATER, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });
    assert_step_device("CMP A0 @500", &mut device, Dump { pc: 29, acc: EQUAL, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });
    assert_step_device("CMP A0 @1024", &mut device, Dump { pc: 33, acc: LESSER, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });
    assert_step_device("CMP A0 D3 D1", &mut device, Dump { pc: 37, acc: GREATER, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });
    assert_step_device("CMP A0 D1 D3", &mut device, Dump { pc: 41, acc: LESSER, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });
    assert_step_device("CMP D3 D1 A0", &mut device, Dump { pc: 45, acc: LESSER, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });
    assert_step_device("CMP D1 D3 A0", &mut device, Dump { pc: 49, acc: GREATER, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });
    assert_step_device("CMP A1 D3 D1", &mut device, Dump { pc: 53, acc: LESSER, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });
    assert_step_device("CMP D1 A1", &mut device, Dump { pc: 56, acc: LESSER, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });
    assert_step_device("CMP ACC 10", &mut device, Dump { pc: 59, acc: LESSER, data_reg: [0, 20, 10, 0], addr_reg: [500, 2], ..Default::default() });

    assert_no_output(device);
}
