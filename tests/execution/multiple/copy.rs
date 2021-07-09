use crate::{assert_no_output, assert_step_device, setup};
use tape_device::constants::code::{
    CPY_AREG_ADDR, CPY_AREG_AREG, CPY_AREG_REG_REG, CPY_REG_AREG, CPY_REG_REG, CPY_REG_REG_AREG,
    CPY_REG_VAL,
};
use tape_device::constants::hardware::{REG_A0, REG_A1, REG_ACC, REG_D0, REG_D1, REG_D2, REG_D3};
use tape_device::device::Dump;

#[test]
#[rustfmt::skip]
fn test_multiple_copy_ops() {
    let ops = vec![
        CPY_REG_VAL, REG_D0, 10,
        CPY_REG_VAL, REG_D1, 20,
        CPY_AREG_ADDR, REG_A0, 0, 255,
        CPY_REG_REG, REG_ACC, REG_D0,
        CPY_AREG_AREG, REG_A1, REG_A0,
        CPY_REG_REG_AREG, REG_D2, REG_D3, REG_A1,
        CPY_AREG_REG_REG, REG_A1, REG_D0, REG_D1,
        CPY_AREG_ADDR, REG_A0, 0, 2,
        CPY_REG_AREG, REG_D0, REG_A0
    ];
    let mut device = setup(ops);
    device.tape_data = vec![1, 1, 50];

    assert_step_device("CPY D0 10", &mut device, Dump { pc: 3, data_reg: [10, 0, 0, 0], ..Default::default() });
    assert_step_device("CPY D1 20", &mut device, Dump { pc: 6, data_reg: [10, 20, 0, 0], ..Default::default() });
    assert_step_device("CPY A0 @255", &mut device, Dump { pc: 10, data_reg: [10, 20, 0, 0], addr_reg: [255, 0], ..Default::default() });
    assert_step_device("CPY ACC D0", &mut device, Dump { pc: 13, acc: 10, data_reg: [10, 20, 0, 0], addr_reg: [255, 0], ..Default::default() });
    assert_step_device("CPY A1 A0", &mut device, Dump { pc: 16, acc: 10, data_reg: [10, 20, 0, 0], addr_reg: [255, 255], ..Default::default() });
    assert_step_device("CPY D2 D3 A1", &mut device, Dump { pc: 20, acc: 10, data_reg: [10, 20, 0, 255], addr_reg: [255, 255], ..Default::default() });
    assert_step_device("CPY A1 D0 D1", &mut device, Dump { pc: 24, acc: 10, data_reg: [10, 20, 0, 255], addr_reg: [255, 2580], ..Default::default() });
    assert_step_device("CPY A0 @3", &mut device, Dump { pc: 28, acc: 10, data_reg: [10, 20, 0, 255], addr_reg: [2, 2580], ..Default::default() });
    assert_step_device("CPY D0 A0", &mut device, Dump { pc: 31, acc: 10, data_reg: [50, 20, 0, 255], addr_reg: [2, 2580], ..Default::default() });

    assert_no_output(device);
}
