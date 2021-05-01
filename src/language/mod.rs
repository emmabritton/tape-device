use crate::language::ops::OPS;
use crate::language::parser::params::Param;
use anyhow::{Error, Result};

mod ops;
pub mod parser;

pub fn parse_line(input: &str) -> Result<(u8, Vec<Param>)> {
    let parts = input.split_whitespace().collect::<Vec<&str>>();

    for op in OPS.iter() {
        if op.matches(&parts[0]) {
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
