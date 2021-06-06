use anyhow::{Context, Error, Result};

#[derive(Debug, PartialEq)]
enum InnerMode {
    None,
    Number,
    Char,
    Hex,
}

pub fn parse_inner_array(input: &str) -> Result<Vec<u8>> {
    let trimmed = input.trim();
    if !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
        return Err(
            Error::msg("Error parsing, missing array delimiters").context(input.to_string())
        );
    }
    let mut output: Vec<u8> = vec![];
    let mut mode = InnerMode::None;
    let mut chars = trimmed.chars();
    chars.next();
    chars.next_back();
    let mut temp = String::new();

    while let Some(chr) = chars.next() {
        if chr == ',' {
            finish(&mut output, &mut temp, mode).context(input.to_string())?;
            mode = InnerMode::None;
        } else if chr.is_digit(10) {
            process_number(&mut temp, chr, &mut mode).context(input.to_string())?;
        } else if chr.is_digit(16) {
            process_hex_number(&mut temp, chr, &mut mode).context(input.to_string())?;
        } else if chr == 'x' {
            process_x(&mut temp, chr, &mut mode).context(input.to_string())?;
        } else if chr == '\'' {
            process_quote(&mut temp, &mut mode).context(input.to_string())?;
        } else {
            process_char(&mut temp, chr, &mut mode).context(input.to_string())?;
        }
    }
    finish(&mut output, &mut temp, mode).context(input.to_string())?;

    return Ok(output);
}

fn finish(output: &mut Vec<u8>, temp: &mut String, mode: InnerMode) -> Result<()> {
    let str = temp.trim();
    match mode {
        InnerMode::None => {
            return Err(Error::msg("Unable to parse"));
        }
        InnerMode::Number => match str.parse::<u8>() {
            Ok(val) => {
                output.push(val);
            }
            Err(err) => return Err(Error::msg(format!("Error while parsing {}: {}", temp, err))),
        },
        InnerMode::Char => {
            if temp.len() == 3 {
                let char = temp.chars().skip(1).next().unwrap();
                if char.is_ascii() {
                    let val = char as u8;
                    output.push(val)
                }
            } else {
                return Err(Error::msg(format!("Invalid char definition in {}", temp)));
            }
        }
        InnerMode::Hex => {
            let number: String = str.chars().skip(1).collect();
            match u8::from_str_radix(&number, 16) {
                Ok(val) => output.push(val),
                Err(err) => {
                    return Err(Error::msg(format!("Error while parsing {}: {}", temp, err)))
                }
            }
        }
    }
    *temp = String::new();
    Ok(())
}

fn process_number(temp: &mut String, chr: char, mode: &mut InnerMode) -> Result<()> {
    match mode {
        InnerMode::None => {
            *mode = InnerMode::Number;
            temp.push(chr);
        }
        InnerMode::Number | InnerMode::Hex => {
            temp.push(chr);
        }
        InnerMode::Char => {
            if temp.len() == 1 {
                temp.push(chr);
            } else {
                return Err(Error::msg(format!("Unexpected character '{}'", chr)));
            }
        }
    }
    Ok(())
}

fn process_hex_number(temp: &mut String, chr: char, mode: &mut InnerMode) -> Result<()> {
    match mode {
        InnerMode::None => {
            return Err(Error::msg(format!("Unexpected character '{}'", chr)));
        }
        InnerMode::Hex => {
            temp.push(chr);
        }
        InnerMode::Number => {
            return Err(Error::msg(format!("Unexpected character '{}'", chr)));
        }
        InnerMode::Char => {
            if temp.len() == 1 {
                temp.push(chr);
            } else {
                return Err(Error::msg(format!("Unexpected character '{}'", chr)));
            }
        }
    }
    Ok(())
}

fn process_quote(temp: &mut String, mode: &mut InnerMode) -> Result<()> {
    match mode {
        InnerMode::None => {
            *mode = InnerMode::Char;
            temp.push('\'');
        }
        InnerMode::Char => {
            // if temp.len() == 2 {
            temp.push('\'');
            // } else {
            //     return Err(Error::msg(format!("Unexpected character '{}'", chr)));
            // }
        }
        _ => {
            return Err(Error::msg(format!("Unexpected character '''")));
        }
    }
    Ok(())
}

fn process_x(temp: &mut String, chr: char, mode: &mut InnerMode) -> Result<()> {
    match mode {
        InnerMode::None => {
            *mode = InnerMode::Hex;
            temp.push(chr);
        }
        InnerMode::Number | InnerMode::Hex => {
            return Err(Error::msg(format!("Unexpected character '{}'", chr)));
        }
        InnerMode::Char => {
            if temp.len() == 1 {
                temp.push(chr);
            } else {
                return Err(Error::msg(format!("Unexpected character '{}'", chr)));
            }
        }
    }
    Ok(())
}

fn process_char(temp: &mut String, chr: char, mode: &mut InnerMode) -> Result<()> {
    match mode {
        InnerMode::Number | InnerMode::Hex | InnerMode::None => {
            if !chr.is_whitespace() {
                return Err(Error::msg(format!("Unexpected character '{}'", chr)));
            }
        }
        InnerMode::Char => {
            if temp.len() == 1 {
                temp.push(chr);
            } else {
                return Err(Error::msg(format!("Unexpected character '{}'", chr)));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_process_char_space_none() {
        let mut temp = String::new();
        let mut mode = InnerMode::None;
        process_char(&mut temp, ' ', &mut mode).unwrap();
        assert_eq!(temp.as_str(), "");
        assert_eq!(mode, InnerMode::None);
    }

    #[test]
    fn test_process_char_az_none() {
        let mut temp = String::new();
        let mut mode = InnerMode::None;
        for chr in 'a'..'z' {
            assert!(process_char(&mut temp, chr, &mut mode).is_err(), "{}", chr);
        }
        for chr in 'A'..'Z' {
            assert!(process_char(&mut temp, chr, &mut mode).is_err(), "{}", chr);
        }
        for chr in '0'..'9' {
            assert!(process_char(&mut temp, chr, &mut mode).is_err(), "{}", chr);
        }
    }

    #[test]
    fn test_process_char_az_char() {
        let ranges = vec!['a'..'z', 'A'..'Z', '0'..'9'];
        for range in ranges {
            for chr in range {
                let mut temp = String::from("'");
                let mut mode = InnerMode::Char;
                let mut output = vec![];
                process_char(&mut temp, chr, &mut mode).unwrap();
                assert_eq!(mode, InnerMode::Char, "{}", chr);
                assert_eq!(temp, format!("'{}", chr).as_str(), "{}", chr);
                process_quote(&mut temp, &mut mode).unwrap();
                assert_eq!(mode, InnerMode::Char, "{}", chr);
                assert_eq!(temp, format!("'{}'", chr).as_str(), "{}", chr);
                finish(&mut output, &mut temp, mode).unwrap();
                assert_eq!(temp, "", "{}", chr);
                assert_eq!(output, vec![chr as u8], "{}", chr);
            }
        }
    }

    #[test]
    fn test_process_inner() {
        let bytes = parse_inner_array("[1,2,3]").unwrap();
        assert_eq!(vec![1, 2, 3], bytes);
        let bytes = parse_inner_array("[x1,x2,x3]").unwrap();
        assert_eq!(vec![1, 2, 3], bytes);
        let bytes = parse_inner_array("['1','2','3']").unwrap();
        assert_eq!(vec![49, 50, 51], bytes);
        let bytes = parse_inner_array("[100,xF,'a']").unwrap();
        assert_eq!(vec![100, 15, 97], bytes);
    }
}
