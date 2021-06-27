use crate::assembler::FORMAT_ERROR;
use crate::constants::hardware::{MAX_DATA_ARRAY_COUNT, MAX_DATA_ARRAY_LEN};
use anyhow::{Context, Error, Result};
use std::collections::HashMap;

pub(super) fn compile_data(lines: &mut Vec<String>) -> Result<(HashMap<String, u16>, Vec<u8>)> {
    let mut mapping = HashMap::with_capacity(lines.len());
    let mut output = Vec::with_capacity(lines.len() * 10);
    let mut line = lines.remove(0);
    while line != ".ops" {
        if let Some(idx) = line.find('=') {
            let (key, content) = line.split_at(idx);
            let content: String = content.chars().skip(1).collect();
            let mut parser = DataParser::new();
            parser.run(&content)?;
            mapping.insert(key.to_owned(), output.len() as u16);
            output.extend_from_slice(&parser.into_bytes()?);
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
    }

    output.shrink_to_fit();
    mapping.shrink_to_fit();
    Ok((mapping, output))
}

#[derive(Debug)]
pub struct DataParser {
    output: Vec<Vec<u8>>,
    current_array: Vec<u8>,
    current_content: String,
    container_mode: ContainerMode,
    value_mode: ValueMode,
    escaping: bool,
}

impl DataParser {
    pub fn new() -> Self {
        DataParser {
            output: vec![],
            current_array: vec![],
            current_content: String::new(),
            container_mode: ContainerMode::None,
            value_mode: ValueMode::None,
            escaping: false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum ContainerMode {
    None,
    String,
    Array,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum ValueMode {
    None,
    Number,
    Hex,
    Char,
}

impl DataParser {
    pub fn into_bytes(self) -> Result<Vec<u8>> {
        if self.output.len() > MAX_DATA_ARRAY_COUNT {
            return Err(Error::msg(format!(
                "Too many arrays/string, max {}",
                MAX_DATA_ARRAY_COUNT
            )));
        }
        let mut bytes = vec![self.output.len() as u8];
        for (idx, array) in self.output.iter().enumerate() {
            if array.len() > MAX_DATA_ARRAY_LEN {
                return Err(Error::msg(format!(
                    "Array/string {} too long, max bytes {}",
                    idx, MAX_DATA_ARRAY_LEN
                )));
            }
            bytes.push(array.len() as u8);
        }
        for array in self.output {
            bytes.extend_from_slice(&array)
        }
        Ok(bytes)
    }

    pub fn run(&mut self, content: &str) -> Result<()> {
        let array_text = validate_content(content).context(content.to_owned())?;
        self.parse(array_text)
    }

    fn parse(&mut self, content: String) -> Result<()> {
        for (chr_idx, chr) in content.chars().enumerate() {
            match self.container_mode {
                ContainerMode::None => self.handle_none_char(chr, chr_idx)?,
                ContainerMode::String => self.handle_string_char(chr, chr_idx)?,
                ContainerMode::Array => self.handle_array_char(chr, chr_idx)?,
            }
        }
        Ok(())
    }

    fn handle_none_char(&mut self, chr: char, chr_idx: usize) -> Result<()> {
        if self.escaping {
            return Err(Error::msg(format!("Unexpected {} at {}", chr, chr_idx)));
        }
        match chr {
            '[' => self.container_mode = ContainerMode::Array,
            '"' => self.container_mode = ContainerMode::String,
            ',' => {}
            _ => {
                return Err(Error::msg(format!(
                    "Unexpected {} at {}, outside of array/string",
                    chr, chr_idx
                )));
            }
        }
        Ok(())
    }

    fn finish_num(&mut self, chr_idx: usize) -> Result<()> {
        self.value_mode = ValueMode::None;
        match self.current_content.parse::<u8>() {
            Ok(num) => {
                self.current_array.push(num);
                self.current_content.clear();
            }
            Err(_) => return Err(Error::msg(format!("Invalid number at {}", chr_idx))),
        }
        Ok(())
    }

    fn finish_hex(&mut self, chr_idx: usize) -> Result<()> {
        self.value_mode = ValueMode::None;
        match u8::from_str_radix(&self.current_content, 16) {
            Ok(num) => {
                self.current_array.push(num);
                self.current_content.clear();
            }
            Err(_) => return Err(Error::msg(format!("Invalid number at {}", chr_idx))),
        }
        Ok(())
    }

    fn finish_char(&mut self, chr_idx: usize) -> Result<()> {
        if self.current_content.chars().count() == 3
            && self.current_content.starts_with('\'')
            && self.current_content.ends_with('\'')
        {
            self.value_mode = ValueMode::None;
            self.current_array
                .push(self.current_content.chars().nth(1).unwrap() as u8);
            self.current_content.clear();
        } else {
            return Err(Error::msg(format!("Unable to parse char at {}", chr_idx)));
        }
        Ok(())
    }

    fn finish_array(&mut self, chr_idx: usize) -> Result<()> {
        self.container_mode = ContainerMode::None;
        if self.current_array.is_empty() {
            return Err(Error::msg(format!("Empty array at {}", chr_idx)));
        }
        self.output.push(self.current_array.clone());
        self.current_array.clear();
        Ok(())
    }

    fn handle_string_char(&mut self, chr: char, chr_idx: usize) -> Result<()> {
        if self.escaping {
            self.escaping = false;
            self.current_content.push('\\');
            self.current_content.push(chr);
        } else {
            match chr {
                '\\' => self.escaping = true,
                '"' => {
                    if self.current_content.is_empty() {
                        return Err(Error::msg(format!("Empty string at {}", chr_idx)));
                    }
                    self.output.push(self.current_content.as_bytes().to_vec());
                    self.current_content = String::new();
                    self.container_mode = ContainerMode::None;
                }
                _ => self.current_content.push(chr),
            }
        }
        Ok(())
    }

    fn handle_array_char(&mut self, chr: char, chr_idx: usize) -> Result<()> {
        match chr {
            ']' => match self.value_mode {
                ValueMode::None => {
                    if self.escaping {
                        return Err(Error::msg(format!("Unexpected ] at {}", chr_idx)));
                    } else {
                        self.finish_array(chr_idx)?;
                    }
                }
                ValueMode::Number => {
                    self.finish_num(chr_idx)?;
                    self.finish_array(chr_idx)?;
                }
                ValueMode::Hex => {
                    self.finish_hex(chr_idx)?;
                    self.finish_array(chr_idx)?;
                }
                ValueMode::Char => {
                    if self.current_content.chars().count() == 1 {
                        self.current_content.push('[');
                    } else {
                        self.finish_char(chr_idx)?;
                    }
                }
            },
            ',' => match self.value_mode {
                ValueMode::None => {}
                ValueMode::Number => self.finish_num(chr_idx)?,
                ValueMode::Hex => self.finish_hex(chr_idx)?,
                ValueMode::Char => {
                    if self.current_content == String::from("'") {
                        self.current_content.push(',')
                    } else {
                        self.finish_char(chr_idx)?;
                    }
                }
            },
            '\'' => match self.value_mode {
                ValueMode::None => {
                    self.current_content.push('\'');
                    self.value_mode = ValueMode::Char;
                }
                ValueMode::Hex | ValueMode::Number => {
                    return Err(Error::msg(format!("Unexpected ' at {}", chr_idx)))
                }
                ValueMode::Char => match self.current_content.chars().count() {
                    1 => self.current_content.push('\''),
                    2 => {
                        self.current_content.push('\'');
                        self.finish_char(chr_idx)?;
                    }
                    _ => return Err(Error::msg(format!("Unexpected ' at {}", chr_idx))),
                },
            },
            'x' => match self.value_mode {
                ValueMode::None => {
                    self.value_mode = ValueMode::Hex;
                }
                ValueMode::Number | ValueMode::Hex => {
                    return Err(Error::msg(format!("Unexpected x at {}", chr_idx)))
                }
                ValueMode::Char => self.current_content.push('x'),
            },
            '0'..='9' => match self.value_mode {
                ValueMode::None => {
                    self.current_content.push(chr);
                    self.value_mode = ValueMode::Number;
                }
                ValueMode::Number => self.current_content.push(chr),
                ValueMode::Hex => self.current_content.push(chr),
                ValueMode::Char => self.current_content.push(chr),
            },
            'A'..='F' | 'a'..='f' => match self.value_mode {
                ValueMode::Number | ValueMode::None => {
                    return Err(Error::msg(format!("Unexpected {} at {}", chr, chr_idx)))
                }
                ValueMode::Hex => self.current_content.push(chr),
                ValueMode::Char => self.current_content.push(chr),
            },
            ' ' => { /* ignore whitespace*/ }
            _ => match self.value_mode {
                ValueMode::Number | ValueMode::Hex | ValueMode::None => {
                    return Err(Error::msg(format!("Unexpected {} at {}", chr, chr_idx)))
                }
                ValueMode::Char => self.current_content.push(chr),
            },
        }
        Ok(())
    }
}

fn validate_content(content: &str) -> Result<String> {
    let trimmed = content.trim();
    return if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let mut chars = trimmed.chars();
        chars.next();
        chars.next_back();
        Ok(chars.collect::<String>().trim().to_owned())
    } else {
        Err(Error::msg("Invalid data definition, data must be made of an array with arrays or strings inside.\ne.g. [[1,2],['a','b'],\"ex\"]"))
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_validate_content() {
        let valid_simple = "[[1,2]]";
        assert_eq!(
            validate_content(valid_simple).unwrap(),
            String::from("[1,2]")
        );
        let valid_complex = "[[1,2],\"abc\",[3,'a']]";
        assert_eq!(
            validate_content(valid_complex).unwrap(),
            String::from("[1,2],\"abc\",[3,'a']")
        );
        let valid_spacing = "  [  [ 1 , 2 ] , \" a bc\", [3 , 'a' ] ] ";
        assert_eq!(
            validate_content(valid_spacing).unwrap(),
            String::from("[ 1 , 2 ] , \" a bc\", [3 , 'a' ]")
        );
        let valid_incorrect = "[[1,2]";
        assert_eq!(
            validate_content(valid_incorrect).unwrap(),
            String::from("[1,2")
        );
        let invalid_end = "[[10";
        assert!(
            validate_content(invalid_end).is_err(),
            "{}",
            validate_content(invalid_end).unwrap_err()
        );
        let invalid_start = "10]]";
        assert!(
            validate_content(invalid_start).is_err(),
            "{}",
            validate_content(invalid_start).unwrap_err()
        );
        let invalid_both = "10";
        assert!(
            validate_content(invalid_both).is_err(),
            "{}",
            validate_content(invalid_both).unwrap_err()
        );
    }

    mod integration {
        use super::*;

        #[test]
        fn basic_parsing() {
            let mut parser = DataParser::new();
            parser.run("[[1]]").unwrap();
            assert_eq!(parser.into_bytes().unwrap(), vec![1, 1, 1]);

            let mut parser = DataParser::new();
            parser.run("[[40, 41]]").unwrap();
            assert_eq!(parser.into_bytes().unwrap(), vec![1, 2, 40, 41]);

            let mut parser = DataParser::new();
            parser.run("[['a', 'b'],['c', 'd']]").unwrap();
            assert_eq!(parser.into_bytes().unwrap(), vec![2, 2, 2, 97, 98, 99, 100]);
        }

        #[test]
        fn test_numbers() {
            let mut parser = DataParser::new();
            parser.container_mode = ContainerMode::Array;
            parser.handle_array_char('2', 0).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::from("2"));
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::Number);
            assert!(!parser.escaping);
            parser.handle_array_char('0', 1).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::from("20"));
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::Number);
            assert!(!parser.escaping);
            parser.handle_array_char(',', 2).unwrap();
            assert_eq!(parser.current_array, vec![20]);
            assert_eq!(parser.current_content, String::new());
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::None);
            assert!(!parser.escaping);
            parser.handle_array_char('x', 3).unwrap();
            assert_eq!(parser.current_array, vec![20]);
            assert_eq!(parser.current_content, String::new());
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::Hex);
            assert!(!parser.escaping);
            parser.handle_array_char('2', 4).unwrap();
            assert_eq!(parser.current_array, vec![20]);
            assert_eq!(parser.current_content, String::from("2"));
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::Hex);
            assert!(!parser.escaping);
            parser.handle_array_char('F', 5).unwrap();
            assert_eq!(parser.current_array, vec![20]);
            assert_eq!(parser.current_content, String::from("2F"));
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::Hex);
            assert!(!parser.escaping);
            parser.handle_array_char(']', 6).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::new());
            assert_eq!(parser.container_mode, ContainerMode::None);
            assert_eq!(parser.value_mode, ValueMode::None);
            assert!(!parser.escaping);
            assert_eq!(parser.output, vec![vec![20, 47]]);
        }

        #[test]
        fn simple_number_array() {
            let mut parser = DataParser::new();
            parser.handle_none_char('[', 0).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::new());
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::None);
            assert!(!parser.escaping);
            parser.handle_array_char('1', 1).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::from("1"));
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::Number);
            assert!(!parser.escaping);
            parser.handle_array_char('0', 2).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::from("10"));
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::Number);
            assert!(!parser.escaping);
            parser.handle_array_char(',', 3).unwrap();
            assert_eq!(parser.current_array, vec![10]);
            assert_eq!(parser.current_content, String::new());
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::None);
            assert!(!parser.escaping);
            parser.handle_array_char('2', 4).unwrap();
            assert_eq!(parser.current_array, vec![10]);
            assert_eq!(parser.current_content, String::from("2"));
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::Number);
            assert!(!parser.escaping);
            parser.handle_array_char('0', 5).unwrap();
            assert_eq!(parser.current_array, vec![10]);
            assert_eq!(parser.current_content, String::from("20"));
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::Number);
            assert!(!parser.escaping);
            parser.handle_array_char(']', 6).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::new());
            assert_eq!(parser.container_mode, ContainerMode::None);
            assert_eq!(parser.value_mode, ValueMode::None);
            assert_eq!(parser.output, vec![vec![10, 20]]);
            assert!(!parser.escaping);
        }

        #[test]
        fn test_string() {
            let mut parser = DataParser::new();
            parser.handle_none_char('"', 0).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::new());
            assert_eq!(parser.container_mode, ContainerMode::String);
            assert_eq!(parser.value_mode, ValueMode::None);
            assert!(!parser.escaping);
            parser.handle_string_char('\'', 1).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::from("'"));
            assert_eq!(parser.container_mode, ContainerMode::String);
            assert_eq!(parser.value_mode, ValueMode::None);
            assert!(!parser.escaping);
            parser.handle_string_char('a', 2).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::from("'a"));
            assert_eq!(parser.container_mode, ContainerMode::String);
            assert_eq!(parser.value_mode, ValueMode::None);
            assert!(!parser.escaping);
            parser.handle_string_char('"', 3).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::new());
            assert_eq!(parser.container_mode, ContainerMode::None);
            assert_eq!(parser.value_mode, ValueMode::None);
            assert_eq!(parser.output, vec![vec![39, 97]]);
            assert!(!parser.escaping);
        }
    }

    mod handling_quote {
        use super::*;

        #[test]
        fn no_mode() {
            let mut parser = DataParser::new();
            let result = parser.handle_none_char('\'', 0);
            assert!(result.is_err(), "{}", result.unwrap_err());
        }

        #[test]
        fn string() {
            let mut parser = DataParser::new();
            parser.container_mode = ContainerMode::String;
            parser.handle_string_char('\'', 1).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::from("'"));
            assert_eq!(parser.container_mode, ContainerMode::String);
            assert_eq!(parser.value_mode, ValueMode::None);
            assert!(!parser.escaping);
        }

        #[test]
        fn array() {
            let mut parser = DataParser::new();
            parser.container_mode = ContainerMode::Array;
            parser.handle_array_char('\'', 1).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::from("'"));
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::Char);
            assert!(!parser.escaping);
        }

        #[test]
        fn finishing() {
            let mut parser = DataParser::new();
            parser.container_mode = ContainerMode::Array;
            parser.handle_array_char('\'', 0).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::from("'"));
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::Char);
            assert!(!parser.escaping);
            parser.handle_array_char('a', 1).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::from("'a"));
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::Char);
            assert!(!parser.escaping);
            parser.handle_array_char('\'', 2).unwrap();
            assert_eq!(parser.current_array, vec![97]);
            assert_eq!(parser.current_content, String::new());
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::None);
            assert!(!parser.escaping);
        }
    }

    mod handling_array_end_valid {
        use super::*;

        #[test]
        fn no_mode() {
            let mut parser = DataParser::new();
            let result = parser.handle_array_char(']', 0);
            assert!(result.is_err());
        }

        #[test]
        fn string() {
            let mut parser = DataParser::new();
            parser.container_mode = ContainerMode::String;
            parser.handle_string_char(']', 0).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::from("]"));
            assert_eq!(parser.container_mode, ContainerMode::String);
            assert_eq!(parser.value_mode, ValueMode::None);
            assert!(!parser.escaping);
        }

        #[test]
        fn array_empty() {
            let mut parser = DataParser::new();
            parser.container_mode = ContainerMode::Array;
            let result = parser.handle_array_char(']', 1);
            assert!(result.is_err());
        }

        #[test]
        fn array() {
            let mut parser = DataParser::new();
            parser.container_mode = ContainerMode::Array;
            parser.current_array = vec![1];
            parser.handle_array_char(']', 0).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::new());
            assert_eq!(parser.container_mode, ContainerMode::None);
            assert_eq!(parser.value_mode, ValueMode::None);
            assert_eq!(parser.output, vec![vec![1]]);
            assert!(!parser.escaping);
        }
    }

    mod handling_array_start_valid {
        use super::*;

        #[test]
        fn no_mode() {
            let mut parser = DataParser::new();
            parser.handle_none_char('[', 0).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::new());
            assert_eq!(parser.container_mode, ContainerMode::Array);
            assert_eq!(parser.value_mode, ValueMode::None);
            assert!(!parser.escaping);
        }

        #[test]
        fn array() {
            let mut parser = DataParser::new();
            let result = parser.handle_array_char('[', 0);
            assert!(result.is_err());
        }

        #[test]
        fn string() {
            let mut parser = DataParser::new();
            parser.container_mode = ContainerMode::String;
            parser.handle_string_char('[', 0).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::from("["));
            assert_eq!(parser.container_mode, ContainerMode::String);
            assert_eq!(parser.value_mode, ValueMode::None);
            assert!(!parser.escaping);
        }

        #[test]
        fn string_escaped_backslash() {
            let mut parser = DataParser::new();
            parser.container_mode = ContainerMode::String;
            parser.escaping = true;
            parser.handle_string_char('[', 1).unwrap();
            assert_eq!(parser.current_array, vec![]);
            assert_eq!(parser.current_content, String::from("\\["));
            assert_eq!(parser.container_mode, ContainerMode::String);
            assert_eq!(parser.value_mode, ValueMode::None);
            assert!(!parser.escaping);
        }
    }
}
