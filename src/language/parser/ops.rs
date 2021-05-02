use crate::language::parser::params::{Param, Parameters};
use anyhow::Result;
use std::fmt;
use std::fmt::{Display, Formatter};

pub struct Op {
    mnemonic: &'static str,
    variants: [Option<OpVariant>; 2],
}

impl Op {
    pub fn matches(&self, text: &str) -> bool {
        self.mnemonic == text.to_ascii_uppercase()
    }

    pub fn error_text(&self) -> String {
        let mut output = format!("{} supports:", self.mnemonic);
        for variant in self.variants.iter().flatten() {
            output.push_str(&format!("\n{} {}", self.mnemonic, variant))
        }
        output
    }

    pub fn parse(&self, parts: &[&str]) -> Option<(u8, Vec<Param>)> {
        for variant in self.variants.iter().flatten() {
            if let Ok(params) = variant.parse(&parts) {
                return Some((variant.opcode, params));
            }
        }
        None
    }
}

impl Op {
    pub const fn new_none(mnemonic: &'static str, opcode: u8) -> Self {
        Op {
            mnemonic,
            variants: [
                OpVariant::new_double(opcode, Parameters::NONE, Parameters::NONE),
                None,
            ],
        }
    }

    pub const fn new_string(mnemonic: &'static str, opcode: u8) -> Self {
        Op {
            mnemonic,
            variants: [OpVariant::new_single(opcode, Parameters::STRING_KEY), None],
        }
    }

    pub const fn new_regval(mnemonic: &'static str, opcode_reg: u8, opcode_val: u8) -> Self {
        Op {
            mnemonic,
            variants: [
                OpVariant::new_single(opcode_reg, Parameters::DATA_REG),
                OpVariant::new_single(opcode_val, Parameters::NUMBER),
            ],
        }
    }

    pub const fn new_addrregval(mnemonic: &'static str, opcode_reg: u8, opcode_val: u8) -> Self {
        Op {
            mnemonic,
            variants: [
                OpVariant::new_single(opcode_reg, Parameters::REGISTERS),
                OpVariant::new_single(opcode_val, Parameters::NUMBER),
            ],
        }
    }

    pub const fn new_addrreg_regval(
        mnemonic: &'static str,
        opcode_regreg: u8,
        opcode_regval: u8,
    ) -> Self {
        Op {
            mnemonic,
            variants: [
                OpVariant::new_double(opcode_regreg, Parameters::REGISTERS, Parameters::DATA_REG),
                OpVariant::new_double(opcode_regval, Parameters::REGISTERS, Parameters::NUMBER),
            ],
        }
    }

    pub const fn new_single_reg(mnemonic: &'static str, opcode: u8) -> Self {
        Op {
            mnemonic,
            variants: [OpVariant::new_single(opcode, Parameters::REGISTERS), None],
        }
    }

    pub const fn new_mem(mnemonic: &'static str, opcode_addr: u8, opcode_addr_reg: u8) -> Self {
        Op {
            mnemonic,
            variants: [
                OpVariant::new_single(opcode_addr, Parameters::ADDRESS),
                OpVariant::new_single(opcode_addr_reg, Parameters::ADDR_REG),
            ],
        }
    }

    pub const fn new_reg_val(
        mnemonic: &'static str,
        opcode_reg_reg: u8,
        opcode_reg_val: u8,
    ) -> Self {
        Op {
            mnemonic,
            variants: [
                OpVariant::new_double(opcode_reg_reg, Parameters::DATA_REG, Parameters::DATA_REG),
                OpVariant::new_double(opcode_reg_val, Parameters::DATA_REG, Parameters::NUMBER),
            ],
        }
    }

    pub const fn new_reg_reg_addr(
        mnemonic: &'static str,
        opcode_reg_reg: u8,
        opcode_addr: u8,
    ) -> Self {
        Op {
            mnemonic,
            variants: [
                OpVariant::new_double(opcode_reg_reg, Parameters::DATA_REG, Parameters::DATA_REG),
                OpVariant::new_single(opcode_addr, Parameters::ADDRESSES),
            ],
        }
    }

    pub const fn new_reg_reg(mnemonic: &'static str, opcode_reg_reg: u8) -> Self {
        Op {
            mnemonic,
            variants: [
                OpVariant::new_double(opcode_reg_reg, Parameters::DATA_REG, Parameters::DATA_REG),
                None,
            ],
        }
    }

    pub const fn new_jmp(mnemonic: &'static str, opcode_addr: u8, opcode_addr_reg: u8) -> Self {
        Op {
            mnemonic,
            variants: [
                OpVariant::new_single(opcode_addr_reg, Parameters::ADDR_REG),
                OpVariant::new_single(opcode_addr, Parameters::ADDRESSES),
            ],
        }
    }
}

struct OpVariant {
    opcode: u8,
    params: [Parameters; 2],
}

impl OpVariant {
    #[allow(clippy::len_zero)]
    fn parse(&self, input: &[&str]) -> Result<Vec<Param>> {
        let mut output = vec![];
        if input.len() > 0 {
            output.push(self.params[0].parse(input[0])?);
        }
        if input.len() > 1 {
            output.push(self.params[1].parse(input[1])?);
        }
        Ok(output)
    }
}

impl OpVariant {
    const fn new_single(opcode: u8, param: Parameters) -> Option<Self> {
        Some(OpVariant {
            opcode,
            params: [param, Parameters::NONE],
        })
    }

    const fn new_double(opcode: u8, param1: Parameters, param2: Parameters) -> Option<Self> {
        Some(OpVariant {
            opcode,
            params: [param1, param2],
        })
    }
}

impl Display for OpVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.params[0], self.params[1])
    }
}
