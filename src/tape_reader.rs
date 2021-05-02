use crate::common::read_bytes;
use crate::constants::system::*;
use anyhow::{Context, Error, Result};

pub struct Tape {
    pub name: String,
    pub version: String,
    pub ops: Vec<u8>,
    pub data: Vec<u8>,
}

pub fn read_tape(path: &str) -> Result<Tape> {
    let mut idx = 0;
    let mut bytes = read_bytes(path)?;
    if get_byte(&mut bytes, &mut idx, "header")? != TAPE_HEADER_1
        || get_byte(&mut bytes, &mut idx, "header")? != TAPE_HEADER_2
    {
        return Err(Error::msg("Not a TD tape file"));
    }
    if get_byte(&mut bytes, &mut idx, "tape version")? != PRG_VERSION {
        return Err(Error::msg("Incompatible TD version"));
    }
    let name = read_string(&mut bytes, &mut idx, "program name")?;
    let version = read_string(&mut bytes, &mut idx, "program version")?;
    let pc_byte_count = u16::from_be_bytes([
        get_byte(&mut bytes, &mut idx, "program op count")?,
        get_byte(&mut bytes, &mut idx, "program op count")?,
    ]) as usize;
    let mut ops = vec![];
    for _ in 0..pc_byte_count {
        ops.push(get_byte(&mut bytes, &mut idx, "program")?);
    }

    Ok(Tape {
        name,
        version,
        ops,
        data: bytes,
    })
}

fn read_string(bytes: &mut Vec<u8>, idx: &mut usize, name: &str) -> Result<String> {
    let length = get_byte(bytes, idx, name)?;
    let mut str_bytes = vec![];
    for _ in 0..length {
        str_bytes.push(get_byte(bytes, idx, name)?);
    }
    String::from_utf8(str_bytes).context(format!("parsing {}", name))
}

fn get_byte(bytes: &mut Vec<u8>, idx: &mut usize, area: &str) -> Result<u8> {
    *idx += 1;
    if !bytes.is_empty() {
        Ok(bytes.remove(0))
    } else {
        Err(Error::msg(format!(
            "Unexpected EoF at byte {} when parsing {}",
            idx, area
        )))
    }
}
