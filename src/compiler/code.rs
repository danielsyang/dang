use num_enum::{IntoPrimitive, TryFromPrimitive};

// u8 -> byte
pub type Instructions = Vec<u8>;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
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
    OpJumpNotTruthy,
    OpJump,
    OpNone,
    OpBang,
    OpMinus,
    OpSetGlobal,
    OpGetGlobal,
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
            Opcode::OpJumpNotTruthy => Definition {
                name: "OpJumpNotTruthy",
                operands_widths: &[2],
            },
            Opcode::OpJump => Definition {
                name: "OpJump",
                operands_widths: &[2],
            },
            Opcode::OpNone => Definition {
                name: "OpNone",
                operands_widths: &[],
            },
            Opcode::OpBang => Definition {
                name: "OpBang",
                operands_widths: &[],
            },
            Opcode::OpMinus => Definition {
                name: "OpMinus",
                operands_widths: &[],
            },
            Opcode::OpGetGlobal => Definition {
                name: "OpGetGlobal",
                operands_widths: &[2],
            },
            Opcode::OpSetGlobal => Definition {
                name: "OpSetGlobal",
                operands_widths: &[2],
            },
        }
    }

    pub fn make(&self, operands: &[u16]) -> Instructions {
        let definition = self.lookup();

        let mut instructions_len = 1;

        for op in definition.operands_widths {
            instructions_len += op;
        }

        let mut instruction: Instructions = Vec::with_capacity(instructions_len as usize);
        instruction.push(*self as u8);

        for (i, op) in operands.iter().enumerate() {
            let width = definition.operands_widths.get(i);

            match width {
                Some(2) => {
                    let o = *op;
                    for byte in o.to_be_bytes() {
                        instruction.push(byte);
                    }
                }
                _ => {}
            };
        }

        instruction
    }

    pub fn read_operands(definition: &Definition, instructions: &[u8]) -> (Vec<u16>, usize) {
        let mut operands = Vec::with_capacity(definition.operands_widths.len());
        let mut offset = 0;

        for width in definition.operands_widths {
            let width = *width as usize;

            match width {
                2 => {
                    let value =
                        u16::from_be_bytes([instructions[offset], instructions[offset + 1]]);
                    operands.push(value);
                }
                _ => {}
            }

            offset += width;
        }

        (operands, offset)
    }
}

// For helping with debugging tests only for now
#[allow(dead_code)]
pub fn format_instructions(instructions: &Instructions) -> String {
    let mut out = String::new();
    let mut offset = 0;

    while offset < instructions.len() {
        let opcode_from_byte: Opcode = match instructions.get(offset) {
            Some(byte) => Opcode::try_from(*byte).unwrap(),
            None => panic!("Something went wrong in pub fn format_instructions"),
        };

        let definition = opcode_from_byte.lookup();

        let (operands, bytes_read) =
            Opcode::read_operands(&definition, &instructions[offset + 1..]);

        let operand_string = operands
            .iter()
            .map(|x| format!(" {}", x))
            .collect::<String>();

        out.push_str(&format!("{:04} ", offset));
        out.push_str(definition.name);
        out.push_str(&operand_string);
        out.push('\n');

        offset += 1 + bytes_read;
    }

    out
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use crate::compiler::code::{Instructions, Opcode, format_instructions};

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

    #[rstest]
    #[case([Opcode::OpAdd.make(&[]), Opcode::OpPop.make(&[])].concat(), "0000 OpAdd\n0001 OpPop\n")]
    #[case([Opcode::Constant.make(&[1]), Opcode::OpAdd.make(&[])].concat(), "0000 OpConstant 1\n0003 OpAdd\n")]
    fn test_format_instructions(#[case] bytes: Instructions, #[case] expected: String) {
        let result = format_instructions(&bytes);
        assert_eq!(result, expected);
    }
}
