use crate::language::parser::params::{Param, Parameters};
use anyhow::{Error, Result};
use std::fmt;
use std::fmt::{Display, Formatter};

pub struct Op {
    mnemonic: &'static str,
    variants: Vec<OpVariant>,
}

impl Op {
    pub fn matches(&self, text: &str) -> bool {
        self.mnemonic == text.to_ascii_uppercase()
    }

    pub fn error_text(&self) -> String {
        let mut output = format!("{} supports:", self.mnemonic);
        for variant in self.variants.iter() {
            output.push_str(&format!("\n{} {}", self.mnemonic, variant))
        }
        output
    }

    pub fn parse(&self, parts: &[&str]) -> Option<(u8, Vec<Param>)> {
        for variant in self.variants.iter() {
            if let Ok(params) = variant.parse(parts) {
                return Some((variant.opcode, params));
            }
        }
        None
    }
}

impl Op {
    pub fn new_none(mnemonic: &'static str, opcode: u8) -> Self {
        Op {
            mnemonic,
            variants: vec![OpVariant::new(opcode, vec![])],
        }
    }

    pub fn new_string(mnemonic: &'static str, opcode: u8) -> Self {
        Op {
            mnemonic,
            variants: vec![OpVariant::new(opcode, vec![Parameters::STRING_KEY])],
        }
    }

    pub fn new_areg(mnemonic: &'static str, opcode: u8) -> Self {
        Op {
            mnemonic,
            variants: vec![OpVariant::new(opcode, vec![Parameters::ADDR_REG])],
        }
    }

    pub fn new_regvaldata(
        mnemonic: &'static str,
        opcode_reg: u8,
        opcode_val: u8,
        opcode_areg: u8,
    ) -> Self {
        Op {
            mnemonic,
            variants: vec![
                OpVariant::new(opcode_reg, vec![Parameters::DATA_REG]),
                OpVariant::new(opcode_val, vec![Parameters::NUMBER]),
                OpVariant::new(opcode_areg, vec![Parameters::ADDR_REG]),
            ],
        }
    }

    pub fn new_regval(mnemonic: &'static str, opcode_reg: u8, opcode_val: u8) -> Self {
        Op {
            mnemonic,
            variants: vec![
                OpVariant::new(opcode_reg, vec![Parameters::DATA_REG]),
                OpVariant::new(opcode_val, vec![Parameters::NUMBER]),
            ],
        }
    }

    pub fn new_addrregval(mnemonic: &'static str, opcode_reg: u8, opcode_val: u8) -> Self {
        Op {
            mnemonic,
            variants: vec![
                OpVariant::new(opcode_reg, vec![Parameters::REGISTERS]),
                OpVariant::new(opcode_val, vec![Parameters::NUMBER]),
            ],
        }
    }

    pub fn new_addrreg_regval(
        mnemonic: &'static str,
        opcode_regreg: u8,
        opcode_regval: u8,
    ) -> Self {
        Op {
            mnemonic,
            variants: vec![
                OpVariant::new(
                    opcode_regreg,
                    vec![Parameters::REGISTERS, Parameters::DATA_REG],
                ),
                OpVariant::new(
                    opcode_regval,
                    vec![Parameters::REGISTERS, Parameters::NUMBER],
                ),
            ],
        }
    }

    pub fn new_single_reg(mnemonic: &'static str, opcode: u8) -> Self {
        Op {
            mnemonic,
            variants: vec![OpVariant::new(opcode, vec![Parameters::REGISTERS])],
        }
    }

    pub fn new_file_mem(
        mnemonic: &'static str,
        opcode_reg_addr: u8,
        opcode_reg_addr_reg: u8,
        opcode_val_addr: u8,
        opcode_val_addr_reg: u8,
    ) -> Self {
        Op {
            mnemonic,
            variants: vec![
                OpVariant::new(
                    opcode_reg_addr,
                    vec![Parameters::DATA_REG, Parameters::ADDRESS],
                ),
                OpVariant::new(
                    opcode_reg_addr_reg,
                    vec![Parameters::DATA_REG, Parameters::ADDR_REG],
                ),
                OpVariant::new(
                    opcode_val_addr,
                    vec![Parameters::NUMBER, Parameters::ADDRESS],
                ),
                OpVariant::new(
                    opcode_val_addr_reg,
                    vec![Parameters::NUMBER, Parameters::ADDR_REG],
                ),
            ],
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_file_mem_value(
        mnemonic: &'static str,
        opcode_reg_addr: u8,
        opcode_reg_addr_reg: u8,
        opcode_val_addr: u8,
        opcode_val_addr_reg: u8,
        opcode_reg_reg: u8,
        opcode_reg_val: u8,
        opcode_val_reg: u8,
        opcode_val_val: u8,
    ) -> Self {
        Op {
            mnemonic,
            variants: vec![
                OpVariant::new(
                    opcode_reg_addr,
                    vec![Parameters::DATA_REG, Parameters::ADDRESS],
                ),
                OpVariant::new(
                    opcode_reg_addr_reg,
                    vec![Parameters::DATA_REG, Parameters::ADDR_REG],
                ),
                OpVariant::new(
                    opcode_val_addr,
                    vec![Parameters::NUMBER, Parameters::ADDRESS],
                ),
                OpVariant::new(
                    opcode_val_addr_reg,
                    vec![Parameters::NUMBER, Parameters::ADDR_REG],
                ),
                OpVariant::new(
                    opcode_reg_reg,
                    vec![Parameters::DATA_REG, Parameters::DATA_REG],
                ),
                OpVariant::new(
                    opcode_reg_val,
                    vec![Parameters::DATA_REG, Parameters::NUMBER],
                ),
                OpVariant::new(
                    opcode_val_reg,
                    vec![Parameters::NUMBER, Parameters::DATA_REG],
                ),
                OpVariant::new(opcode_val_val, vec![Parameters::NUMBER, Parameters::NUMBER]),
            ],
        }
    }

    pub fn new_mem(mnemonic: &'static str, opcode_addr: u8, opcode_addr_reg: u8) -> Self {
        Op {
            mnemonic,
            variants: vec![
                OpVariant::new(opcode_addr, vec![Parameters::ADDRESS]),
                OpVariant::new(opcode_addr_reg, vec![Parameters::ADDR_REG]),
            ],
        }
    }

    pub fn new_regval_regval(
        mnemonic: &'static str,
        opcode_reg_reg: u8,
        opcode_reg_val: u8,
        opcode_val_reg: u8,
        opcode_val_val: u8,
    ) -> Self {
        Op {
            mnemonic,
            variants: vec![
                OpVariant::new(
                    opcode_reg_reg,
                    vec![Parameters::DATA_REG, Parameters::DATA_REG],
                ),
                OpVariant::new(
                    opcode_reg_val,
                    vec![Parameters::DATA_REG, Parameters::NUMBER],
                ),
                OpVariant::new(
                    opcode_val_reg,
                    vec![Parameters::NUMBER, Parameters::DATA_REG],
                ),
                OpVariant::new(opcode_val_val, vec![Parameters::NUMBER, Parameters::NUMBER]),
            ],
        }
    }

    pub fn new_reg_val(
        mnemonic: &'static str,
        opcode_reg_reg: u8,
        opcode_reg_val: u8,
        opcode_reg_areg: u8,
    ) -> Self {
        Op {
            mnemonic,
            variants: vec![
                OpVariant::new(
                    opcode_reg_reg,
                    vec![Parameters::DATA_REG, Parameters::DATA_REG],
                ),
                OpVariant::new(
                    opcode_reg_val,
                    vec![Parameters::DATA_REG, Parameters::NUMBER],
                ),
                OpVariant::new(
                    opcode_reg_areg,
                    vec![Parameters::DATA_REG, Parameters::ADDR_REG],
                ),
            ],
        }
    }

    pub fn new_either_reg_reg(
        mnemonic: &'static str,
        opcode_reg_reg: u8,
        opcode_areg_areg: u8,
    ) -> Self {
        Op {
            mnemonic,
            variants: vec![
                OpVariant::new(
                    opcode_reg_reg,
                    vec![Parameters::DATA_REG, Parameters::DATA_REG],
                ),
                OpVariant::new(
                    opcode_areg_areg,
                    vec![Parameters::ADDR_REG, Parameters::ADDR_REG],
                ),
            ],
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_reg_complex(
        mnemonic: &'static str,
        opcode_dreg_dreg: u8,
        opcode_dreg_num: u8,
        opcode_areg_areg: u8,
        opcode_areg_addr: u8,
        opcode_areg_dreg_dreg: u8,
        opcode_dreg_dreg_areg: u8,
        opcode_dreg_areg: u8,
    ) -> Self {
        Op {
            mnemonic,
            variants: vec![
                OpVariant::new(
                    opcode_dreg_dreg,
                    vec![Parameters::DATA_REG, Parameters::DATA_REG],
                ),
                OpVariant::new(
                    opcode_dreg_num,
                    vec![Parameters::DATA_REG, Parameters::NUMBER],
                ),
                OpVariant::new(
                    opcode_areg_areg,
                    vec![Parameters::ADDR_REG, Parameters::ADDR_REG],
                ),
                OpVariant::new(
                    opcode_areg_addr,
                    vec![Parameters::ADDR_REG, Parameters::ADDRESSES],
                ),
                OpVariant::new(
                    opcode_dreg_dreg_areg,
                    vec![
                        Parameters::DATA_REG,
                        Parameters::DATA_REG,
                        Parameters::ADDR_REG,
                    ],
                ),
                OpVariant::new(
                    opcode_areg_dreg_dreg,
                    vec![
                        Parameters::ADDR_REG,
                        Parameters::DATA_REG,
                        Parameters::DATA_REG,
                    ],
                ),
                OpVariant::new(
                    opcode_dreg_areg,
                    vec![Parameters::DATA_REG, Parameters::ADDR_REG],
                ),
            ],
        }
    }

    pub fn new_data(
        mnemonic: &'static str,
        opcode_areg_data_reg_reg: u8,
        opcode_areg_data_reg_val: u8,
        opcode_areg_data_val_reg: u8,
        opcode_areg_data_val_val: u8,
    ) -> Self {
        Op {
            mnemonic,
            variants: vec![
                OpVariant::new(
                    opcode_areg_data_reg_reg,
                    vec![
                        Parameters::ADDR_REG,
                        Parameters::DATA_KEY,
                        Parameters::DATA_REG,
                        Parameters::DATA_REG,
                    ],
                ),
                OpVariant::new(
                    opcode_areg_data_reg_val,
                    vec![
                        Parameters::ADDR_REG,
                        Parameters::DATA_KEY,
                        Parameters::DATA_REG,
                        Parameters::NUMBER,
                    ],
                ),
                OpVariant::new(
                    opcode_areg_data_val_reg,
                    vec![
                        Parameters::ADDR_REG,
                        Parameters::DATA_KEY,
                        Parameters::NUMBER,
                        Parameters::DATA_REG,
                    ],
                ),
                OpVariant::new(
                    opcode_areg_data_val_val,
                    vec![
                        Parameters::ADDR_REG,
                        Parameters::DATA_KEY,
                        Parameters::NUMBER,
                        Parameters::NUMBER,
                    ],
                ),
            ],
        }
    }

    pub fn new_jmp(mnemonic: &'static str, opcode_addr: u8, opcode_addr_reg: u8) -> Self {
        Op {
            mnemonic,
            variants: vec![
                OpVariant::new(opcode_addr_reg, vec![Parameters::ADDR_REG]),
                OpVariant::new(opcode_addr, vec![Parameters::ADDRESSES]),
            ],
        }
    }

    pub fn new_regval_jmp(
        mnemonic: &'static str,
        opcode_reg_addr: u8,
        opcode_reg_addr_reg: u8,
        opcode_val_addr: u8,
        opcode_val_addr_reg: u8,
    ) -> Self {
        Op {
            mnemonic,
            variants: vec![
                OpVariant::new(
                    opcode_reg_addr_reg,
                    vec![Parameters::DATA_REG, Parameters::ADDR_REG],
                ),
                OpVariant::new(
                    opcode_reg_addr,
                    vec![Parameters::DATA_REG, Parameters::ADDRESSES],
                ),
                OpVariant::new(
                    opcode_val_addr_reg,
                    vec![Parameters::NUMBER, Parameters::ADDR_REG],
                ),
                OpVariant::new(
                    opcode_val_addr,
                    vec![Parameters::NUMBER, Parameters::ADDRESSES],
                ),
            ],
        }
    }
}

struct OpVariant {
    opcode: u8,
    params: Vec<Parameters>,
}

impl OpVariant {
    #[allow(clippy::len_zero)]
    fn parse(&self, input: &[&str]) -> Result<Vec<Param>> {
        let mut output = vec![];
        if input.len() > self.params.len() {
            return Err(Error::msg("Too many operands"));
        }
        for (idx, param) in self.params.iter().enumerate() {
            if input.len() > idx {
                output.push(param.parse(input[idx])?);
            } else {
                return Err(Error::msg("Missing operands"));
            }
        }
        Ok(output)
    }
}

impl OpVariant {
    pub fn new(opcode: u8, params: Vec<Parameters>) -> Self {
        OpVariant { opcode, params }
    }
}

impl Display for OpVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.params
                .iter()
                .map(|param| param.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}
