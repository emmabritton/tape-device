use crate::{assert_no_output, assert_step_device, setup};
use tape_device::constants::code::{
    ADD_REG_AREG, ADD_REG_REG, ADD_REG_VAL, DEC_REG, INC_REG, SUB_REG_AREG, SUB_REG_REG,
};
use tape_device::constants::hardware::{REG_A0, REG_A1, REG_ACC, REG_D0, REG_D1, REG_D3};
use tape_device::device::Dump;

#[test]
#[rustfmt::skip]
fn test_multiple_math_ops() {
    let ops = vec![
        INC_REG, REG_D0,
        INC_REG, REG_D1,
        ADD_REG_REG, REG_D0, REG_D1,
        ADD_REG_VAL, REG_D3, 40,
        DEC_REG, REG_ACC,
        DEC_REG, REG_ACC,
        DEC_REG, REG_D3,
        SUB_REG_REG, REG_D3, REG_ACC,
        INC_REG, REG_A0,
        INC_REG, REG_A0,
        DEC_REG, REG_A1,
        SUB_REG_REG, REG_ACC, REG_ACC,
        ADD_REG_AREG, REG_ACC, REG_A0,
        SUB_REG_AREG, REG_D3, REG_A0,
    ];
    let mut device = setup(ops);
    device.tape_data = vec![1,1,4];

    assert_step_device("INC D0", &mut device, Dump { pc: 2, data_reg: [1, 0, 0, 0], ..Default::default() });
    assert_step_device("INC D1", &mut device, Dump { pc: 4, data_reg: [1, 1, 0, 0], ..Default::default() });
    assert_step_device("ADD D0 D1", &mut device, Dump { pc: 7, acc: 2, data_reg: [1, 1, 0, 0], ..Default::default() });
    assert_step_device("ADD D3 40", &mut device, Dump { pc: 10, acc: 40, data_reg: [1, 1, 0, 0], ..Default::default() });
    assert_step_device("DEC ACC", &mut device, Dump { pc: 12, acc: 39, data_reg: [1, 1, 0, 0], ..Default::default() });
    assert_step_device("DEC ACC", &mut device, Dump { pc: 14, acc: 38, data_reg: [1, 1, 0, 0], ..Default::default() });
    assert_step_device("DEC D3", &mut device, Dump { pc: 16, acc: 38, data_reg: [1, 1, 0, 255],  overflow: true, ..Default::default() });
    assert_step_device("SUB D3 ACC", &mut device, Dump { pc: 19, acc: 217, data_reg: [1, 1, 0, 255], ..Default::default() });
    assert_step_device("INC A0", &mut device, Dump { pc: 21, acc: 217, data_reg: [1, 1, 0, 255], addr_reg: [1,0], ..Default::default() });
    assert_step_device("INC A0", &mut device, Dump { pc: 23, acc: 217, data_reg: [1, 1, 0, 255], addr_reg: [2,0], ..Default::default() });
    assert_step_device("DEC A1", &mut device, Dump { pc: 25, acc: 217, data_reg: [1, 1, 0, 255], addr_reg: [2,65535], overflow: true, ..Default::default() });
    assert_step_device("SUB ACC ACC", &mut device, Dump { pc: 28, acc: 0, data_reg: [1, 1, 0, 255], addr_reg: [2,65535], ..Default::default() });
    assert_step_device("ADD ACC A0", &mut device, Dump { pc: 31, acc: 4, data_reg: [1, 1, 0, 255], addr_reg: [2,65535], ..Default::default() });
    assert_step_device("SUB D3 A0", &mut device, Dump { pc: 34, acc: 251, data_reg: [1, 1, 0, 255], addr_reg: [2,65535], ..Default::default() });

    assert_no_output(device);
}
