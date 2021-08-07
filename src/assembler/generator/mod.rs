use crate::assembler::debug_model::{
    DebugData, DebugLabel, DebugModel, DebugOp, DebugString, DebugUsage,
};
use crate::assembler::program_model::{
    AddressReplacement, DataModel, LabelModel, OpModel, ProgramModel, StringModel,
};
use crate::constants::hardware::{MAX_DATA_BYTES, MAX_STRING_BYTES};
use crate::constants::system::{PRG_VERSION, TAPE_HEADER_1, TAPE_HEADER_2};
use crate::constants::{get_addr_byte_offset, get_byte_count};
use anyhow::{Error, Result};
use std::collections::{BTreeMap, HashMap};

pub fn generate_byte_code(program_model: ProgramModel) -> Result<(Vec<u8>, DebugModel)> {
    //Write header
    //0xFD A0 01 <name len> <name> <ver len> <ver>
    let mut output = vec![TAPE_HEADER_1, TAPE_HEADER_2, PRG_VERSION];
    let mut debug_model = DebugModel::default();
    output.push(program_model.name.len() as u8);
    output.extend_from_slice(program_model.name.as_bytes());
    output.push(program_model.version.len() as u8);
    output.extend_from_slice(program_model.version.as_bytes());

    let op_byte_start = output.len() + 2; //+2 for op byte count written once len is known

    //Generate bytes and addresses for strings and data
    let (string_bytes, string_addresses) =
        generate_string_bytes(program_model.strings, &mut debug_model)?;

    let (data_bytes, data_addresses) = generate_data_bytes(program_model.data, &mut debug_model)?;

    //Generate and write op bytes
    let ops_output = generate_ops_bytes(
        &program_model.ops,
        op_byte_start,
        program_model.labels,
        &mut debug_model,
        string_addresses,
        data_addresses,
    )?;

    output.extend_from_slice(&(ops_output.bytes.len() as u16).to_be_bytes());
    output.extend_from_slice(&ops_output.bytes);

    //Now all label positions are known, update addresses
    output = update_addresses(
        output,
        ops_output.label_targets,
        ops_output.label_addresses,
        op_byte_start,
        &mut debug_model,
    );

    //Write string len, string bytes and data bytes
    output.extend_from_slice(&(string_bytes.len() as u16).to_be_bytes());
    output.extend_from_slice(&string_bytes);
    output.extend_from_slice(&data_bytes);

    Ok((output, debug_model))
}

/// Replace placeholder address bytes with actual values
/// * `bytes`: The list of bytes to update
/// * `targets`: The indexes of bytes in `bytes` to update, mapped by a string key
/// * `sources`: The actual values to write at the indexes in `targets`, mapped by a string key
/// * `op_byte_start`: Index of the first op byte
fn update_addresses(
    mut bytes: Vec<u8>,
    targets: HashMap<String, Vec<u16>>,
    sources: HashMap<String, u16>,
    op_byte_start: usize,
    debug: &mut DebugModel,
) -> Vec<u8> {
    for (key, source) in sources {
        if let Some(op_offsets) = targets.get(&key) {
            for offset in op_offsets {
                let addr = source.to_be_bytes();
                bytes[*offset as usize] = addr[0];
                bytes[(*offset + 1) as usize] = addr[1];
                let op_offset = *offset - (op_byte_start as u16);
                let debug_op = debug
                    .ops
                    .iter_mut()
                    .find(|op| {
                        op.byte_addr < op_offset
                            && op_offset < op.byte_addr + get_byte_count(op.bytes[0]) as u16
                    })
                    .unwrap_or_else(|| {
                        panic!(
                            "No DebugOp found but label target exists for '{}', targets: {:?}",
                            key, op_offsets
                        )
                    });
                let local_offset = op_offset - debug_op.byte_addr;
                debug_op.bytes[local_offset as usize] = addr[0];
                debug_op.bytes[(local_offset + 1) as usize] = addr[1];
            }
        }
    }
    bytes
}

#[derive(Debug, Default)]
struct OpsOutput {
    bytes: Vec<u8>,
    label_targets: HashMap<String, Vec<u16>>,
    label_addresses: HashMap<String, u16>,
}

fn generate_ops_bytes(
    ops: &[OpModel],
    offset: usize,
    labels: HashMap<String, LabelModel>,
    debug: &mut DebugModel,
    string_addresses: HashMap<String, u16>,
    data_addresses: HashMap<String, u16>,
) -> Result<OpsOutput> {
    let mut labels: BTreeMap<usize, LabelModel> = convert_label_map_to_linenum(labels);
    let mut output = OpsOutput::default();
    for op in ops {
        if !labels.is_empty() {
            let lbl = labels.values().next().unwrap();
            let lbl_line_num = lbl.definition.as_ref().unwrap().line_num;
            if lbl_line_num <= op.line_num {
                let original_line = lbl.definition.as_ref().unwrap().original_line.clone();
                debug.labels.push(DebugLabel::new(
                    output.bytes.len() as u16,
                    lbl.key.clone(),
                    original_line,
                    lbl_line_num,
                ));
                output
                    .label_addresses
                    .insert(lbl.key.clone(), output.bytes.len() as u16);
                labels.remove(&lbl_line_num);
            }
        }
        let (mut bytes, replacement) = op.to_bytes();
        if replacement != AddressReplacement::None {
            let param_offset = get_addr_byte_offset(op.opcode).unwrap_or_else(|| {
                panic!(
                    "AddressReplacement found for op with no addr byte offset for line {}",
                    op.line_num
                )
            });
            match replacement {
                AddressReplacement::None => panic!("Assembler error: None after a not none check"),
                AddressReplacement::Label(key) => {
                    output
                        .label_targets
                        .entry(key)
                        .or_insert_with(Vec::new)
                        .push((output.bytes.len() + param_offset + offset) as u16);
                }
                AddressReplacement::Str(key) => {
                    debug
                        .strings
                        .iter_mut()
                        .find(|str| str.key == key)
                        .unwrap_or_else(|| {
                            panic!(
                                "Unknown string '{}' found in generation on line {} (e1)",
                                key, op.line_num
                            )
                        })
                        .usage
                        .push(DebugUsage::new(
                            output.bytes.len() as u16,
                            param_offset as u8,
                            op.line_num,
                        ));
                    let addr = string_addresses
                        .get(&key)
                        .unwrap_or_else(|| {
                            panic!(
                                "Unknown string '{}' found in generation on line {} (e2)",
                                key, op.line_num
                            )
                        })
                        .to_be_bytes();
                    bytes[param_offset] = addr[0];
                    bytes[param_offset + 1] = addr[1];
                }
                AddressReplacement::Data(key) => {
                    debug
                        .data
                        .iter_mut()
                        .find(|datum| datum.key == key)
                        .unwrap_or_else(|| {
                            panic!(
                                "Unknown data '{}' found in generation on line {} (e1)",
                                key, op.line_num
                            )
                        })
                        .usage
                        .push(DebugUsage::new(
                            output.bytes.len() as u16,
                            param_offset as u8,
                            op.line_num,
                        ));
                    let addr = data_addresses
                        .get(&key)
                        .unwrap_or_else(|| {
                            panic!(
                                "Unknown data '{}' found in generation on line {} (e2)",
                                key, op.line_num
                            )
                        })
                        .to_be_bytes();
                    bytes[param_offset] = addr[0];
                    bytes[param_offset + 1] = addr[1];
                }
            };
        }
        debug.ops.push(DebugOp::new(
            output.bytes.len() as u16,
            op.original_line.clone(),
            op.line_num,
            op.after_processing.clone(),
            bytes.clone(),
        ));
        output.bytes.extend_from_slice(&bytes);
    }

    Ok(output)
}

fn generate_data_bytes(
    data: HashMap<String, DataModel>,
    debug: &mut DebugModel,
) -> Result<(Vec<u8>, HashMap<String, u16>)> {
    let mut output = vec![];
    let mut addresses = HashMap::new();
    let mut list: Vec<(String, DataModel)> = data.into_iter().collect();
    list.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
    for (key, data_model) in list {
        if (output.len() + data_model.content.len()) > MAX_DATA_BYTES {
            return Err(Error::msg(format!(
                "Too much data at `{}` on line {}, max {} bytes but is at least {} bytes",
                data_model.definition.original_line,
                data_model.definition.line_num,
                MAX_DATA_BYTES,
                output.len() + data_model.content.len()
            )));
        }
        addresses.insert(key.clone(), output.len() as u16);
        debug.data.push(DebugData::new(
            output.len() as u16,
            key,
            data_model.interpretation,
            data_model.definition.original_line.clone(),
            data_model.definition.line_num,
        ));
        output.extend_from_slice(&data_model.content);
    }

    Ok((output, addresses))
}

fn generate_string_bytes(
    strings: HashMap<String, StringModel>,
    debug: &mut DebugModel,
) -> Result<(Vec<u8>, HashMap<String, u16>)> {
    let mut output = vec![];
    let mut addresses = HashMap::new();
    let mut list: Vec<(String, StringModel)> = strings.into_iter().collect();
    list.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
    for (key, string_model) in list {
        if (output.len() + string_model.content.len()) > MAX_STRING_BYTES {
            return Err(Error::msg(format!(
                "Too many strings at `{}` on line {}, max {} bytes but is at least {} bytes",
                string_model.definition.original_line,
                string_model.definition.line_num,
                MAX_STRING_BYTES,
                output.len() + string_model.content.len()
            )));
        }
        addresses.insert(key.clone(), output.len() as u16);
        debug.strings.push(DebugString::new(
            output.len() as u16,
            key,
            string_model.content.clone(),
            string_model.definition.original_line.clone(),
            string_model.definition.line_num,
        ));
        output.push(string_model.content.len() as u8);
        output.extend_from_slice(string_model.content.as_bytes());
    }

    Ok((output, addresses))
}

fn convert_label_map_to_linenum(
    labels: HashMap<String, LabelModel>,
) -> BTreeMap<usize, LabelModel> {
    labels
        .into_iter()
        .map(|(_, model)| (model.definition.as_ref().unwrap().line_num, model))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::code::{
        ADD_REG_REG, CPY_REG_REG, INC_REG, LD_AREG_DATA_VAL_REG, PRTS_STR,
    };
    use crate::constants::hardware::*;
    use crate::language::parser::params::Param;

    #[test]
    #[rustfmt::skip]
    fn test_update_addresses_single() {
        let bytes = vec![PRTS_STR, 0, 0];
        let mut targets = HashMap::new();
        let mut sources = HashMap::new();

        targets.insert(String::from("foo"), vec![1]);
        sources.insert(String::from("abc"), 0);
        sources.insert(String::from("foo"), 4);

        let ops = vec![
            DebugOp::new(0, String::from("PRTS foo"), 0, String::from("PRTS foo"), vec![PRTS_STR, 0, 0])
        ];

        let output = update_addresses(bytes, targets, sources, 0,&mut DebugModel::new(ops, vec![], vec![], vec![]));
        assert_eq!(output, vec![PRTS_STR, 0, 4]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_gen_string_bytes() {
        let mut strings = HashMap::new();
        strings.insert(
            String::from("a"),
            StringModel::new(String::new(), String::from("test string"), String::new(), 0),
        );
        strings.insert(
            String::from("b"),
            StringModel::new(String::new(), String::from("an example"), String::new(), 0),
        );
        strings.insert(
            String::from("c"),
            StringModel::new(String::new(), String::from("abcdef"), String::new(), 0),
        );

        let (bytes, sources) = generate_string_bytes(strings, &mut DebugModel::default()).unwrap();
        let mut expected = HashMap::new();
        expected.insert(String::from("a"), 0_u16);
        expected.insert(String::from("b"), 12);
        expected.insert(String::from("c"), 23);

        assert_eq!(bytes, vec![
            11, 116, 101, 115, 116, 32, 115, 116, 114, 105, 110, 103,
            10, 97, 110, 32, 101, 120, 97, 109, 112, 108, 101,
            6, 97, 98, 99, 100, 101, 102
        ]);

        assert_eq!(expected.len(), sources.len());
        assert_eq!(expected.get("a"), sources.get("a"));
        assert_eq!(expected.get("b"), sources.get("b"));
        assert_eq!(expected.get("c"), sources.get("c"));
    }

    #[test]
    #[rustfmt::skip]
    fn test_gen_data_bytes() {
        let mut data = HashMap::new();
        data.insert(
            String::from("a"),
            DataModel::new(String::new(), vec![2, 5, 2, 1, 2, 3, 4, 5, 6, 7], vec![vec![1, 2, 3, 4, 5], vec![6, 7]], String::new(), 0),
        );
        data.insert(
            String::from("b"),
            DataModel::new(String::new(), vec![4, 2, 2, 2, 2, 97, 98, 99, 100, 101, 102, 103, 104], vec![vec![97, 98], vec![99, 100], vec![101, 102], vec![103, 104]], String::new(), 0),
        );

        let (bytes, sources) = generate_data_bytes(data, &mut DebugModel::default()).unwrap();
        let mut expected = HashMap::new();
        expected.insert(String::from("a"), 0_u16);
        expected.insert(String::from("b"), 10);

        assert_eq!(bytes, vec![
            2, 5, 2, 1, 2, 3, 4, 5, 6, 7,
            4, 2, 2, 2, 2, 97, 98, 99, 100, 101, 102, 103, 104
        ]);

        assert_eq!(expected.len(), sources.len());
        assert_eq!(expected.get("a"), sources.get("a"));
        assert_eq!(expected.get("b"), sources.get("b"));
    }

    mod generate_ops {
        use super::*;
        use crate::constants::code::LD_AREG_DATA_REG_VAL;
        use crate::language::parser::params::Param::*;

        #[test]
        #[rustfmt::skip]
        fn test_target_gen() {
            let mut string_addresses = HashMap::new();
            string_addresses.insert(String::from("foo"), 0_u16);
            let mut data_addresses = HashMap::new();
            data_addresses.insert(String::from("bar"), 0_u16);
            let mut debug = DebugModel::new(
                vec![],
                vec![DebugString::new(0, String::from("foo"), String::new(), String::new(), 0)],
                vec![DebugData::new(0, String::from("bar"), vec![], String::new(), 0)],
                vec![]
            );

            let output = generate_ops_bytes(
                &[
                    OpModel::new(PRTS_STR, vec![StrKey(String::from("foo"))], String::new(), String::from("prts foo"), 0),
                    OpModel::new(LD_AREG_DATA_REG_VAL, vec![AddrReg(REG_A0), DataKey(String::from("bar")), DataReg(REG_D2), Number(10), ], String::new(), String::from("ld a0 bar d2 10"), 0),
                ],
                0,
                HashMap::new(),
                &mut debug,
                string_addresses,
                data_addresses,
            )
                .unwrap();

            assert_eq!(
                output.bytes,
                vec![
                    PRTS_STR, 0, 0,
                    LD_AREG_DATA_REG_VAL, REG_A0, 0, 0, REG_D2, 10
                ]
            );
        }
    }

    #[test]
    #[rustfmt::skip]
    fn test_header() {
        let model = ProgramModel::new(String::from("Test Prog"), String::from("1.0"));
        let (bytes, _) = generate_byte_code(model).unwrap();

        assert_eq!(
            bytes,
            vec![
                TAPE_HEADER_1, TAPE_HEADER_2, PRG_VERSION,
                9, 84, 101, 115, 116, 32, 80, 114, 111, 103,
                3, 49, 46, 48,
                0, 0,
                0, 0
            ]
        )
    }

    #[test]
    #[rustfmt::skip]
    fn test_simple_prog() {
        let mut model = ProgramModel::new(String::from("a"), String::from("b"));

        model.ops.push(OpModel::new(INC_REG, vec![Param::DataReg(REG_D0)], String::new(), String::from("inc d0"), 0));
        model.ops.push(OpModel::new(CPY_REG_REG, vec![Param::DataReg(REG_D1), Param::DataReg(REG_D0)], String::new(), String::from("cpy d1 d0"), 1));
        model.ops.push(OpModel::new(ADD_REG_REG, vec![Param::DataReg(REG_D0), Param::DataReg(REG_D1)], String::new(), String::from("add d0 d1"), 2));

        let (bytes, _) = generate_byte_code(model).unwrap();

        assert_eq!(
            bytes,
            vec![
                TAPE_HEADER_1, TAPE_HEADER_2, PRG_VERSION,
                1, 97,
                1, 98,
                0, 8,
                INC_REG, REG_D0,
                CPY_REG_REG, REG_D1, REG_D0,
                ADD_REG_REG, REG_D0, REG_D1,
                0, 0
            ]
        )
    }

    #[test]
    #[rustfmt::skip]
    fn test_simple_prog_with_strings() {
        let mut model = ProgramModel::new(String::from("a"), String::from("b"));

        model.strings.insert(String::from("abc"), StringModel::new(String::from("abc"), String::from("foo"), String::new(), 0));
        model.strings.insert(String::from("test"), StringModel::new(String::from("test"), String::from("bar"), String::new(), 0));

        model.ops.push(OpModel::new(INC_REG, vec![Param::DataReg(REG_D0)], String::new(), String::from("inc d0"), 0));
        model.ops.push(OpModel::new(PRTS_STR, vec![Param::StrKey(String::from("test"))], String::new(), String::from("prts test"), 1));

        let (bytes, _) = generate_byte_code(model).unwrap();

        assert_eq!(
            bytes,
            vec![
                TAPE_HEADER_1, TAPE_HEADER_2, PRG_VERSION,
                1, 97,
                1, 98,
                0, 5,
                INC_REG, REG_D0,
                PRTS_STR, 0, 4,
                0, 8,
                3, 102, 111, 111,
                3, 98, 97, 114
            ]
        )
    }

    #[test]
    #[rustfmt::skip]
    fn test_simple_prog_with_data() {
        let mut model = ProgramModel::new(String::from("a"), String::from("b"));

        model.data.insert(String::from("dk1"), DataModel::new(String::new(), vec![3, 2, 2, 4, 10, 11, 50, 51, 97, 98, 99, 100], vec![vec![10, 11], vec![50, 51], vec![97, 98, 99, 100]], String::new(), 0));
        model.data.insert(String::from("dk2"), DataModel::new(String::new(), vec![1, 10, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39], vec![vec![30, 31, 32, 33, 34, 35, 36, 37, 38, 39]], String::new(), 0));

        model.ops.push(OpModel::new(ADD_REG_REG, vec![Param::DataReg(REG_D0), Param::DataReg(REG_D1)], String::new(), String::from("add d0 d1"), 0));
        model.ops.push(OpModel::new(INC_REG, vec![Param::DataReg(REG_ACC)], String::new(), String::from("inc acc"), 0));
        model.ops.push(OpModel::new(LD_AREG_DATA_VAL_REG, vec![Param::AddrReg(REG_A0), Param::DataKey(String::from("dk2")), Param::Number(2), Param::DataReg(REG_D3)], String::new(), String::from("ld a0 dk1 2 d3"), 1));

        let (bytes, _) = generate_byte_code(model).unwrap();

        assert_eq!(
            bytes,
            vec![
                TAPE_HEADER_1, TAPE_HEADER_2, PRG_VERSION,
                1, 97,
                1, 98,
                0, 11,
                ADD_REG_REG, REG_D0, REG_D1,
                INC_REG, REG_ACC,
                LD_AREG_DATA_VAL_REG, REG_A0, 0, 12, 2, REG_D3,
                0, 0,
                3, 2, 2, 4, 10, 11, 50, 51, 97, 98, 99, 100,
                1, 10, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39
            ]
        )
    }

    #[test]
    #[rustfmt::skip]
    fn test_simple_prog_with_strings_and_data() {
        let mut model = ProgramModel::new(String::from("a"), String::from("b"));

        model.strings.insert(String::from("abc"), StringModel::new(String::from("abc"), String::from("foo"), String::new(), 0));
        model.data.insert(String::from("dk1"), DataModel::new(String::new(), vec![3, 2, 2, 4, 10, 11, 50, 51, 97, 98, 99, 100], vec![vec![10, 11], vec![50, 51], vec![97, 98, 99, 100]], String::new(), 0));

        model.ops.push(OpModel::new(ADD_REG_REG, vec![Param::DataReg(REG_D0), Param::DataReg(REG_D1)], String::new(), String::from("add d0 d1"), 0));
        model.ops.push(OpModel::new(INC_REG, vec![Param::DataReg(REG_ACC)], String::new(), String::from("inc acc"), 0));
        model.ops.push(OpModel::new(LD_AREG_DATA_VAL_REG, vec![Param::AddrReg(REG_A0), Param::DataKey(String::from("dk1")), Param::Number(2), Param::DataReg(REG_D3)], String::new(), String::from("ld a0 dk1 2 d3"), 1));
        model.ops.push(OpModel::new(PRTS_STR, vec![Param::StrKey(String::from("abc"))], String::new(), String::from("prts abc"), 3));

        let (bytes, model) = generate_byte_code(model).unwrap();

        assert_eq!(
            bytes,
            vec![
                TAPE_HEADER_1, TAPE_HEADER_2, PRG_VERSION,
                1, 97,
                1, 98,
                0, 14,
                ADD_REG_REG, REG_D0, REG_D1,
                INC_REG, REG_ACC,
                LD_AREG_DATA_VAL_REG, REG_A0, 0, 0, 2, REG_D3,
                PRTS_STR, 0, 0,
                0, 4,
                3, 102, 111, 111,
                3, 2, 2, 4, 10, 11, 50, 51, 97, 98, 99, 100
            ]
        );

        let mut debug_str = DebugString::new(0, String::from("abc"), String::from("foo"), String::new(), 0);
        let mut debug_data = DebugData::new(0, String::from("dk1"), vec![vec![10, 11], vec![50, 51], vec![97, 98, 99, 100]], String::new(), 0);
        debug_str.usage.push(DebugUsage::new(11, 1, 3));
        debug_data.usage.push(DebugUsage::new(5, 2, 1));

        assert_eq!(
            model,
            DebugModel::new(
                vec![
                    DebugOp::new(0, String::from("add d0 d1"), 0, String::new(), vec![ADD_REG_REG, REG_D0, REG_D1]),
                    DebugOp::new(3, String::from("inc acc"), 0, String::new(), vec![INC_REG, REG_ACC]),
                    DebugOp::new(5, String::from("ld a0 dk1 2 d3"), 1, String::new(), vec![LD_AREG_DATA_VAL_REG, REG_A0, 0, 0, 2, REG_D3]),
                    DebugOp::new(11, String::from("prts abc"), 3, String::new(), vec![PRTS_STR, 0, 0]),
                ],
                vec![debug_str],
                vec![debug_data],
                vec![])
        );
    }
}
