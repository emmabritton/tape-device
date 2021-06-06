mod inner;
mod outer;

use crate::assembler::data::outer::parse_outer_array;
use crate::assembler::FORMAT_ERROR;
use anyhow::{Error, Result};
use std::collections::HashMap;

pub(super) fn compile_data(lines: &mut Vec<String>) -> Result<(HashMap<String, u16>, Vec<u8>)> {
    let mut mapping = HashMap::with_capacity(lines.len());
    let mut output = Vec::with_capacity(lines.len() * 10);
    let mut line = lines.remove(0);
    while line != ".ops" {
        println!(">Checking {}", line);
        if let Some(idx) = line.find('=') {
            let (key, content) = line.split_at(idx);
            let content: String = content.chars().skip(1).collect();
            let key = key.trim();
            if key
                .chars()
                .any(|chr| !(chr.is_ascii_alphanumeric() || chr == '_'))
            {
                return Err(Error::msg(format!(
                    "Line '{}' has invalid key must be [a-zA-Z0-9_]+",
                    line
                )));
            }
            if output.len() >= u16::MAX as usize {
                return Err(Error::msg(format!(
                    "Too much data at '{}', max of {} bytes in data, not including keys",
                    line,
                    u16::MAX - 1
                )));
            }

            mapping.insert(key.to_string(), output.len() as u16);
            output.extend_from_slice(&parse_outer_array(&content)?);
        } else {
            return Err(Error::msg(format!("Unexpected data definition: {}", line)));
        }
        if lines.is_empty() {
            return Err(Error::msg(format!(
                "Unexpected EoF while compiling data, check ops section starts with .ops\n\n{}",
                FORMAT_ERROR
            )));
        }
        line = lines.remove(0);
        println!(">Loading {}", line);
    }

    output.shrink_to_fit();
    mapping.shrink_to_fit();
    Ok((mapping, output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_file() {
        let mut input = vec![
            String::from("simple=[[10,11]]"),
            String::from("checking=[[1],[2]]"),
            String::from(".ops"),
        ];
        let result = compile_data(&mut input);
        assert!(input.is_empty());
        assert!(result.is_ok());
        let result = result.unwrap();
        //Keys and values are sorted as otherwise they are in random order
        //So just checking the expected values are somewhere
        let keys = result.0.keys().collect::<Vec<&String>>();
        let values = result.0.values().collect::<Vec<&u16>>();
        assert!(keys.contains(&&String::from("simple")));
        assert!(keys.contains(&&String::from("checking")));
        assert!(values.contains(&&0));
        assert!(values.contains(&&4));
        assert_eq!(result.1, [1, 2, 10, 11, 2, 1, 1, 1, 2]);
    }

    #[test]
    fn test_number_array() {
        let mut input = vec![
            String::from("key=[[10,11], [ 4, x10]]"),
            String::from("key2=[[30,30 , 3, xAA]]"),
            String::from(".ops"),
        ];
        let result = compile_data(&mut input);
        let result = result.unwrap();
        assert_eq!(input, Vec::new() as Vec<String>);
    }

    #[test]
    fn test_string_array() {
        let mut input = vec![
            String::from(r#"key=["ab", " cd " ]"#),
            String::from(r#"key2=["abc"]"#),
            String::from(".ops"),
        ];
        let result = compile_data(&mut input);
        let result = result.unwrap();
        assert_eq!(input, Vec::new() as Vec<String>);
    }

    #[test]
    fn test_char_array() {
        let mut input = vec![String::from("key=[['a','b'],['c']]"), String::from(".ops")];
        let result = compile_data(&mut input);
        let result = result.unwrap();
        assert_eq!(input, Vec::new() as Vec<String>);
    }

    #[test]
    fn test_mixed_array() {
        let mut input = vec![
            String::from(r#"key=[['a','b'], [1,2], [x10, x11], [1, 'c', x10]]"#),
            String::from(".ops"),
        ];
        let result = compile_data(&mut input);
        let result = result.unwrap();
        assert_eq!(input, Vec::new() as Vec<String>);
    }

    #[test]
    fn test_output_order_matches() {
        let mut input = vec![
            String::from("simple=[\"test\"]"),
            String::from("checking=[\"order\"]"),
            String::from("of=[\"output\"]"),
            String::from(".ops"),
        ];
        let result = compile_data(&mut input);
        let result = result.unwrap();
        assert_eq!(input, Vec::new() as Vec<String>);
        let keys = result.0.keys().collect::<Vec<&String>>();
        let values = result.0.values().collect::<Vec<&u16>>();
        assert_eq!(keys.len(), values.len());
        for (idx, key) in keys.iter().enumerate() {
            let key = key.as_str();
            match key {
                "simple" => assert_eq!(values[idx], &0),
                "checking" => assert_eq!(values[idx], &6),
                "of" => assert_eq!(values[idx], &13),
                _ => assert!(false, "Invalid key {}", key),
            }
        }
    }

    #[test]
    fn test_ops_not_consumed() {
        let mut input = vec![
            String::from("key=[[3,2]]"),
            String::from(".ops"),
            String::from("INC D0"),
        ];
        let result = compile_data(&mut input);
        assert_eq!(input[0], String::from("INC D0"));
        assert_eq!(input.len(), 1);
        let result = result.unwrap();
        assert_eq!(
            result.0.keys().collect::<Vec<&String>>(),
            vec![&String::from("key")]
        );
        assert_eq!(result.0.values().collect::<Vec<&u16>>(), vec![&0]);
        assert_eq!(result.1, [1, 2, 3, 2]);
    }

    #[test]
    fn test_just_ops_marker() {
        let mut input = vec![String::from(".ops")];
        let result = compile_data(&mut input);
        assert_eq!(input, Vec::new() as Vec<String>);
        let result = result.unwrap();
        assert!(result.0.is_empty());
        assert_eq!(result.1, vec![]);
    }

    #[test]
    fn test_no_ops_marker() {
        let mut input = vec![String::from("a=[[0]]")];
        assert!(compile_data(&mut input).is_err());
        assert_eq!(input, Vec::new() as Vec<String>);
    }
}
