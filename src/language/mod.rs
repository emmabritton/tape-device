use crate::language::ops::OPS;
use crate::language::parser::params::Param;
use anyhow::{Error, Result};
use lazy_static::lazy_static;
use regex::Regex;

mod ops;
pub mod parser;

lazy_static! {
    //finds groups of non whitespace or chars
    //eg prtc @xAF 10 label 'a' ' '
    static ref LINE_REGEX: Regex = Regex::new("(?:@?\\w)+|'.'").unwrap();
}

///This method converts a BASM instruction into usable parts for the assembler
///The line can not contain any comments or a label
pub fn parse_line(input: &str) -> Result<(u8, Vec<Param>)> {
    let parts = LINE_REGEX
        .find_iter(input)
        .map(|cap| cap.as_str())
        .collect::<Vec<&str>>();

    for op in OPS.iter() {
        if op.matches(parts[0]) {
            let result = if parts.len() > 1 {
                op.parse(&parts[1..])
            } else {
                op.parse(&[])
            };
            return match result {
                None => Err(Error::msg(format!(
                    "parsing line '{}'\n{}",
                    input,
                    op.error_text()
                ))),
                Some(params) => Ok(params),
            };
        }
    }

    Err(Error::msg(format!(
        "Unable to parse {}, instruction not recognised",
        input
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::code::*;
    use crate::constants::hardware::{REG_A1, REG_D0};
    use anyhow::Context;

    //TODO write tests
}
