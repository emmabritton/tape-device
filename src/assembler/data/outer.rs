use crate::assembler::data::inner::parse_inner_array;
use anyhow::{Error, Result};

#[derive(Copy, Clone, PartialEq, Debug)]
enum OuterMode {
    String,
    Array,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Escaping {
    None,
    Backslash,
    Quote,
}

pub fn parse_outer_array(input: &str) -> Result<Vec<u8>> {
    let trimmed = input.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let mut sub_arrays = vec![];
        let mut sub_strings = vec![];
        let mut chars = trimmed.chars();
        chars.next();
        chars.next_back();
        let mut tchars = trimmed.chars();
        println!("Parsing: {}", trimmed);
        tchars.next();
        tchars.next_back();
        println!("Parsing: {}", tchars.collect::<String>());
        let mut escaped = Escaping::None;
        let mut start: Option<OuterMode> = None;
        let mut temp = String::new();
        for chr in chars {
            println!("Checking {}", chr);
            match chr {
                '\'' => process_quote(&mut start, &mut escaped, &mut temp)?,
                '[' => process_start_sub_array(&mut start, &mut escaped, &mut temp)?,
                '"' => {
                    if let Some(()) = process_double_quote(&mut start, &mut escaped, &mut temp)? {
                        sub_strings.push(temp.clone());
                        temp.clear();
                    }
                }
                '\\' => {
                    if escaped == Escaping::Quote {
                        temp.push('\\');
                    } else {
                        escaped = Escaping::Backslash;
                        println!("Escaped set to backslash");
                    }
                }
                ']' => {
                    if let Some(()) = process_end_sub_array(&mut start, &mut escaped, &mut temp)? {
                        sub_arrays.push(temp.clone());
                        temp.clear();
                    }
                }
                _ => process_outer_char(&mut start, &mut escaped, &mut temp, chr)?,
            }
        }

        if escaped != Escaping::None || start.is_some() {
            return Err(Error::msg(format!(
                "Unexpected end of line in {:?} {:?}",
                start, escaped
            )));
        }

        let mut bytes = vec![];

        println!(
            "Found {} strings, {} arrays",
            sub_strings.len(),
            sub_arrays.len()
        );

        for string in sub_strings {
            let str_bytes = parse_string(&string)?;
            if str_bytes.len() > 255 {
                return Err(Error::msg(format!("Max of 255 characters per string")));
            }
            bytes.push(str_bytes);
        }

        for array in sub_arrays {
            let arr_bytes = parse_inner_array(&array)?;
            if arr_bytes.len() > 255 {
                return Err(Error::msg(format!("Max of 255 bytes per sub array")));
            }
            bytes.push(arr_bytes);
        }

        if bytes.len() > 255 {
            return Err(Error::msg(format!(
                "Max of 255 sub arrays/strings per data definition"
            )));
        }

        let mut output = vec![];
        print!("Writing ");
        output.push(bytes.len() as u8);
        print!("{:02X}", bytes.len() as u8);
        for list in &bytes {
            output.push(list.len() as u8);
            print!("{:02X}", list.len() as u8);
        }
        println!();
        for list in &bytes {
            output.extend_from_slice(&list);
        }

        return Ok(output);
    } else {
        return Err(Error::msg(format!(
            "Unable to parse, missing outer array delimiters"
        )));
    }
}

fn process_quote(
    start: &mut Option<OuterMode>,
    escaped: &mut Escaping,
    temp: &mut String,
) -> Result<()> {
    match start {
        Some(mode) => match mode {
            OuterMode::String => temp.push('\''),
            OuterMode::Array => match escaped {
                Escaping::None => {
                    *escaped = Escaping::Quote;
                    temp.push('\'');
                    println!("Temp now {}", temp);
                    println!("Escaped set to quote");
                }
                Escaping::Backslash => {
                    return Err(Error::msg(
                        "Unexpected '\\', to use a quote character write '''",
                    ));
                }
                Escaping::Quote => {
                    let mut temp_chars = temp.chars();
                    let back_1_was_quote = temp_chars.next_back() == Some('\'');
                    let back_2_was_quote = temp_chars.next_back() == Some('\'');
                    match (back_1_was_quote, back_2_was_quote) {
                        (true, false) => {
                            temp.push('\'');
                            println!("Temp now {}", temp);
                        }
                        (true, true) | (false, true) => {
                            temp.push('\'');
                            *escaped = Escaping::None;
                            println!("Temp now {}", temp);
                            println!("Escaped set to none 10");
                        }
                        (false, false) => {
                            return Err(Error::msg(
                                "State error: quote escaping without any quotes",
                            ));
                        }
                    }
                }
            },
        },
        None => {
            return Err(Error::msg(
                "Unexpected quote, characters can only be inside sub arrays",
            ));
        }
    }

    Ok(())
}
fn process_outer_char(
    start: &mut Option<OuterMode>,
    escaped: &mut Escaping,
    temp: &mut String,
    chr: char,
) -> Result<()> {
    match start {
        Some(_) => {
            match escaped {
                Escaping::None => temp.push(chr),
                Escaping::Backslash => {
                    temp.push('\\');
                    temp.push(chr);
                    *escaped = Escaping::None;
                    println!("Escaped set to none 1");
                }
                Escaping::Quote => {
                    temp.push(chr);
                }
            }
            println!("Temp now {}", temp);
        }
        None => {
            if !(chr.is_whitespace() || chr == ',') {
                return Err(Error::msg(format!("Unexpected '{}'", chr)));
            }
        }
    }
    Ok(())
}

fn process_end_sub_array(
    start: &mut Option<OuterMode>,
    escaped: &mut Escaping,
    temp: &mut String,
) -> Result<Option<()>> {
    match start {
        Some(mode) => match mode {
            OuterMode::String => {
                if escaped == &Escaping::Backslash {
                    temp.push('\\');
                }
                temp.push(']');
                *escaped = Escaping::None;
                println!("Escaped set to none 11");
                println!("Temp now {}", temp);
            }
            OuterMode::Array => {
                if escaped == &Escaping::Quote {
                    temp.push(']');
                } else {
                    if temp.len() == 1 {
                        return Err(Error::msg("Empty arrays not allowed"));
                    }
                    temp.push(']');
                    *start = None;
                    *escaped = Escaping::None;
                    println!("Escaped set to none 3");
                    println!("Array finished: {}", temp);
                    return Ok(Some(()));
                }
            }
        },
        None => {
            return Err(Error::msg("End of array found outside array"));
        }
    }
    Ok(None)
}

fn process_double_quote(
    start: &mut Option<OuterMode>,
    escaped: &mut Escaping,
    temp: &mut String,
) -> Result<Option<()>> {
    match start {
        Some(mode) => match mode {
            OuterMode::String => {
                if escaped == &Escaping::Backslash {
                    temp.push('\\');
                    temp.push('"');
                    *escaped = Escaping::None;
                    println!("Escaped set to none 4");
                    println!("Temp now {}", temp);
                } else {
                    if escaped == &Escaping::Quote {
                        temp.push('\'');
                    }
                    temp.push('"');
                    *start = None;
                    *escaped = Escaping::None;
                    println!("Temp now {}", temp);
                    println!("Escaped set to none 5");
                    return Ok(Some(()));
                }
            }
            OuterMode::Array => {
                if escaped == &Escaping::Quote {
                    temp.push('"');
                } else {
                    return Err(Error::msg("Unexpected double quote"));
                }
            }
        },
        None => {
            if escaped != &Escaping::None {
                return Err(Error::msg("Unexpected double quote"));
            }
            temp.push('"');
            *start = Some(OuterMode::String);
            println!("Escaped set to string");
            println!("Temp now {}", temp);
        }
    }
    Ok(None)
}

fn process_start_sub_array(
    start: &mut Option<OuterMode>,
    escaped: &mut Escaping,
    temp: &mut String,
) -> Result<()> {
    match start {
        Some(mode) => match mode {
            OuterMode::String => {
                if escaped == &Escaping::Backslash {
                    temp.push('\\');
                }
                temp.push('[');
                *escaped = Escaping::None;
                println!("Temp now {}", temp);
                println!("Escaped set to none 7");
            }
            OuterMode::Array => {
                if escaped == &Escaping::Quote {
                    temp.push('[');
                } else {
                    return Err(Error::msg(
                        "Unexpected '[', only one level of sub arrays is allowed",
                    ));
                }
            }
        },
        None => {
            if escaped != &Escaping::None {
                return Err(Error::msg("Unexpected start of sub array"));
            }
            temp.push('[');
            *start = Some(OuterMode::Array);
            println!("Mode set to Array, temp now {}", temp);
        }
    }
    Ok(())
}

fn parse_string(input: &str) -> Result<Vec<u8>> {
    let mut chars = input.trim().chars();
    chars.next();
    chars.next_back();
    let str = chars.as_str();
    if str.is_empty() {
        return Err(Error::msg("Empty strings not allowed"));
    }
    println!("Parsing {}, post trim {}", input, str);

    Ok(str.chars().map(|chr| chr as u8).collect())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_string() {
        let bytes = parse_string("\"abc\"").unwrap();
        assert_eq!(vec![97, 98, 99], bytes);
        let bytes = parse_string("\"12\"").unwrap();
        assert_eq!(vec![49, 50], bytes);
        let bytes = parse_string("\"![\"").unwrap();
        assert_eq!(vec![33, 91], bytes);
    }

    #[test]
    fn test_invalid_strings() {
        assert!(parse_string("\"\"").is_err());
    }

    #[test]
    fn test_outer_array() {
        let bytes = parse_outer_array("[[1,2,3],[x1,x2,x3],['a','b','c'],\"abc\"]").unwrap();
        assert_eq!(
            vec![4, 3, 3, 3, 3, 97, 98, 99, 1, 2, 3, 1, 2, 3, 97, 98, 99],
            bytes
        )
    }

    #[test]
    fn test_long_array() {
        let bytes = parse_outer_array(
            "[[10,10,10,10,10,10,10,10],[20,20,20,20,20,20,20,20],[30,30,30,30,30,30,30,30,30]]",
        )
        .unwrap();
        assert_eq!(
            vec![
                3, 8, 8, 9, 10, 10, 10, 10, 10, 10, 10, 10, 20, 20, 20, 20, 20, 20, 20, 20, 30, 30,
                30, 30, 30, 30, 30, 30, 30
            ],
            bytes
        )
    }

    #[test]
    fn test_end_sub_array() {
        let mut mode = Some(OuterMode::Array);
        let mut escaping = Escaping::None;
        let mut temp = String::from("[1");
        process_end_sub_array(&mut mode, &mut escaping, &mut temp).unwrap();
        assert!(mode.is_none());
        assert_eq!(escaping, Escaping::None);
        assert_eq!(temp.as_str(), "[1]");
    }

    #[test]
    fn test_end_sub_array_in_char() {
        let mut mode = Some(OuterMode::Array);
        let mut escaping = Escaping::Quote;
        let mut temp = String::from("['");
        process_end_sub_array(&mut mode, &mut escaping, &mut temp).unwrap();
        assert_eq!(mode, Some(OuterMode::Array));
        assert_eq!(escaping, Escaping::Quote);
        assert_eq!(temp.as_str(), "[']");
    }

    #[test]
    fn test_tricky_array() {
        let bytes = parse_outer_array(r#"[['[',']','\','''],"\"''[']\[\]\""]"#).unwrap();
        assert_eq!(
            vec![2, 13, 4, 92, 34, 39, 39, 91, 39, 93, 92, 91, 92, 93, 92, 34, 91, 93, 92, 39],
            bytes
        )
    }
}
