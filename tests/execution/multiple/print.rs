use crate::{assert_specific_output, assert_step_device, setup};
use tape_device::constants::code::{
    PRTC_AREG, PRTC_REG, PRTC_VAL, PRTD_AREG, PRTLN, PRTS_STR, PRT_AREG, PRT_REG, PRT_VAL,
};
use tape_device::constants::hardware::{REG_A0, REG_A1, REG_D0, REG_D2};
use tape_device::device::Dump;

#[test]
#[rustfmt::skip]
fn test_multiple_print_ops() {
    let ops = vec![
        PRT_VAL, 97,
        PRTC_VAL, 97,
        PRT_REG, REG_D0,
        PRTC_REG, REG_D2,
        PRTLN,
        PRTS_STR, 0, 0,
        PRT_AREG, REG_A0,
        PRTC_AREG, REG_A1,
        PRTD_AREG, REG_A1
    ];
    let mut device = setup(ops);

    device.tape_strings = vec![5, 87, 111, 114, 108, 100];
    device.tape_data = vec![1, 4, 50, 98, 99, 2];

    assert_step_device("PRT 97", &mut device, Dump { pc: 2, ..Default::default() });
    assert_step_device("PRTC 97", &mut device, Dump { pc: 4, ..Default::default() });

    device.data_reg = [68, 0, 70, 0];

    assert_step_device("PRT D0", &mut device, Dump { pc: 6, data_reg: [68, 0, 70, 0], ..Default::default() });
    assert_step_device("PRTC D2", &mut device, Dump { pc: 8, data_reg: [68, 0, 70, 0], ..Default::default() });
    assert_step_device("PRTLN", &mut device, Dump { pc: 9, data_reg: [68, 0, 70, 0], ..Default::default() });
    assert_step_device("PRTS 0", &mut device, Dump { pc: 12, data_reg: [68, 0, 70, 0], ..Default::default() });

    device.addr_reg = [2, 3];

    assert_step_device("PRT A0", &mut device, Dump { pc: 14, data_reg: [68, 0, 70, 0], addr_reg: [2, 3], ..Default::default() });
    assert_step_device("PRTC A1", &mut device, Dump { pc: 16, data_reg: [68, 0, 70, 0], addr_reg: [2, 3], ..Default::default() });

    device.acc = 2;

    assert_step_device("PRTD A1", &mut device, Dump { pc: 18, acc: 2, data_reg: [68, 0, 70, 0], addr_reg: [2, 3], ..Default::default() });

    assert_specific_output(device, "97a68F\nWorld50bbc");
}
