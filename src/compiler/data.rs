use anyhow::{Error, Result};
use std::collections::HashMap;

pub(super) fn compile_data(
    lines: Vec<String>,
    keep_whitespace: bool,
) -> Result<(HashMap<String, u16>, Vec<u8>)> {
    let mut mapping = HashMap::with_capacity(lines.len());
    let mut output = Vec::with_capacity(lines.len() * 10);
    for (line_num, line) in lines.iter().enumerate() {
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
                    "Line {} has invalid key must be [a-zA-Z0-9_]+",
                    line_num
                )));
            }
            if content.len() > 255 {
                return Err(Error::msg(format!("Line {} in strings is too long, must be at most 255 chars (including whitespace if --keep_whitespace)", line_num)));
            }
            if output.len() >= u16::MAX as usize {
                return Err(Error::msg(format!("Too many strings at line {}, max of {} chars in file including whitespace but not including keys", line_num, u16::MAX - 1)));
            }
            mapping.insert(key.to_string(), output.len() as u16);
            output.push(content.len() as u8);
            output.extend_from_slice(content.as_bytes());
        }
    }

    output.shrink_to_fit();
    mapping.shrink_to_fit();
    Ok((mapping, output))
}
