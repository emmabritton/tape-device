use crate::{assert_specific_output, assert_step_device, setup};
use tape_device::constants::code::{
    DEBUG, HALT, NOP, RAND_REG, SEED_REG, SWP_AREG_AREG, SWP_REG_REG, TIME,
};
use tape_device::constants::hardware::{REG_A0, REG_A1, REG_D0, REG_D1};
use tape_device::device::internals::{Device, RunResult};
use tape_device::device::Dump;

#[test]
#[rustfmt::skip]
fn test_multiple_misc_ops() {
    let ops = vec![
        NOP,
        SEED_REG, REG_D0,
        RAND_REG, REG_D0,
        SWP_REG_REG, REG_D0, REG_D1,
        SWP_AREG_AREG, REG_A0, REG_A1,
        DEBUG,
        TIME,
        HALT
    ];
    let mut device = setup(ops);
    device.data_reg = [41, 0, 0, 0];
    device.addr_reg = [304, 0];

    assert_step_device("NOP", &mut device, Dump { pc: 1, data_reg: [41, 0, 0, 0], addr_reg: [304, 0], ..Default::default() });
    assert_step_device("SEED D0", &mut device, Dump { pc: 3, data_reg: [41, 0, 0, 0], addr_reg: [304, 0], ..Default::default() });
    assert_step_device("RAND D0", &mut device, Dump { pc: 5, data_reg: [110, 0, 0, 0], addr_reg: [304, 0], ..Default::default() });
    assert_step_device("SWP D0 D1", &mut device, Dump { pc: 8, data_reg: [0, 110, 0, 0], addr_reg: [304, 0], ..Default::default() });
    assert_step_device("SWP A0 A1", &mut device, Dump { pc: 11, data_reg: [0, 110, 0, 0], addr_reg: [0, 304], ..Default::default() });
    assert_step_device("DEBUG", &mut device, Dump { pc: 12, data_reg: [0, 110, 0, 0], addr_reg: [0, 304], ..Default::default() });

    //TIME has to be handled manually as the result isn't known
    assert_eq!(device.step(true), RunResult::Pause);
    assert_eq!(device.dump().pc, 13);
    validate(&mut device);

    assert_eq!(device.step(true), RunResult::Halt);
    assert_eq!(device.dump().pc, 13);
    validate(&mut device);

    assert_specific_output(device, "ACC: 00  D0: 00  D1: 6E  D2: 00  D3: 00 A0: 0000 A1: 0130PC:   11 SP: FFFF FP: FFFF Overflowed: falseStack (FFFF..FFFF): []");
}

fn validate(device: &mut Device) {
    let dump = device.dump();
    assert_eq!(dump.addr_reg, [0, 304]);
    assert!(!dump.overflow);
    assert_eq!(dump.sp, 65535);
    assert_eq!(dump.fp, 65535);
    assert_eq!(dump.acc, 0);
    assert!(dump.data_reg[0] < 60, "{}", dump.data_reg[0]);
    assert!(dump.data_reg[1] < 60, "{}", dump.data_reg[1]);
    assert!(dump.data_reg[2] < 24, "{}", dump.data_reg[2]);
    assert_eq!(dump.data_reg[3], 0);
}
