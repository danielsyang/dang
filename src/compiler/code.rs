// u8 -> byte
pub type Instructions = Vec<u8>;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    Constant,
    OpAdd,
    OpPop,
    OpSub,
    OpMul,
    OpDiv,
    OpTrue,
    OpFalse,
    OpEqual,
    OpNotEqual,
    OpGreaterThan,
}

pub struct Definition {
    pub name: &'static str,
    pub operands_widths: &'static [u16],
}

impl Opcode {
    pub fn lookup(&self) -> Definition {
        match self {
            Opcode::Constant => Definition {
                name: "OpConstant",
                operands_widths: &[2],
            },
            Opcode::OpAdd => Definition {
                name: "OpAdd",
                operands_widths: &[],
            },
            Opcode::OpPop => Definition {
                name: "OpPop",
                operands_widths: &[],
            },
            Opcode::OpSub => Definition {
                name: "OpSub",
                operands_widths: &[],
            },
            Opcode::OpMul => Definition {
                name: "OpMul",
                operands_widths: &[],
            },
            Opcode::OpDiv => Definition {
                name: "OpDiv",
                operands_widths: &[],
            },
            Opcode::OpTrue => Definition {
                name: "OpTrue",
                operands_widths: &[],
            },
            Opcode::OpFalse => Definition {
                name: "OpFalse",
                operands_widths: &[],
            },
            Opcode::OpEqual => Definition {
                name: "OpEqual",
                operands_widths: &[],
            },
            Opcode::OpNotEqual => Definition {
                name: "OpNotEqual",
                operands_widths: &[],
            },
            Opcode::OpGreaterThan => Definition {
                name: "OpGreaterThan",
                operands_widths: &[],
            },
        }
    }

    pub fn make(&self, operands: &[u16]) -> Instructions {
        let definition = self.lookup();

        let mut instructions_len = 1;

        for op in definition.operands_widths {
            instructions_len = instructions_len + op;
        }

        let mut instruction: Instructions = Vec::with_capacity(instructions_len as usize);
        instruction.push(*self as u8);

        for (i, op) in operands.iter().enumerate() {
            let width = definition.operands_widths.get(i);

            match width {
                None => {}
                Some(val) => {
                    if *val == 2 {
                        let o = *op as u16;

                        for byte in o.to_be_bytes() {
                            instruction.push(byte);
                        }
                    }
                }
            };
        }

        instruction
    }
}

impl From<Opcode> for u8 {
    fn from(value: Opcode) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for Opcode {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Opcode::Constant),
            1 => Ok(Opcode::OpAdd),
            2 => Ok(Opcode::OpPop),
            3 => Ok(Opcode::OpSub),
            4 => Ok(Opcode::OpMul),
            5 => Ok(Opcode::OpDiv),
            6 => Ok(Opcode::OpTrue),
            7 => Ok(Opcode::OpFalse),
            8 => Ok(Opcode::OpEqual),
            9 => Ok(Opcode::OpNotEqual),
            10 => Ok(Opcode::OpGreaterThan),
            rest => Err(rest),
        }
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use crate::compiler::code::Opcode;

    #[rstest]
    #[case(Opcode::Constant,vec![65534], vec![Opcode::Constant.into(), 255, 254])]
    #[case(Opcode::OpAdd,vec![], vec![Opcode::OpAdd as u8])]
    #[case(Opcode::OpSub,vec![], vec![Opcode::OpSub as u8])]
    #[case(Opcode::OpMul,vec![], vec![Opcode::OpMul as u8])]
    #[case(Opcode::OpDiv,vec![], vec![Opcode::OpDiv as u8])]
    #[case(Opcode::OpTrue,vec![], vec![Opcode::OpTrue as u8])]
    #[case(Opcode::OpFalse,vec![], vec![Opcode::OpFalse as u8])]
    fn make_test(#[case] op: Opcode, #[case] operands: Vec<u16>, #[case] expected: Vec<u8>) {
        let instructions: Vec<u8> = op.make(&operands);
        assert_eq!(instructions, expected);
    }
}
