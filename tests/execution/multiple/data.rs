use crate::{assert_no_output, assert_step_device, setup};
use tape_device::constants::code::{
    CPY_REG_AREG, LD_AREG_DATA_REG_REG, LD_AREG_DATA_REG_VAL, LD_AREG_DATA_VAL_REG,
    LD_AREG_DATA_VAL_VAL,
};
use tape_device::constants::hardware::{REG_A0, REG_A1, REG_ACC, REG_D0, REG_D1, REG_D2, REG_D3};
use tape_device::device::Dump;

#[test]
#[rustfmt::skip]
fn test_multiple_data_ops() {
    let ops = vec![
        LD_AREG_DATA_VAL_VAL, REG_A0, 0, 0, 0, 0,
        LD_AREG_DATA_VAL_VAL, REG_A0, 0, 0, 0, 3,
        LD_AREG_DATA_VAL_VAL, REG_A1, 0, 0, 0, 2,
        CPY_REG_AREG, REG_ACC, REG_A0,
        CPY_REG_AREG, REG_ACC, REG_A1,
        LD_AREG_DATA_VAL_VAL, REG_A0, 0, 0, 1, 0,
        CPY_REG_AREG, REG_ACC, REG_A0,
        LD_AREG_DATA_VAL_VAL, REG_A1, 0, 8, 2, 1,
        CPY_REG_AREG, REG_ACC, REG_A1,
        LD_AREG_DATA_VAL_VAL, REG_A1, 0, 0, 2, 1,
        LD_AREG_DATA_REG_REG, REG_A1, 0, 0, REG_D0, REG_D2,
        CPY_REG_AREG, REG_D3, REG_A1,
        LD_AREG_DATA_VAL_REG, REG_A0, 0, 0, 0, REG_D0,
        CPY_REG_AREG, REG_D2, REG_A0,
        LD_AREG_DATA_REG_VAL, REG_A1, 0, 8, REG_D0, 0,
        CPY_REG_AREG, REG_D1, REG_A1,
    ];
    let mut device = setup(ops);
    device.tape_data = vec![3, 1, 2, 1, 50, 10, 11, 100, 2, 2, 2, 40, 41, 50, 51]; //keys = dk1, dk2

    assert_step_device("LD A0 dk1 0 0", &mut device, Dump { pc: 6, ..Default::default() });
    assert_step_device("LD A0 dk1 0 1", &mut device, Dump { pc: 12, addr_reg: [3, 0], ..Default::default() });
    assert_step_device("LD A1 dk1 0 2", &mut device, Dump { pc: 18, addr_reg: [3, 2], ..Default::default() });
    assert_step_device("CPY ACC A0", &mut device, Dump { pc: 21, acc: 1, addr_reg: [3, 2], ..Default::default() });
    assert_step_device("CPY ACC A1", &mut device, Dump { pc: 24, acc: 2, addr_reg: [3, 2], ..Default::default() });
    assert_step_device("LD A0 dk1 1 0", &mut device, Dump { pc: 30, acc: 2, addr_reg: [4, 2], ..Default::default() });
    assert_step_device("CPY ACC A0", &mut device, Dump { pc: 33, acc: 50, addr_reg: [4, 2], ..Default::default() });
    assert_step_device("LD A1 dk2 2 1", &mut device, Dump { pc: 39, acc: 50, addr_reg: [4, 14], ..Default::default() });
    assert_step_device("CPY ACC A1", &mut device, Dump { pc: 42, acc: 51, addr_reg: [4, 14], ..Default::default() });

    device.data_reg = [2, 3, 1, 0];
    assert_step_device("LD A1 dk1 2 1", &mut device, Dump { pc: 48, acc: 51, data_reg: [2,3,1,0], addr_reg: [4, 6], ..Default::default() });
    assert_step_device("LD A1 dk1 D0 D2", &mut device, Dump { pc: 54, acc: 51, data_reg: [2,3,1,0], addr_reg: [4, 6], ..Default::default() });
    assert_step_device("CPY D3 A1", &mut device, Dump { pc: 57, acc: 51, data_reg: [2,3,1,11], addr_reg: [4, 6], ..Default::default() });
    assert_step_device("LD A0 dk1 0 D0", &mut device, Dump { pc: 63, acc: 51, data_reg: [2,3,1,11], addr_reg: [2, 6], ..Default::default() });
    assert_step_device("CPY D2 A0", &mut device, Dump { pc: 66, acc: 51, data_reg: [2,3,2,11], addr_reg: [2, 6], ..Default::default() });
    assert_step_device("LD A1 dk2 D0 0", &mut device, Dump { pc: 72, acc: 51, data_reg: [2,3,2,11], addr_reg: [2, 13], ..Default::default() });
    assert_step_device("CPY D1 A1", &mut device, Dump { pc: 75, acc: 51, data_reg: [2,50,2,11], addr_reg: [2, 13], ..Default::default() });


    assert_no_output(device);
}
