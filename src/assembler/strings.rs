use crate::assembler::FORMAT_ERROR;
use anyhow::{Error, Result};
use std::collections::HashMap;

pub(super) fn compile_strings(lines: &mut Vec<String>) -> Result<(HashMap<String, u16>, Vec<u8>)> {
    let mut mapping = HashMap::with_capacity(lines.len());
    let mut output = Vec::with_capacity(lines.len() * 10);
    let mut line = lines.remove(0);
    while line != ".ops" && line != ".data" {
        if let Some(idx) = line.find('=') {
            let (key, content) = line.split_at(idx);
            let mut content = content
                .chars()
                .skip(1)
                .collect::<String>()
                .trim()
                .to_owned();
            if content.is_empty() {
                return Err(Error::msg(format!(
                    "String on line '{}' has no content, it must be defined as <key>=<content>, e.g. greeting=Hello world",
                    line
                )));
            }
            if content.starts_with('"') && content.ends_with('"') && content.len() > 2 {
                let mut chars = content.chars();
                chars.next();
                chars.next_back();
                content = chars.collect();
            }
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
            if content.len() > 255 {
                return Err(Error::msg(format!(
                    "Line '{}' in strings is too long, must be at most 255 chars",
                    line
                )));
            }
            if output.len() >= u16::MAX as usize {
                return Err(Error::msg(format!("Too many strings at '{}', max of {} chars in strings data including whitespace but not including keys", line, u16::MAX - 1)));
            }

            mapping.insert(key.to_string(), output.len() as u16);
            output.push(content.len() as u8);
            output.extend_from_slice(content.as_bytes());
        } else {
            return Err(Error::msg(format!(
                "Unexpected string definition: {}",
                line
            )));
        }
        if lines.is_empty() {
            return Err(Error::msg(format!(
                "Unexpected EoF while compiling strings, check ops section starts with .ops\n\n{}",
                FORMAT_ERROR
            )));
        }
        line = lines.remove(0);
    }

    if line == ".data" {
        //This is a horrible hack but without the code would have to be significantly changed
        //This line '.data' is needed by code outside this file so it must reinserted if it was
        //found and removed above
        lines.insert(0, String::from(".data"));
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
            String::from("simple=test"),
            String::from("checking=bytes"),
            String::from(".ops"),
        ];
        let result = compile_strings(&mut input);
        assert!(input.is_empty());
        assert!(result.is_ok());
        let result = result.unwrap();
        let keys = result.0.keys().collect::<Vec<&String>>();
        let values = result.0.values().collect::<Vec<&u16>>();
        assert!(keys.contains(&&String::from("simple")));
        assert!(keys.contains(&&String::from("checking")));
        assert!(values.contains(&&0));
        assert!(values.contains(&&5));
        assert_eq!(result.1, [4, 116, 101, 115, 116, 5, 98, 121, 116, 101, 115]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_whitespace() {
        let mut input = vec![
            String::from("nows1=  before"),
            String::from("nows2=  before  "),
            String::from("nows3=before  "),
            String::from("ws1=\"  before\""),
            String::from("ws2=\"before  \""),
            String::from("ws3=\"  before  \""),
            String::from(".ops"),
        ];
        let result = compile_strings(&mut input);
        assert!(input.is_empty());
        assert!(result.is_ok());
        let result = result.unwrap();
        let keys = result.0.keys().collect::<Vec<&String>>();
        let values = result.0.values().collect::<Vec<&u16>>();
        assert_eq!(keys.len(), values.len());
        assert!(keys.contains(&&String::from("nows1")));
        assert!(keys.contains(&&String::from("nows2")));
        assert!(keys.contains(&&String::from("nows3")));
        assert!(keys.contains(&&String::from("ws1")));
        assert!(keys.contains(&&String::from("ws2")));
        assert!(keys.contains(&&String::from("ws3")));
        assert_eq!(
            result.1,
            [
                6, 98, 101, 102, 111, 114, 101,
                6, 98, 101, 102, 111, 114, 101,
                6, 98, 101, 102, 111, 114, 101,
                8, 32, 32, 98, 101, 102, 111, 114, 101,
                8, 98, 101, 102, 111, 114, 101, 32, 32,
                10, 32, 32, 98, 101, 102, 111, 114, 101, 32, 32
            ]
        )
    }

    #[test]
    fn test_output_order_matches() {
        let mut input = vec![
            String::from("simple=test"),
            String::from("checking=order"),
            String::from("of=output"),
            String::from(".ops"),
        ];
        let result = compile_strings(&mut input);
        assert!(input.is_empty());
        assert!(result.is_ok());
        let result = result.unwrap();
        let keys = result.0.keys().collect::<Vec<&String>>();
        let values = result.0.values().collect::<Vec<&u16>>();
        assert_eq!(keys.len(), values.len());
        for (idx, key) in keys.iter().enumerate() {
            let key = key.as_str();
            match key {
                "simple" => assert_eq!(values[idx], &0),
                "checking" => assert_eq!(values[idx], &5),
                "of" => assert_eq!(values[idx], &11),
                _ => assert!(false, "Invalid key {}", key),
            }
        }
    }

    #[test]
    fn test_ops_not_consumed() {
        let mut input = vec![
            String::from("a=string"),
            String::from(".ops"),
            String::from("INC D0"),
        ];
        let result = compile_strings(&mut input);
        assert_eq!(input.len(), 1);
        assert_eq!(input[0], String::from("INC D0"));
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(
            result.0.keys().collect::<Vec<&String>>(),
            vec![&String::from("a")]
        );
        assert_eq!(result.0.values().collect::<Vec<&u16>>(), vec![&0]);
        assert_eq!(result.1, [6, 115, 116, 114, 105, 110, 103]);
    }

    #[test]
    fn test_data_not_consumed() {
        let mut input = vec![
            String::from("a=string"),
            String::from(".data"),
            String::from("key=[[0]]"),
        ];
        let result = compile_strings(&mut input);
        assert_eq!(input.len(), 2);
        assert_eq!(input[0], String::from(".data"));
        assert_eq!(input[1], String::from("key=[[0]]"));
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(
            result.0.keys().collect::<Vec<&String>>(),
            vec![&String::from("a")]
        );
        assert_eq!(result.0.values().collect::<Vec<&u16>>(), vec![&0]);
        assert_eq!(result.1, [6, 115, 116, 114, 105, 110, 103]);
    }

    #[test]
    fn test_just_ops_marker() {
        let mut input = vec![String::from(".ops")];
        let result = compile_strings(&mut input);
        assert!(input.is_empty());
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.0.is_empty());
        assert_eq!(result.1, vec![]);
    }

    #[test]
    fn test_no_ops_marker() {
        let mut input = vec![String::from("a=string")];
        assert!(compile_strings(&mut input).is_err());
        assert!(input.is_empty());
    }
}
