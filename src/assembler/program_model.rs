use crate::assembler::{FORMAT_ERROR, KEY_NAME_ERROR};
use crate::constants::code::{DIVDERS, KEYWORDS, MNEMONICS, REGISTERS};
use crate::language::parser::params::Param;
use anyhow::{Error, Result};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct ProgramModel {
    pub name: String,
    pub version: String,
    pub strings: HashMap<String, StringModel>,
    pub data: HashMap<String, DataModel>,
    pub constants: HashMap<String, ConstantModel>,
    pub ops: Vec<OpModel>,
    pub labels: HashMap<String, LabelModel>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct LabelModel {
    pub key: String,
    pub definition: Option<Definition>,
    pub usage: Vec<Usage>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct ConstantModel {
    pub key: String,
    pub content: String,
    pub definition: Definition,
    pub usage: Vec<Usage>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Usage {
    pub original_line: String,
    pub line_num: usize,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Definition {
    pub original_line: String,
    pub line_num: usize,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct StringModel {
    pub key: String,
    pub content: String,
    pub definition: Definition,
    pub usage: Vec<Usage>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct DataModel {
    pub key: String,
    pub content: Vec<u8>,
    pub definition: Definition,
    pub usage: Vec<Usage>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct OpModel {
    pub opcode: u8,
    pub params: Vec<Param>,
    pub after_processing: String,
    pub original_line: String,
    pub line_num: usize,
}

impl ProgramModel {
    pub fn new(name: String, version: String) -> Self {
        ProgramModel {
            name,
            version,
            strings: HashMap::new(),
            data: HashMap::new(),
            constants: HashMap::new(),
            ops: vec![],
            labels: HashMap::new(),
        }
    }

    pub fn validate_name(name: String) -> Result<String> {
        let trimmed = name.trim();
        if trimmed.is_empty() || trimmed.chars().count() > 20 {
            return Err(Error::msg("Invalid program name, must be between 1 and 20 ASCII characters, numbers and symbols"));
        }
        Ok(trimmed.to_string())
    }

    pub fn validate_version(name: String) -> Result<String> {
        let trimmed = name.trim();
        if trimmed.is_empty() || trimmed.chars().count() > 10 {
            return Err(Error::msg("Invalid program version, must be between 1 and 10 ASCII characters, numbers and symbols"));
        }
        Ok(trimmed.to_string())
    }
}

impl ProgramModel {
    pub fn validate_key(
        &self,
        key_type: &str,
        key: &str,
        line_num: usize,
        is_label: bool,
    ) -> Result<()> {
        let lowercased = key.to_lowercase();
        let lowercased = lowercased.as_str();
        if REGISTERS.contains(&lowercased)
            || KEYWORDS.contains(&lowercased)
            || MNEMONICS.contains(&lowercased)
            || DIVDERS.contains(&lowercased)
        {
            return Err(Error::msg(format!(
                "Invalid {} '{}' on line {}\n\n{}",
                key_type,
                key,
                line_num,
                KEY_NAME_ERROR.to_string()
            )));
        }

        let has_invalid_chars = lowercased
            .chars()
            .any(|chr| !chr.is_ascii_alphanumeric() && chr != '_');
        let starts_with_letter = lowercased
            .chars()
            .next()
            .map_or(false, |chr| chr.is_ascii_alphabetic());
        if has_invalid_chars || !starts_with_letter {
            return Err(Error::msg(format!(
                "Invalid {} '{}' on line {}\n{}s can only include ASCII letters, numbers and '_' and must start with a letter",
                key_type, key, line_num, key_type
            )));
        }
        if let Some(string_model) = self.strings.get(key) {
            return Err(Error::msg(format!(
                "Invalid {} '{}' on line {}\nAlready defined as string on line {}",
                key_type, key, line_num, string_model.definition.line_num
            )));
        }
        if let Some(data_model) = self.data.get(key) {
            return Err(Error::msg(format!(
                "Invalid {} '{}' on line {}\nAlready defined as data on line {}",
                key_type, key, line_num, data_model.definition.line_num
            )));
        }
        if let Some(constant_model) = self.constants.get(key) {
            return Err(Error::msg(format!(
                "Invalid {} '{}' on line {}\nAlready defined as constant on line {}",
                key_type, key, line_num, constant_model.definition.line_num
            )));
        }
        if let Some(label_model) = self.labels.get(key) {
            if let Some(def) = &label_model.definition {
                return Err(Error::msg(format!(
                    "Invalid {} '{}' on line {}\nAlready defined as label on line {}",
                    key_type, key, line_num, def.line_num
                )));
            } else if !is_label {
                let usage = label_model
                    .usage
                    .iter()
                    .map(|usage| usage.line_num.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                return Err(Error::msg(format!(
                    "Invalid {} '{}' on line {}\nAlready defined as label via usage on lines {}",
                    key_type, key, line_num, usage
                )));
            };
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if self.ops.is_empty() {
            return Err(Error::msg(format!("No ops found\n\n{}", FORMAT_ERROR)));
        }

        let mut error = String::new();
        let mut warning = String::new();

        for label in &self.labels {
            if label.1.definition.is_none() {
                error.push_str(&format!("Label {} is never defined\n", label.0));
            }
            if label.1.usage.is_empty() {
                warning.push_str(&format!("Label {} is never used\n", label.0));
            }
        }

        for string in &self.strings {
            if string.1.usage.is_empty() {
                warning.push_str(&format!("String {} is never used\n", string.0));
            }
        }

        for data in &self.data {
            if data.1.usage.is_empty() {
                warning.push_str(&format!("Data {} is never used\n", data.0));
            }
        }

        println!("{}", warning);
        if error.is_empty() {
            Ok(())
        } else {
            Err(Error::msg(error))
        }
    }
}

impl LabelModel {
    pub fn new(key: String, definition: Option<Definition>, usage: Vec<Usage>) -> Self {
        LabelModel {
            key,
            definition,
            usage,
        }
    }
}

impl ConstantModel {
    pub fn new(key: String, content: String, original_line: String, line_num: usize) -> Self {
        ConstantModel {
            key,
            content,
            definition: Definition::new(original_line, line_num),
            usage: vec![],
        }
    }
}

impl Usage {
    pub fn new(original_line: String, line_num: usize) -> Self {
        Usage {
            original_line,
            line_num,
        }
    }
}

impl Definition {
    pub fn new(original_line: String, line_num: usize) -> Self {
        Definition {
            original_line,
            line_num,
        }
    }
}

impl StringModel {
    pub fn new(key: String, content: String, original_line: String, line_num: usize) -> Self {
        StringModel {
            key,
            content,
            definition: Definition::new(original_line, line_num),
            usage: vec![],
        }
    }
}

impl DataModel {
    pub fn new(key: String, content: Vec<u8>, original_line: String, line_num: usize) -> Self {
        DataModel {
            key,
            content,
            definition: Definition::new(original_line, line_num),
            usage: vec![],
        }
    }
}

impl OpModel {
    pub fn new(
        opcode: u8,
        params: Vec<Param>,
        after_constants: String,
        original_line: String,
        line_num: usize,
    ) -> Self {
        OpModel {
            opcode,
            params,
            after_processing: after_constants,
            original_line,
            line_num,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum AddressReplacement {
    None,
    Label(String),
    Str(String),
    Data(String),
}

impl OpModel {
    pub fn to_bytes(&self) -> (Vec<u8>, AddressReplacement) {
        let mut output = vec![self.opcode];
        let mut replacement = AddressReplacement::None;
        for param in &self.params {
            match param {
                Param::DataReg(val) | Param::AddrReg(val) | Param::Number(val) => output.push(*val),
                Param::Addr(addr) => output.extend_from_slice(&addr.to_be_bytes()),
                Param::Label(lbl) => {
                    output.push(0);
                    output.push(0);
                    replacement = AddressReplacement::Label(lbl.to_owned());
                }
                Param::StrKey(key) => {
                    output.push(0);
                    output.push(0);
                    replacement = AddressReplacement::Str(key.to_owned());
                }
                Param::DataKey(key) => {
                    output.push(0);
                    output.push(0);
                    replacement = AddressReplacement::Data(key.to_owned());
                }
            }
        }
        (output, replacement)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::code::{JMP_ADDR, LD_AREG_DATA_VAL_VAL, PRTS_STR};
    use crate::constants::hardware::REG_A1;

    #[test]
    fn test_valid_keys() {
        let valid_keys = vec![
            "abc",
            "fgrea",
            "gerawer",
            "rgte",
            "rfegr_gfg",
            "fgsg",
            "sa_sdfaf",
            "z",
            "DSDGFHRGERHTSJRYTHGRERHTRSRGER",
        ];
        let program_model = ProgramModel::new(String::from("TEST"), String::new());

        for key in valid_keys {
            program_model.validate_key("test", key, 0, false).unwrap();
            program_model.validate_key("test", key, 0, true).unwrap();
        }
    }

    #[test]
    fn test_system_invalid_keys() {
        let invalid_keys = vec![
            "d0", "d1", "d2", "d3", "acc", "a0", "a1", ".data", ".strings", ".ops", "const", "add",
            "sub", "inc", "dec", "jmp", "je", "jl", "jg", "jne", "cpy", "cmp", "over", "nover",
            "ld", "memr", "memw", "memp", "halt", "nop", "fopen", "filer", "filew", "fchk",
            "fseek", "fskip", "call", "ret", "swp", "prt", "prtc", "prtln", "prts", "prtd", "push",
            "pop", "arg", "ipoll", "rchr", "rstr", "and", "or", "xor", "not", "rand", "seed",
            "time", "debug",
        ];

        let program_model = ProgramModel::new(String::from("TEST"), String::new());

        for key in invalid_keys {
            let result = program_model.validate_key("test key", key, 0, false);
            assert!(result.is_err(), "not invalid: {}", key);
            let error = result.unwrap_err().to_string();
            assert!(
                &error.contains("Key names must not include any register"),
                "error text mismatch {}: {}",
                key,
                error
            );
            let result = program_model.validate_key("test key", key, 0, true);
            assert!(result.is_err(), "not invalid: {}", key);
            let error = result.unwrap_err().to_string();
            assert!(
                &error.contains("Key names must not include any register"),
                "error text mismatch {}: {}",
                key,
                error
            );
        }
    }

    #[test]
    fn test_invalid_keys() {
        let invalid_keys = vec!["1", " ", "[]]", "ddf fsdfs", "rfegr-gfg", "ds,."];
        let program_model = ProgramModel::new(String::from("TEST"), String::new());

        for key in invalid_keys {
            assert!(
                program_model.validate_key("test", key, 0, false).is_err(),
                "{}",
                key
            );
            assert!(
                program_model.validate_key("test", key, 0, true).is_err(),
                "{}",
                key
            );
        }
    }

    #[test]
    fn test_duplicate_keys_constant() {
        let mut program_model = ProgramModel::new(String::from("TEST"), String::new());
        assert!(program_model
            .validate_key("test key", "TEST", 0, false)
            .is_ok());
        assert!(program_model
            .validate_key("test key", "TEST", 0, true)
            .is_ok());
        program_model.constants.insert(
            String::from("TEST"),
            ConstantModel::new(
                String::from("TEST"),
                String::from("TESTC"),
                String::new(),
                0,
            ),
        );
        let result = program_model.validate_key("test key", "TEST", 0, false);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(
            &error.contains("Already defined as constant"),
            "error text mismatch {}",
            error
        );
        let result = program_model.validate_key("test key", "TEST", 0, true);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(
            &error.contains("Already defined as constant"),
            "error text mismatch {}",
            error
        );
    }

    #[test]
    fn test_duplicate_keys_string() {
        let mut program_model = ProgramModel::new(String::from("TEST"), String::new());
        assert!(program_model
            .validate_key("test key", "TEST", 0, false)
            .is_ok());
        assert!(program_model
            .validate_key("test key", "TEST", 0, true)
            .is_ok());
        program_model.strings.insert(
            String::from("TEST"),
            StringModel::new(
                String::from("TEST"),
                String::from("TESTC"),
                String::new(),
                0,
            ),
        );
        let result = program_model.validate_key("test key", "TEST", 0, false);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(
            &error.contains("Already defined as string"),
            "error text mismatch {}",
            error
        );
        let result = program_model.validate_key("test key", "TEST", 0, true);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(
            &error.contains("Already defined as string"),
            "error text mismatch {}",
            error
        );
    }

    #[test]
    fn test_duplicate_keys_data() {
        let mut program_model = ProgramModel::new(String::from("TEST"), String::new());
        assert!(program_model
            .validate_key("test key", "TEST", 0, false)
            .is_ok());
        assert!(program_model
            .validate_key("test key", "TEST", 0, true)
            .is_ok());
        program_model.data.insert(
            String::from("TEST"),
            DataModel::new(String::from("TEST"), vec![], String::new(), 0),
        );
        let result = program_model.validate_key("test key", "TEST", 0, false);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(
            &error.contains("Already defined as data"),
            "error text mismatch {}",
            error
        );
        let result = program_model.validate_key("test key", "TEST", 0, true);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(
            &error.contains("Already defined as data"),
            "error text mismatch {}",
            error
        );
    }

    #[test]
    fn test_duplicate_keys_label_no_def() {
        let mut program_model = ProgramModel::new(String::from("TEST"), String::new());
        assert!(program_model
            .validate_key("test key", "TEST", 0, false)
            .is_ok());
        assert!(program_model
            .validate_key("test key", "TEST", 0, true)
            .is_ok());
        program_model.labels.insert(
            String::from("TEST"),
            LabelModel::new(
                String::from("TEST"),
                None,
                vec![Usage::new(String::new(), 50), Usage::new(String::new(), 51)],
            ),
        );
        let result = program_model.validate_key("test key", "TEST", 0, false);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(
            &error.contains("Already defined as label via usage on lines 50, 51"),
            "error text mismatch {}",
            error
        );
        assert!(program_model
            .validate_key("test key", "TEST", 0, true)
            .is_ok());
    }

    #[test]
    fn test_duplicate_keys_label_with_def() {
        let mut program_model = ProgramModel::new(String::from("TEST"), String::new());
        assert!(program_model
            .validate_key("test key", "TEST", 0, false)
            .is_ok());
        assert!(program_model
            .validate_key("test key", "TEST", 0, true)
            .is_ok());
        program_model.labels.insert(
            String::from("TEST"),
            LabelModel::new(
                String::from("TEST"),
                Some(Definition::new(String::new(), 10)),
                vec![],
            ),
        );
        let result = program_model.validate_key("test key", "TEST", 0, false);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(
            &error.contains("Already defined as label on line 10"),
            "error text mismatch {}",
            error
        );
        let result = program_model.validate_key("test key", "TEST", 0, true);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(
            &error.contains("Already defined as label on line 10"),
            "error text mismatch {}",
            error
        );
    }

    #[test]
    fn test_duplicate_keys_label_with_both() {
        let mut program_model = ProgramModel::new(String::from("TEST"), String::new());
        assert!(program_model
            .validate_key("test key", "TEST", 0, false)
            .is_ok());
        assert!(program_model
            .validate_key("test key", "TEST", 0, true)
            .is_ok());
        program_model.labels.insert(
            String::from("TEST"),
            LabelModel::new(
                String::from("TEST"),
                Some(Definition::new(String::new(), 11)),
                vec![Usage::new(String::new(), 20)],
            ),
        );
        let result = program_model.validate_key("test key", "TEST", 0, false);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(
            &error.contains("Already defined as label on line 11"),
            "error text mismatch {}",
            error
        );
        let result = program_model.validate_key("test key", "TEST", 0, true);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(
            &error.contains("Already defined as label on line 11"),
            "error text mismatch {}",
            error
        );
    }

    #[test]
    fn test_valid_program_names() {
        let valid_names = vec![
            String::from("a"),
            String::from("abcdefghijklmnopqrst"),
            String::from("program"),
            String::from("1"),
            String::from("_ asd rg 3g3£@"),
            String::from("       dsgsfgsfgdf               "),
        ];

        for name in valid_names {
            let result = ProgramModel::validate_name(name.clone());
            assert_eq!(result.unwrap().as_str(), name.trim(), "{}", name)
        }
    }

    #[test]
    fn test_invalid_program_names() {
        let invalid_names = vec![
            String::from(""),
            String::from("                                     "),
            String::from("abcdefghijklmnopqrstu"),
        ];

        for name in invalid_names {
            assert!(
                ProgramModel::validate_name(name.clone()).is_err(),
                "{}",
                name
            );
        }
    }

    #[test]
    fn test_valid_program_versions() {
        let valid_names = vec![
            String::from("@"),
            String::from("abcdefghij"),
            String::from("1.0"),
            String::from("1"),
            String::from("      2.0    "),
        ];

        for name in valid_names {
            let result = ProgramModel::validate_version(name.clone());
            assert_eq!(result.unwrap().as_str(), name.trim(), "{}", name)
        }
    }

    #[test]
    fn test_invalid_program_versions() {
        let invalid_names = vec![
            String::from(""),
            String::from("                                     "),
            String::from("abcdefghijklmnopqrstu"),
        ];

        for name in invalid_names {
            assert!(
                ProgramModel::validate_version(name.clone()).is_err(),
                "{}",
                name
            );
        }
    }

    #[test]
    fn check_program_model_json_format() {
        let mut model = ProgramModel::new(String::from("prog name"), String::from("ver1"));
        let mut s_model = StringModel::new(
            String::from("s_key"),
            String::from("example string"),
            String::from("s_key=example string"),
            3,
        );
        s_model
            .usage
            .push(Usage::new(String::from("prts s_key"), 10));
        model.strings.insert(String::from("s_key"), s_model);
        let mut c_model = ConstantModel::new(
            String::from("foo"),
            String::from("a1"),
            String::from("const foo a1"),
            8,
        );
        c_model
            .usage
            .push(Usage::new(String::from("ld foo d_key 0 0"), 11));
        model.constants.insert(String::from("foo"), c_model);
        let mut d_model = DataModel::new(
            String::from("d_key"),
            vec![1, 1, 1],
            String::from("d_key=[[1]]"),
            6,
        );
        d_model
            .usage
            .push(Usage::new(String::from("ld foo d_key 0 0"), 11));
        model.data.insert(String::from("d_key"), d_model);
        let l_model = LabelModel::new(
            String::from("lbl"),
            Some(Definition::new(String::from("lbl:"), 7)),
            vec![Usage::new(String::from("jmp lbl"), 12)],
        );
        model.labels.insert(String::from("lbl"), l_model);
        model.ops.push(OpModel::new(
            PRTS_STR,
            vec![Param::StrKey(String::from("s_key"))],
            String::from("prts s_key"),
            String::from("prts s_key"),
            10,
        ));
        model.ops.push(OpModel::new(
            LD_AREG_DATA_VAL_VAL,
            vec![
                Param::AddrReg(REG_A1),
                Param::DataKey(String::from("d_key")),
                Param::Number(0),
                Param::Number(0),
            ],
            String::from("ld a1 d_key 0 0"),
            String::from("ld foo d_key 0 0"),
            11,
        ));
        model.ops.push(OpModel::new(
            JMP_ADDR,
            vec![Param::Label(String::from("lbl"))],
            String::from("jmp lbl"),
            String::from("jmp lbl"),
            12,
        ));

        assert_eq!(serde_json::to_string(&model).unwrap(), String::from("{\"name\":\"prog name\",\"version\":\"ver1\",\"strings\":{\"s_key\":{\"key\":\"s_key\",\"content\":\"example string\",\"definition\":{\"original_line\":\"s_key=example string\",\"line_num\":3},\"usage\":[{\"original_line\":\"prts s_key\",\"line_num\":10}]}},\"data\":{\"d_key\":{\"key\":\"d_key\",\"content\":[1,1,1],\"definition\":{\"original_line\":\"d_key=[[1]]\",\"line_num\":6},\"usage\":[{\"original_line\":\"ld foo d_key 0 0\",\"line_num\":11}]}},\"constants\":{\"foo\":{\"key\":\"foo\",\"content\":\"a1\",\"definition\":{\"original_line\":\"const foo a1\",\"line_num\":8},\"usage\":[{\"original_line\":\"ld foo d_key 0 0\",\"line_num\":11}]}},\"ops\":[{\"opcode\":147,\"params\":[{\"StrKey\":\"s_key\"}],\"after_processing\":\"prts s_key\",\"original_line\":\"prts s_key\",\"line_num\":10},{\"opcode\":71,\"params\":[{\"AddrReg\":33},{\"DataKey\":\"d_key\"},{\"Number\":0},{\"Number\":0}],\"after_processing\":\"ld a1 d_key 0 0\",\"original_line\":\"ld foo d_key 0 0\",\"line_num\":11},{\"opcode\":32,\"params\":[{\"Label\":\"lbl\"}],\"after_processing\":\"jmp lbl\",\"original_line\":\"jmp lbl\",\"line_num\":12}],\"labels\":{\"lbl\":{\"key\":\"lbl\",\"definition\":{\"original_line\":\"lbl:\",\"line_num\":7},\"usage\":[{\"original_line\":\"jmp lbl\",\"line_num\":12}]}}}"));
    }
}
