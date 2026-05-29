// u8 -> byte
pub type Instructions = Vec<u8>;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Opcode {
    Constant = 0,
    OpAdd = 1,
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
                Some(val) => match val {
                    2 => {
                        let o = *op as u16;

                        for byte in o.to_be_bytes() {
                            instruction.push(byte);
                        }
                    }
                    _ => {}
                },
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

#[cfg(test)]
mod test {
    use crate::compiler::code::Opcode;

    struct Test {
        op: Opcode,
        operands: Vec<u16>,
        expected: Vec<u8>,
    }

    #[test]
    fn make_test() {
        let tests: Vec<Test> = vec![Test {
            op: Opcode::Constant,
            operands: vec![65534],
            expected: vec![Opcode::Constant.into(), 255, 254],
        }];

        for test in tests {
            let instructions: Vec<u8> = test.op.make(&test.operands);

            assert_eq!(instructions, test.expected);
        }
    }
}
