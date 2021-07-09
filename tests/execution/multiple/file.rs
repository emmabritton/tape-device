use crate::{assert_memory, assert_no_output, assert_step_device};
use std::fs::{remove_file, File};
use std::io::Write;
use tape_device::constants::code::{
    FCHK_REG_ADDR, FCHK_REG_AREG, FCHK_VAL_ADDR, FCHK_VAL_AREG, FILER_REG_ADDR, FILER_REG_AREG,
    FILER_VAL_ADDR, FILEW_REG_REG, FILEW_REG_VAL, FILEW_VAL_ADDR, FILEW_VAL_REG, FILEW_VAL_VAL,
    FOPEN_REG, FOPEN_VAL, FSEEK_REG, FSEEK_VAL, FSKIP_VAL_VAL, HALT, PUSH_VAL,
};
use tape_device::constants::hardware::{REG_A0, REG_A1, REG_ACC, REG_D0, REG_D1, REG_D3};
use tape_device::device::internals::Device;
use tape_device::device::Dump;
use tempfile::tempdir;

#[test]
#[rustfmt::skip]
fn test_file_check_ops() {
    let path = setup_test_file("-0");

    let ops = vec![
        FCHK_VAL_ADDR, 0, 0, 5,
        HALT,
        FCHK_VAL_AREG, 0, REG_A0,
        HALT,
        FCHK_REG_ADDR, REG_ACC, 0, 14,
        HALT,
        FCHK_REG_AREG, REG_D1, REG_A1,
        HALT,
        FOPEN_VAL, 0,
    ];

    let mut device = Device::new(ops, vec![], vec![], vec![path]);

    device.addr_reg = [9, 18];

    assert_step_device("FCHK 0 @5", &mut device, Dump { pc: 5, addr_reg: [9, 18], ..Default::default() });
    assert_step_device("FCHK 0 A0", &mut device, Dump { pc: 9, addr_reg: [9, 18], ..Default::default() });
    assert_step_device("FCHK ACC @14", &mut device, Dump { pc: 14, addr_reg: [9, 18], ..Default::default() });
    assert_step_device("FCHK D1 A1", &mut device, Dump { pc: 18, addr_reg: [9, 18], ..Default::default() });
    assert_step_device("FOPEN 0", &mut device, Dump { pc: 20, data_reg: [0, 0, 0, 6], addr_reg: [9, 18], ..Default::default() });

    assert_no_output(device);
}

#[test]
#[rustfmt::skip]
fn test_multiple_test_ops() {
    let path = setup_test_file("-1");

    let ops = vec![
        FOPEN_REG, REG_ACC,
        FILER_VAL_ADDR, 0, 0, 0,
        FILER_VAL_ADDR, 0, 0, 0,
        FILER_VAL_ADDR, 0, 0, 0,
        FSKIP_VAL_VAL, 0, 1,
        FILER_VAL_ADDR, 0, 0, 0,
        FSEEK_VAL, 0,
        FILER_REG_AREG, REG_D0, REG_A0,
        FILER_REG_AREG, REG_D3, REG_A1,
        PUSH_VAL, 0,
        PUSH_VAL, 0,
        PUSH_VAL, 0,
        PUSH_VAL, 0,
        FSEEK_REG, REG_D0,
        FILEW_REG_VAL, REG_ACC, 4,
        FILEW_VAL_VAL, 0, 4,
        FILEW_VAL_REG, 0, REG_D0,
        FILEW_REG_REG, REG_D1, REG_D1,
        FSEEK_VAL, 0,
        FILER_REG_ADDR, REG_D0, 0, 20,
        FSEEK_VAL, 0,
        FILEW_VAL_ADDR, 0, 0, 20,
        FSEEK_VAL, 0,
    ];

    let mut device = Device::new(ops, vec![], vec![], vec![path]);

    assert_step_device("FOPEN ACC", &mut device, Dump { pc: 2, data_reg: [0, 0, 0, 6], ..Default::default() });
    assert_step_device("FILER 0 @0", &mut device, Dump { pc: 6, data_reg: [0, 0, 0, 6], ..Default::default() });
    assert_memory(&device, 0, &[0, 0, 0]);
    device.acc = 1;
    assert_step_device("FILER 0 @0", &mut device, Dump { pc: 10, acc: 1, data_reg: [0, 0, 0, 6], ..Default::default() });
    assert_memory(&device, 0, &[5, 0, 0]);
    assert_step_device("FILER 0 @0", &mut device, Dump { pc: 14, acc: 1, data_reg: [0, 0, 0, 6], ..Default::default() });
    assert_memory(&device, 0, &[6, 0, 0]);
    assert_step_device("FSKIP 0 1", &mut device, Dump { pc: 17, acc: 1, data_reg: [0, 0, 0, 6], ..Default::default() });
    assert_step_device("FILER 0 @0", &mut device, Dump { pc: 21, acc: 1, data_reg: [0, 0, 0, 6], ..Default::default() });
    assert_memory(&device, 0, &[8, 0, 0]);
    device.data_reg[3] = 0;
    assert_step_device("FSEEK 0", &mut device, Dump { pc: 23, acc: 1, ..Default::default() });
    assert_step_device("FILER D0 A0", &mut device, Dump { pc: 26, acc: 1, ..Default::default() });
    assert_memory(&device, 0, &[5, 0, 0]);
    assert_step_device("FILER D0 A1", &mut device, Dump { pc: 29, acc: 1, ..Default::default() });
    assert_memory(&device, 0, &[6, 0, 0]);
    assert_step_device("PUSH 0", &mut device, Dump { pc: 31, acc: 1, sp: 65534, ..Default::default() });
    assert_step_device("PUSH 0", &mut device, Dump { pc: 33, acc: 1, sp: 65533,..Default::default() });
    assert_step_device("PUSH 0", &mut device, Dump { pc: 35, acc: 1, sp: 65532,..Default::default() });
    assert_step_device("PUSH 0", &mut device, Dump { pc: 37, acc: 1, sp: 65531,..Default::default() });
    assert_step_device("FSEEK D0", &mut device, Dump { pc: 39, acc: 0, ..Default::default() });
    assert_step_device("FILEW ACC 4", &mut device, Dump {pc: 42, acc: 1, ..Default::default()});
    assert_step_device("FILEW 0 4", &mut device, Dump {pc: 45, acc: 1, ..Default::default()});
    assert_step_device("FILEW 0 D0", &mut device, Dump {pc: 48, acc: 1, ..Default::default()});
    assert_step_device("FILEW D1 D1", &mut device, Dump {pc: 51, acc: 1, ..Default::default()});
    assert_step_device("FSEEK 0", &mut device, Dump {pc: 53, acc: 1, ..Default::default()});
    device.acc = 5;
    assert_step_device("FILER D0 @20", &mut device, Dump { pc: 57, acc: 5, ..Default::default() });
    assert_memory(&device, 0, &[6]);
    assert_memory(&device, 20, &[4, 4, 0, 0, 9]);
    assert_step_device("FSEEK 0", &mut device, Dump {pc: 59, acc: 5, ..Default::default()});
    device.acc = 2;
    assert_step_device("FILEW 0 @20", &mut device, Dump { pc: 63, acc: 2, ..Default::default() });
    assert_step_device("FSEEK 0", &mut device, Dump {pc: 65, acc: 2, ..Default::default()});


    assert_no_output(device);
}

fn setup_test_file(suffix: &str) -> String {
    let mut path = tempdir().unwrap().into_path();
    path.push(format!("tape-device-test-file{}.test.bin", suffix));

    if path.exists() {
        remove_file(&path).unwrap();
    }
    let mut file = File::create(&path).unwrap();

    file.write_all(&[5, 6, 7, 8, 9, 10]).unwrap();

    path.to_str().unwrap().to_owned()
}
