use anyhow::{Context, Error, Result};
use lazy_static::lazy_static;

use crate::assembler::parser::data_parser::DataParser;
use crate::assembler::program_model::{
    ConstantModel, DataModel, Definition, LabelModel, OpModel, ProgramModel, StringModel, Usage,
};
use crate::assembler::FORMAT_ERROR;
use crate::constants::code::{DIVDERS, KEYWORDS, MNEMONICS, REGISTERS};
use crate::constants::hardware::MAX_STRING_LEN;
use crate::language::parse_line;
use crate::language::parser::params::Param;
use std::collections::HashMap;

mod data_parser;

#[derive(Debug, Eq, PartialEq)]
pub enum ParseMode {
    Header,
    Strings,
    Data,
    Ops,
}

pub fn generate_program_model(input: Vec<String>) -> Result<ProgramModel> {
    if input.len() < 4 {
        return Err(Error::msg(FORMAT_ERROR));
    }
    let mut iter = input.into_iter();
    let name = ProgramModel::validate_name(
        iter.next()
            .context(format!("Program name missing\n\n{}", FORMAT_ERROR))?,
    )?;
    let version = ProgramModel::validate_version(
        iter.next()
            .context(format!("Program version missing\n\n{}", FORMAT_ERROR))?,
    )?;
    let mut program_model = ProgramModel::new(name, version);
    let mut parse_mode = ParseMode::Header;

    for (idx, line) in iter.enumerate() {
        let line_num = idx + 3;
        let trimmed = line.trim();
        if !trimmed.starts_with('#') && !trimmed.is_empty() {
            match trimmed {
                ".strings" => {
                    if parse_mode == ParseMode::Ops {
                        return Err(Error::msg(format!("Unexpected .strings divider at line {}, all data and strings must be defined before .ops", line_num)));
                    } else {
                        parse_mode = ParseMode::Strings;
                    }
                }
                ".data" => {
                    if parse_mode == ParseMode::Ops {
                        return Err(Error::msg(format!("Unexpected .data divider at line {}, all data and strings must be defined before .ops", line_num)));
                    } else {
                        parse_mode = ParseMode::Data;
                    }
                }
                ".ops" => {
                    if parse_mode == ParseMode::Ops {
                        return Err(Error::msg(format!(
                            "Unexpected .ops divider at line {}, already in ops section",
                            line_num
                        )));
                    } else {
                        parse_mode = ParseMode::Ops;
                    }
                }
                "" => {}
                _ => match parse_mode {
                    ParseMode::Header => {
                        return Err(Error::msg(format!(
                            "Unexpected content: {}\n\n{}",
                            line, FORMAT_ERROR
                        )))
                    }
                    ParseMode::Strings => {
                        parse_string(&mut program_model, &line, line_num).context(line)?
                    }
                    ParseMode::Data => {
                        parse_data(&mut program_model, &line, line_num).context(line)?
                    }
                    ParseMode::Ops => {
                        if trimmed.to_lowercase().starts_with("const") {
                            parse_constant(&mut program_model, &line, line_num).context(line)?
                        } else {
                            parse_op(&mut program_model, &line, line_num).context(line)?
                        }
                    }
                },
            }
        }
    }

    Ok(program_model)
}

fn parse_constant(program_model: &mut ProgramModel, line: &str, line_num: usize) -> Result<()> {
    let splits = line.split_whitespace().collect::<Vec<&str>>();
    if splits.len() < 2 {
        return Err(Error::msg(format!(
            "Error parsing constant on line {}, format must be const <key> <value>, e.g. const result d3",
            line_num
        )));
    }
    let key = splits[1];
    let value = splits[2];
    program_model.validate_key("constant key", key, line_num, false)?;
    let model = ConstantModel::new(key.to_owned(), value.to_owned(), line.to_owned(), line_num);
    program_model.constants.insert(key.to_owned(), model);
    Ok(())
}

fn parse_string(program_model: &mut ProgramModel, line: &str, line_num: usize) -> Result<()> {
    return if let Some((key, content)) = line.split_once('=') {
        program_model.validate_key("string key", key, line_num, false)?;
        let mut content = content.trim().to_owned();
        if content.is_empty() {
            return Err(Error::msg(format!(
                "String on line {} has no content, it must be defined as <key>=<content>, e.g. greeting=Hello world",
                line_num
            )));
        }
        if content.starts_with('"') && content.ends_with('"') && content.len() > 2 {
            let mut chars = content.chars();
            chars.next();
            chars.next_back();
            content = chars.collect();
        }
        if content.len() > MAX_STRING_LEN {
            return Err(Error::msg(format!(
                "String {} (parsed as {}) on line {} is too long, max {} chars",
                line, content, line_num, MAX_STRING_LEN
            )));
        }
        program_model.strings.insert(
            key.to_owned(),
            StringModel::new(key.to_owned(), content, line.to_owned(), line_num),
        );
        Ok(())
    } else {
        Err(Error::msg(format!(
            "String on line {} must be defined as <key>=<content>, e.g. greeting=Hello world",
            line_num
        )))
    };
}

fn parse_data(program_model: &mut ProgramModel, line: &str, line_num: usize) -> Result<()> {
    return if let Some((key, content)) = line.split_once('=') {
        program_model.validate_key("data key", key, line_num, false)?;
        let mut parser = DataParser::new();
        let error_msg = format!("Data definition on line {}: \"{}\"", line_num, line);
        parser.run(content).context(error_msg.clone())?;
        let bytes = parser.into_bytes().context(error_msg)?;
        program_model.data.insert(
            key.to_owned(),
            DataModel::new(key.to_owned(), bytes, line.to_owned(), line_num),
        );
        Ok(())
    } else {
        Err(Error::msg(format!(
            "Data on line {} must be defined as <key>=<content>, e.g. some_data=[[1,2,3],[50,60]]",
            line_num
        )))
    };
}

fn parse_op(program_model: &mut ProgramModel, orig_line: &str, line_num: usize) -> Result<()> {
    let mut line = orig_line.to_owned();
    if line.contains('#') {
        line = line.split_once('#').unwrap().0.to_owned();
    }
    if line.contains(':') {
        let (lbl, content) = line.split_once(':').unwrap();
        program_model.validate_key("label", lbl, line_num, true)?;
        let def = Some(Definition::new(orig_line.to_owned(), line_num));
        if program_model.labels.contains_key(lbl) {
            program_model.labels.get_mut(lbl).unwrap().definition = def;
        } else {
            program_model
                .labels
                .insert(lbl.to_owned(), LabelModel::new(lbl.to_owned(), def, vec![]));
        }
        line = content.to_owned();
    }

    let trimmed = line.trim();

    if trimmed.is_empty() {
        //line is label only
        return Ok(());
    }

    let processed = replace_constants(&mut program_model.constants, trimmed, line_num);

    let (opcode, params) = parse_line(&processed)?;

    for param in &params {
        match param {
            Param::Label(lbl) => {
                if !program_model.labels.contains_key(lbl) {
                    program_model.labels.insert(
                        lbl.to_owned(),
                        LabelModel::new(lbl.to_owned(), None, vec![]),
                    );
                }
                program_model
                    .labels
                    .get_mut(lbl)
                    .unwrap()
                    .usage
                    .push(Usage::new(orig_line.to_owned(), line_num));
            }
            Param::StrKey(key) => {
                if let Some(model) = program_model.strings.get_mut(key) {
                    model.usage.push(Usage::new(orig_line.to_owned(), line_num));
                } else {
                    return Err(Error::msg(format!(
                        "String key {} used on {}, line {} but was never defined",
                        key, orig_line, line_num
                    )));
                }
            }
            Param::DataKey(key) => {
                if let Some(model) = program_model.data.get_mut(key) {
                    model.usage.push(Usage::new(orig_line.to_owned(), line_num));
                } else {
                    return Err(Error::msg(format!(
                        "Data key {} used on {}, line {} but was never defined",
                        key, orig_line, line_num
                    )));
                }
            }
            _ => {}
        }
    }

    program_model.ops.push(OpModel::new(
        opcode,
        params,
        processed,
        orig_line.to_string(),
        line_num,
    ));

    Ok(())
}

fn replace_constants(
    constants: &mut HashMap<String, ConstantModel>,
    line: &str,
    line_num: usize,
) -> String {
    line.split_whitespace()
        .map(|word| {
            if let Some(model) = constants.get_mut(word) {
                model.usage.push(Usage::new(line.to_owned(), line_num));
                model.content.clone()
            } else {
                word.to_owned()
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

lazy_static! {
    static ref KEY_NAME_ERROR: String = format!("Key names must not include any register, keyword, section divider or mnemonic.\nThese include:\n{}\n{}\n{}\n{}",
        REGISTERS.join(" "),KEYWORDS.join(" "),DIVDERS.join(" "),MNEMONICS.join(" ")
        );
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::code::{ADD_REG_VAL, CMP_REG_REG};
    use crate::constants::hardware::{REG_ACC, REG_D1, REG_D3};
    use crate::language::parser::params::Param;

    #[rustfmt::skip]
    fn make_constant_model(key: &str, content: &str, def_line: &str, def_num: usize, usage_line: &str, usage_num: usize) -> ConstantModel {
        let mut model = ConstantModel::new(key.to_owned(), content.to_owned(), def_line.to_owned(), def_num);
        model.usage.push(Usage::new(usage_line.to_owned(), usage_num));
        model
    }

    #[test]
    fn test_whitespace() {
        let mut program_model = ProgramModel::new(String::new(), String::new());

        parse_constant(&mut program_model, "  const     key   3", 0).unwrap();
    }

    mod method_integration {
        use super::*;

        #[test]
        fn test_parse_valid_strings() {
            let mut program_model = ProgramModel::new(String::new(), String::new());

            #[rustfmt::skip]
                let data = vec![
                ("key", "key=content", 10, StringModel::new(String::from("key"), String::from("content"), String::from("key=content"), 10)),
                ("spaced", "spaced=this string has spaces", 15, StringModel::new(String::from("spaced"), String::from("this string has spaces"), String::from("spaced=this string has spaces"), 15)),
                ("padding", "padding=string has spaces     ", 1, StringModel::new(String::from("padding"), String::from("string has spaces"), String::from("padding=string has spaces     "), 1)),
                ("quotes", "quotes=\"  two spaced  \"", 2, StringModel::new(String::from("quotes"), String::from("  two spaced  "), String::from("quotes=\"  two spaced  \""), 2)),
                ("doublequotes", "doublequotes=\"\"this is a quote\"\"",32, StringModel::new(String::from("doublequotes"), String::from(r#""this is a quote""#), String::from("doublequotes=\"\"this is a quote\"\""), 32)),
            ];

            for entry in data {
                parse_string(&mut program_model, entry.1, entry.2).unwrap();
                let value = program_model.strings.get(entry.0).unwrap();
                assert_eq!(value, &entry.3, "{}", entry.0);
            }
        }

        #[test]
        fn test_parse_valid_data() {
            let mut program_model = ProgramModel::new(String::new(), String::new());
            #[rustfmt::skip]
                let data = vec![
                ("key", "key=[[10]]", 3, DataModel::new(String::from("key"), vec![1, 1, 10], String::from("key=[[10]]"), 3)),
                ("ex", "ex=[ [ x1, x2 ] , [ 3, 'a'] ]", 4, DataModel::new(String::from("ex"), vec![2, 2, 2, 1, 2, 3, 97], String::from("ex=[ [ x1, x2 ] , [ 3, 'a'] ]"), 4))
            ];

            for entry in data {
                parse_data(&mut program_model, entry.1, entry.2).unwrap();
                let value = program_model.data.get(entry.0).unwrap();
                assert_eq!(value, &entry.3, "{}", entry.0);
            }
        }

        #[test]
        fn test_parse_valid_ops() {
            let mut program_model = ProgramModel::new(String::new(), String::new());

            #[rustfmt::skip]
                let ops = vec![
                ("add reg val", "add d3 10", 30, OpModel::new(ADD_REG_VAL, vec![Param::DataReg(REG_D3), Param::Number(10)], String::from("add d3 10"), String::from("add d3 10"), 30)),
                ("cmp reg reg", "cmp d1 acc", 31, OpModel::new(CMP_REG_REG, vec![Param::DataReg(REG_D1), Param::DataReg(REG_ACC)], String::from("cmp d1 acc"), String::from("cmp d1 acc"), 31)),
            ];

            for (idx, entry) in ops.iter().enumerate() {
                parse_op(&mut program_model, entry.1, entry.2).unwrap();
                let value = &program_model.ops[idx];
                assert_eq!(value, &entry.3, "{}", entry.0);
            }
        }
    }

    mod replace_constants {
        use super::*;

        #[test]
        fn test_no_changes() {
            let processed = replace_constants(&mut HashMap::new(), "ADD A0 1", 0);

            assert_eq!(processed, "ADD A0 1");
        }

        #[test]
        fn test_first_param_change() {
            let mut constants = HashMap::new();
            constants.insert(
                String::from("test"),
                ConstantModel::new(String::from("test"), String::from("d0"), String::new(), 0),
            );
            let processed = replace_constants(&mut constants, "add test 1", 0);

            assert_eq!(processed, "add d0 1");
        }

        #[test]
        fn test_second_param_change() {
            let mut constants = HashMap::new();
            constants.insert(
                String::from("other"),
                ConstantModel::new(String::from("other"), String::from("d0"), String::new(), 0),
            );
            let processed = replace_constants(&mut constants, "sub d2 other", 0);

            assert_eq!(processed, "sub d2 d0");
        }

        #[test]
        fn test_multiple_param() {
            let mut constants = HashMap::new();
            constants.insert(
                String::from("first"),
                ConstantModel::new(String::new(), String::from("450"), String::new(), 0),
            );
            constants.insert(
                String::from("second"),
                ConstantModel::new(String::new(), String::from("xF1"), String::new(), 0),
            );
            let processed = replace_constants(&mut constants, "ld a0 words first second", 0);

            assert_eq!(processed, "ld a0 words 450 xF1");
        }

        #[test]
        fn test_duplicate_param() {
            let mut constants = HashMap::new();
            constants.insert(
                String::from("ex"),
                ConstantModel::new(String::new(), String::from("45"), String::new(), 0),
            );
            let processed = replace_constants(&mut constants, "cmp ex ex", 0);

            assert_eq!(processed, "cmp 45 45");
        }

        #[test]
        fn test_undeclared_param() {
            let mut constants = HashMap::new();
            let processed = replace_constants(&mut constants, "cpy d0 ex", 0);

            assert_eq!(processed, "cpy d0 ex");
        }
    }

    mod missing_parts {
        use super::*;

        #[test]
        fn test_no_name() {
            let input = vec!["1.0", ".ops", "add d0 1", "sub acc 1"]
                .into_iter()
                .map(|line| line.to_string())
                .collect();
            assert!(generate_program_model(input).is_err());
        }

        #[test]
        fn test_no_version() {
            let input = vec!["Test", ".ops", "add d0 1", "sub acc 1"]
                .into_iter()
                .map(|line| line.to_string())
                .collect();
            assert!(generate_program_model(input).is_err());
        }

        #[test]
        fn test_no_content() {
            assert!(generate_program_model(vec![]).is_err());
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_mixed_strings_data() {
            let input = vec![
                "test",
                "1.0",
                ".data",
                "dk1=[[10,20]]",
                "dk2=[[4,5]]",
                ".strings",
                "sk1=Test",
                ".data",
                "dk3=[['a']]",
                ".strings",
                "sk2=Another",
            ]
            .into_iter()
            .map(|line| line.to_string())
            .collect();

            let model = generate_program_model(input).unwrap();

            assert_eq!(model.strings.len(), 2);
            assert_eq!(model.data.len(), 3);
            assert!(model.strings.contains_key("sk1"));
            assert!(model.strings.contains_key("sk2"));
            assert!(model.data.contains_key("dk1"));
            assert!(model.data.contains_key("dk2"));
            assert!(model.data.contains_key("dk3"));
        }
    }

    mod class_integration {
        use super::*;
        use crate::constants::code::{CMP_REG_VAL, CPY_REG_VAL, LD_AREG_DATA_VAL_VAL, PRTS_STR};
        use crate::constants::hardware::{REG_A0, REG_D0};

        #[test]
        #[rustfmt::skip]
        fn test_direct_calls() {
            let mut program_model = ProgramModel::new(String::from("Test Program"), String::from("1"));
            parse_string(&mut program_model, "str_test1=First test string", 4).unwrap();
            parse_string(&mut program_model, "str_test2=\"  Second test string:  \"", 5).unwrap();
            parse_data(&mut program_model, "dat_numbers=[[4, 8, 15 , 16, 23,42],[ 1, 4 ,9, 16, 25, 36 ] ]", 7).unwrap();
            parse_data(&mut program_model, "dat_chars=[['f', 'o', 'o'] , ['b', 'a', 'r']]", 8).unwrap();
            parse_constant(&mut program_model, "const true 0", 10).unwrap();
            parse_constant(&mut program_model, "const false 1", 11).unwrap();
            parse_op(&mut program_model, "cpy d1 10", 12).unwrap();
            parse_op(&mut program_model, "cmp d0 false", 13).unwrap();
            parse_op(&mut program_model, "prts str_test1", 14).unwrap();
            parse_op(&mut program_model, "ld a0 dat_numbers 0 0", 15).unwrap();

            validate_integration_program_model(program_model);
        }

        #[test]
        #[rustfmt::skip]
        fn test_execution() {
            let input = vec![
                "Test Program",
                "1",
                ".strings",
                "str_test1=First test string",
                "str_test2=\"  Second test string:  \"",
                ".data",
                "dat_numbers=[[4, 8, 15 , 16, 23,42],[ 1, 4 ,9, 16, 25, 36 ] ]",
                "dat_chars=[['f', 'o', 'o'] , ['b', 'a', 'r']]",
                ".ops",
                "const true 0",
                "const false 1",
                "cpy d1 10",
                "cmp d0 false",
                "prts str_test1",
                "ld a0 dat_numbers 0 0"
            ].into_iter().map(|line| line.to_string()).collect();
            
            let program_model = generate_program_model(input).unwrap();
            
            validate_integration_program_model(program_model);
        }

        fn validate_integration_program_model(program_model: ProgramModel) {
            assert_eq!(program_model.name, String::from("Test Program"));
            assert_eq!(program_model.version, String::from("1"));
            assert_eq!(program_model.strings.len(), 2);
            assert_eq!(program_model.data.len(), 2);
            assert_eq!(program_model.constants.len(), 2);
            assert_eq!(program_model.ops.len(), 4);
            let mut model = StringModel::new(
                String::from("str_test1"),
                String::from("First test string"),
                String::from("str_test1=First test string"),
                4,
            );
            model
                .usage
                .push(Usage::new(String::from("prts str_test1"), 14));
            assert_eq!(program_model.strings.get("str_test1"), Some(&model));
            assert_eq!(
                program_model.strings.get("str_test2"),
                Some(&StringModel::new(
                    String::from("str_test2"),
                    String::from("  Second test string:  "),
                    String::from("str_test2=\"  Second test string:  \""),
                    5
                ))
            );
            let mut model = DataModel::new(
                String::from("dat_numbers"),
                vec![2, 6, 6, 4, 8, 15, 16, 23, 42, 1, 4, 9, 16, 25, 36],
                String::from("dat_numbers=[[4, 8, 15 , 16, 23,42],[ 1, 4 ,9, 16, 25, 36 ] ]"),
                7,
            );
            model
                .usage
                .push(Usage::new(String::from("ld a0 dat_numbers 0 0"), 15));
            assert_eq!(program_model.data.get("dat_numbers"), Some(&model));
            assert_eq!(
                program_model.data.get("dat_chars"),
                Some(&DataModel::new(
                    String::from("dat_chars"),
                    vec![2, 3, 3, 102, 111, 111, 98, 97, 114],
                    String::from("dat_chars=[['f', 'o', 'o'] , ['b', 'a', 'r']]"),
                    8
                ))
            );
            assert_eq!(
                program_model.constants.get("true"),
                Some(&ConstantModel::new(
                    String::from("true"),
                    String::from("0"),
                    String::from("const true 0"),
                    10
                ))
            );
            assert_eq!(
                program_model.constants.get("false"),
                Some(&make_constant_model(
                    "false",
                    "1",
                    "const false 1",
                    11,
                    "cmp d0 false",
                    13
                ))
            );
            assert_eq!(
                program_model.ops.get(0).unwrap(),
                &OpModel::new(
                    CPY_REG_VAL,
                    vec![Param::DataReg(REG_D1), Param::Number(10)],
                    String::from("cpy d1 10"),
                    String::from("cpy d1 10"),
                    12
                )
            );
            assert_eq!(
                program_model.ops.get(1).unwrap(),
                &OpModel::new(
                    CMP_REG_VAL,
                    vec![Param::DataReg(REG_D0), Param::Number(1)],
                    String::from("cmp d0 1"),
                    String::from("cmp d0 false"),
                    13
                )
            );
            assert_eq!(
                program_model.ops.get(2).unwrap(),
                &OpModel::new(
                    PRTS_STR,
                    vec![Param::StrKey(String::from("str_test1"))],
                    String::from("prts str_test1"),
                    String::from("prts str_test1"),
                    14
                )
            );
            assert_eq!(
                program_model.ops.get(3).unwrap(),
                &OpModel::new(
                    LD_AREG_DATA_VAL_VAL,
                    vec![
                        Param::AddrReg(REG_A0),
                        Param::DataKey(String::from("dat_numbers")),
                        Param::Number(0),
                        Param::Number(0)
                    ],
                    String::from("ld a0 dat_numbers 0 0"),
                    String::from("ld a0 dat_numbers 0 0"),
                    15
                )
            );
        }
    }

    mod ops {
        use super::*;
        use crate::constants::code::*;
        use crate::constants::hardware::*;
        use crate::language::parser::params::Param::DataReg as DReg;
        use crate::language::parser::params::Param::Label as Lbl;
        use crate::language::parser::params::Param::Number as Num;
        use crate::language::parser::params::Param::{Addr, AddrReg as AReg};

        #[rustfmt::skip]
        fn make_op_model(opcode: u8, params: Vec<Param>, line: &str, line_num: usize) -> OpModel {
            OpModel::new(opcode, params, line.to_owned(), line.to_owned(), line_num)
        }

        #[rustfmt::skip]
        fn make_op_model_constant(opcode: u8, params: Vec<Param>, orig_line: &str, after_constant: &str, line_num: usize) -> OpModel {
            OpModel::new(opcode, params, after_constant.to_owned(), orig_line.to_owned(), line_num)
        }

        #[rustfmt::skip]
        fn insert_constant(program_model: &mut ProgramModel, key: &str, value: &str, line_num: usize, usages: Vec<(&str, usize)>) {
            let mut constant = ConstantModel::new(key.to_owned(), value.to_owned(), format!("const {} {}", key, value), line_num);
            for (line, num) in usages {
                constant.usage.push(Usage::new(line.to_owned(), num));
            }
            program_model.constants.insert(key.to_owned(), constant);
        }

        #[test]
        #[rustfmt::skip]
        fn test_valid_add() {
            let mut program_model = ProgramModel::new(String::new(), String::new());
            insert_constant(&mut program_model, "reg", "acc", 0, vec![("ADD reg D2", 10), ("ADD d2 reg", 11)], );
            insert_constant(&mut program_model, "num", "4", 1, vec![("add d3 num", 12)]);
            parse_op(&mut program_model, "ADD D0 x34", 5).unwrap();
            parse_op(&mut program_model, "ADD D0 245", 6).unwrap();
            parse_op(&mut program_model, "ADD d0 'a'", 7).unwrap();
            parse_op(&mut program_model, "ADD D0 D1", 8).unwrap();
            parse_op(&mut program_model, "add D0 A0", 9).unwrap();
            parse_op(&mut program_model, "ADD reg D2", 10).unwrap();
            parse_op(&mut program_model, "ADD d2 reg", 11).unwrap();
            parse_op(&mut program_model, "add d3 num", 12).unwrap();

            program_model.validate().unwrap();

            assert_eq!(program_model.ops[0], make_op_model(ADD_REG_VAL, vec![DReg(REG_D0), Num(52)], "ADD D0 x34", 5));
            assert_eq!(program_model.ops[1], make_op_model(ADD_REG_VAL, vec![DReg(REG_D0), Num(245)], "ADD D0 245", 6));
            assert_eq!(program_model.ops[2], make_op_model(ADD_REG_VAL, vec![DReg(REG_D0), Num(97)], "ADD d0 'a'", 7));
            assert_eq!(program_model.ops[3], make_op_model(ADD_REG_REG, vec![DReg(REG_D0), DReg(REG_D1)], "ADD D0 D1", 8));
            assert_eq!(program_model.ops[4], make_op_model(ADD_REG_AREG, vec![DReg(REG_D0), AReg(REG_A0)], "add D0 A0", 9));
            assert_eq!(program_model.ops[5], make_op_model_constant(ADD_REG_REG, vec![DReg(REG_ACC), DReg(REG_D2)], "ADD reg D2", "ADD acc D2", 10));
            assert_eq!(program_model.ops[6], make_op_model_constant(ADD_REG_REG, vec![DReg(REG_D2), DReg(REG_ACC)], "ADD d2 reg", "ADD d2 acc", 11));
            assert_eq!(program_model.ops[7], make_op_model_constant(ADD_REG_VAL, vec![DReg(REG_D3), Num(4)], "add d3 num", "add d3 4", 12));
        }

        #[test]
        #[rustfmt::skip]
        fn test_valid_sub() {
            let mut program_model = ProgramModel::new(String::new(), String::new());
            insert_constant(&mut program_model, "reg", "acc", 0, vec![("SUB reg D2", 10), ("SUB d2 reg", 11)], );
            insert_constant(&mut program_model, "num", "4", 1, vec![("sub d3 num", 12)]);
            parse_op(&mut program_model, "SUB D0 x34", 5).unwrap();
            parse_op(&mut program_model, "SUB D0 245", 6).unwrap();
            parse_op(&mut program_model, "SUB d0 'a'", 7).unwrap();
            parse_op(&mut program_model, "SUB D0 D1", 8).unwrap();
            parse_op(&mut program_model, "sub D0 A0", 9).unwrap();
            parse_op(&mut program_model, "sUb reg D2", 10).unwrap();
            parse_op(&mut program_model, "SUB d2 reg", 11).unwrap();
            parse_op(&mut program_model, "sub d3 num", 12).unwrap();

            program_model.validate().unwrap();

            assert_eq!(program_model.ops[0], make_op_model(SUB_REG_VAL, vec![DReg(REG_D0), Num(52)], "SUB D0 x34", 5));
            assert_eq!(program_model.ops[1], make_op_model(SUB_REG_VAL, vec![DReg(REG_D0), Num(245)], "SUB D0 245", 6));
            assert_eq!(program_model.ops[2], make_op_model(SUB_REG_VAL, vec![DReg(REG_D0), Num(97)], "SUB d0 'a'", 7));
            assert_eq!(program_model.ops[3], make_op_model(SUB_REG_REG, vec![DReg(REG_D0), DReg(REG_D1)], "SUB D0 D1", 8));
            assert_eq!(program_model.ops[4], make_op_model(SUB_REG_AREG, vec![DReg(REG_D0), AReg(REG_A0)], "sub D0 A0", 9));
            assert_eq!(program_model.ops[5], make_op_model_constant(SUB_REG_REG, vec![DReg(REG_ACC), DReg(REG_D2)], "sUb reg D2", "sUb acc D2", 10));
            assert_eq!(program_model.ops[6], make_op_model_constant(SUB_REG_REG, vec![DReg(REG_D2), DReg(REG_ACC)], "SUB d2 reg", "SUB d2 acc", 11));
            assert_eq!(program_model.ops[7], make_op_model_constant(SUB_REG_VAL, vec![DReg(REG_D3), Num(4)], "sub d3 num", "sub d3 4", 12));
        }

        #[test]
        #[rustfmt::skip]
        fn test_valid_inc() {
            let mut program_model = ProgramModel::new(String::new(), String::new());
            insert_constant(&mut program_model, "reg", "acc", 0, vec![("inc reg", 7)], );
            parse_op(&mut program_model, "inc d0", 5).unwrap();
            parse_op(&mut program_model, "inc a1", 6).unwrap();
            parse_op(&mut program_model, "inc reg", 7).unwrap();

            program_model.validate().unwrap();

            assert_eq!(program_model.ops[0], make_op_model(INC_REG, vec![DReg(REG_D0)], "inc d0", 5));
            assert_eq!(program_model.ops[1], make_op_model(INC_REG, vec![AReg(REG_A1)], "inc a1", 6));
            assert_eq!(program_model.ops[2], make_op_model_constant(INC_REG, vec![DReg(REG_ACC)], "inc reg", "inc acc", 7));
        }

        #[test]
        #[rustfmt::skip]
        fn test_valid_dec() {
            let mut program_model = ProgramModel::new(String::new(), String::new());
            insert_constant(&mut program_model, "n", "a1", 0, vec![("dec reg", 7)], );
            parse_op(&mut program_model, "dec d0", 5).unwrap();
            parse_op(&mut program_model, "dec a1", 6).unwrap();
            parse_op(&mut program_model, "dec n", 7).unwrap();

            program_model.validate().unwrap();
            
            assert_eq!(program_model.ops[0], make_op_model(DEC_REG, vec![DReg(REG_D0)], "dec d0", 5));
            assert_eq!(program_model.ops[1], make_op_model(DEC_REG, vec![AReg(REG_A1)], "dec a1", 6));
            assert_eq!(program_model.ops[2], make_op_model_constant(DEC_REG, vec![AReg(REG_A1)], "dec n", "dec a1", 7));
        }

        #[test]
        #[rustfmt::skip]
        fn test_valid_cmp() {
            let mut program_model = ProgramModel::new(String::new(), String::new());
            parse_op(&mut program_model, "cmp d3 d2", 5).unwrap();
            parse_op(&mut program_model, "cmp acc 10", 6).unwrap();
            parse_op(&mut program_model, "cmp a0 a1", 7).unwrap();
            parse_op(&mut program_model, "cmp a0 @100", 8).unwrap();
            parse_op(&mut program_model, "cmp a0 d0 d1", 9).unwrap();
            parse_op(&mut program_model, "cmp d0 d1 a0", 10).unwrap();
            parse_op(&mut program_model, "cmp d0 a0", 11).unwrap();

            assert_eq!(program_model.ops[0], make_op_model(CMP_REG_REG, vec![DReg(REG_D3), DReg(REG_D2)], "cmp d3 d2", 5));
            assert_eq!(program_model.ops[1], make_op_model(CMP_REG_VAL, vec![DReg(REG_ACC), Num(10)], "cmp acc 10", 6));
            assert_eq!(program_model.ops[2], make_op_model(CMP_AREG_AREG, vec![AReg(REG_A0), AReg(REG_A1)], "cmp a0 a1", 7));
            assert_eq!(program_model.ops[3], make_op_model(CMP_AREG_ADDR, vec![AReg(REG_A0), Addr(100)], "cmp a0 @100", 8));
            assert_eq!(program_model.ops[4], make_op_model(CMP_AREG_REG_REG, vec![AReg(REG_A0), DReg(REG_D0), DReg(REG_D1)], "cmp a0 d0 d1", 9));
            assert_eq!(program_model.ops[5], make_op_model(CMP_REG_REG_AREG, vec![DReg(REG_D0), DReg(REG_D1), AReg(REG_A0)], "cmp d0 d1 a0", 10));
            assert_eq!(program_model.ops[6], make_op_model(CMP_REG_AREG, vec![DReg(REG_D0), AReg(REG_A0)], "cmp d0 a0", 11));
        }

        #[test]
        #[rustfmt::skip]
        fn test_valid_cpy() {
            let mut program_model = ProgramModel::new(String::new(), String::new());
            parse_op(&mut program_model, "cpy d3 d2", 5).unwrap();
            parse_op(&mut program_model, "cpy acc 10", 6).unwrap();
            parse_op(&mut program_model, "cpy a0 a1", 7).unwrap();
            parse_op(&mut program_model, "cpy a0 @100", 8).unwrap();
            parse_op(&mut program_model, "cpy a0 d0 d1", 9).unwrap();
            parse_op(&mut program_model, "cpy d0 d1 a0", 10).unwrap();
            parse_op(&mut program_model, "cpy d0 a0", 11).unwrap();

            assert_eq!(program_model.ops[0], make_op_model(CPY_REG_REG, vec![DReg(REG_D3), DReg(REG_D2)], "cpy d3 d2", 5));
            assert_eq!(program_model.ops[1], make_op_model(CPY_REG_VAL, vec![DReg(REG_ACC), Num(10)], "cpy acc 10", 6));
            assert_eq!(program_model.ops[2], make_op_model(CPY_AREG_AREG, vec![AReg(REG_A0), AReg(REG_A1)], "cpy a0 a1", 7));
            assert_eq!(program_model.ops[3], make_op_model(CPY_AREG_ADDR, vec![AReg(REG_A0), Addr(100)], "cpy a0 @100", 8));
            assert_eq!(program_model.ops[4], make_op_model(CPY_AREG_REG_REG, vec![AReg(REG_A0), DReg(REG_D0), DReg(REG_D1)], "cpy a0 d0 d1", 9));
            assert_eq!(program_model.ops[5], make_op_model(CPY_REG_REG_AREG, vec![DReg(REG_D0), DReg(REG_D1), AReg(REG_A0)], "cpy d0 d1 a0", 10));
            assert_eq!(program_model.ops[6], make_op_model(CPY_REG_AREG, vec![DReg(REG_D0), AReg(REG_A0)], "cpy d0 a0", 11));
        }

        #[test]
        #[rustfmt::skip]
        fn test_valid_jmps() {
            let ops = [
                ("jmp", JMP_ADDR, JMP_AREG), 
                ("je", JE_ADDR, JE_AREG), 
                ("jl", JL_ADDR, JL_AREG), 
                ("jg", JG_ADDR, JG_AREG), 
                ("jne", JNE_ADDR, JNE_AREG),
                ("over", OVER_ADDR, OVER_AREG),
                ("nover", NOVER_ADDR, NOVER_AREG),
                ("ipoll", IPOLL_ADDR, IPOLL_AREG),
                ("call", CALL_ADDR, CALL_AREG)    
            ];
            for (op, op_addr, op_areg) in ops {
                let mut program_model = ProgramModel::new(String::new(), String::new());
                insert_constant(&mut program_model, "addr", "@x100", 0, vec![(&format!("{} addr", op), 5)], );
                insert_constant(&mut program_model, "areg", "a1", 1, vec![(&format!("{} areg", op), 6)], );
                parse_op(&mut program_model, &format!("lbl: {} addr", op), 5).unwrap();
                parse_op(&mut program_model, &format!("{} areg", op), 6).unwrap();
                parse_op(&mut program_model, &format!("{} a0", op), 7).unwrap();
                parse_op(&mut program_model, &format!("{} @200", op), 8).unwrap();
                parse_op(&mut program_model, &format!("{} lbl", op), 9).unwrap();

                program_model.validate().unwrap();

                assert_eq!(program_model.ops[0], make_op_model_constant(op_addr, vec![Addr(256)], &format!("lbl: {} addr", op), &format!("{} @x100", op), 5), "{}", op);
                assert_eq!(program_model.ops[1], make_op_model_constant(op_areg, vec![AReg(REG_A1)], &format!("{} areg", op), &format!("{} a1", op), 6), "{}", op);
                assert_eq!(program_model.ops[2], make_op_model(op_areg, vec![AReg(REG_A0)], &format!("{} a0", op),7), "{}", op);
                assert_eq!(program_model.ops[3], make_op_model(op_addr, vec![Addr(200)], &format!("{} @200", op), 8), "{}", op);
                assert_eq!(program_model.ops[4], make_op_model(op_addr, vec![Lbl(String::from("lbl"))], &format!("{} lbl", op), 9), "{}", op);   
            }
        }

        #[test]
        #[rustfmt::skip]
        fn test_valid_no_params() {
            for (op, opcode) in [("ret", RET), ("prtln", PRTLN), ("debug", DEBUG), ("halt", HALT), ("nop", NOP), ("time", TIME)] {
                let mut program_model = ProgramModel::new(String::new(), String::new());
                parse_op(&mut program_model, op, 0).unwrap();

                program_model.validate().unwrap();   
                
                assert_eq!(program_model.ops[0], make_op_model(opcode, vec![], op, 0), "{}", op);
            }
        }

        #[test]
        #[rustfmt::skip]
        fn test_missing_constant() {
            let mut program_model = ProgramModel::new(String::new(), String::new());
            assert!(parse_op(&mut program_model, "inc not_set", 0).is_err());
        }

        #[test]
        #[rustfmt::skip]
        fn test_missing_string() {
            let mut program_model = ProgramModel::new(String::new(), String::new());
            assert!(parse_op(&mut program_model, "prts not_set", 0).is_err());
        }

        #[test]
        #[rustfmt::skip]
        fn test_missing_data() {
            let mut program_model = ProgramModel::new(String::new(), String::new());
            assert!(parse_op(&mut program_model, "ld a0 not_set 0 0", 0).is_err());
        }
    }
}
