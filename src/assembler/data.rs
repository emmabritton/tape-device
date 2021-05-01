use crate::assembler::FORMAT_ERROR;
use anyhow::{Error, Result};
use std::collections::HashMap;

pub(super) fn compile_strings(
    lines: &mut Vec<String>,
    keep_whitespace: bool,
) -> Result<(HashMap<String, u16>, Vec<u8>)> {
    let mut mapping = HashMap::with_capacity(lines.len());
    let mut output = Vec::with_capacity(lines.len() * 10);
    let mut line = lines.remove(0);
    while line != ".ops" {
        if let Some(idx) = line.find('=') {
            let (key, content) = line.split_at(idx);
            let mut content: String = content.chars().skip(1).collect();
            if !keep_whitespace {
                content = content.trim().to_string();
            }
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
                return Err(Error::msg(format!("Line '{}' in strings is too long, must be at most 255 chars (including whitespace if --keep_whitespace)", line)));
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

    output.shrink_to_fit();
    mapping.shrink_to_fit();
    Ok((mapping, output))
}
