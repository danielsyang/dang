use num_enum::TryFromPrimitive;

use crate::{
    ast::{
        expression::{Expression, Operator},
        literal::Literal,
        statement::Statement,
    },
    compiler::code::{
        Instructions,
        Opcode::{self},
    },
    eval::object::Object,
};

struct EmittedInstruction {
    opcode: Opcode,
    position: usize,
}

pub struct Compiler {
    instructions: Instructions,
    constants: Vec<Object>,

    last_instruction: Option<EmittedInstruction>,
    previous_instruction: Option<EmittedInstruction>,
}

#[derive(PartialEq, Eq)]
pub struct Bytecode {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            instructions: vec![],
            constants: vec![],
            last_instruction: None,
            previous_instruction: None,
        }
    }

    pub fn compile(&mut self, statements: &Vec<Statement>) {
        for sttm in statements {
            match sttm {
                Statement::Expression(expr) => {
                    self.compile_expression(expr);
                    self.emit(Opcode::OpPop, &[]);
                }
                _ => todo!("TODO: not implemented."),
            }
        }
    }

    fn compile_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Infix(Operator::LessThan, left_expr, right_expr) => {
                self.compile_expression(right_expr);
                self.compile_expression(left_expr);
                self.emit(Opcode::OpGreaterThan, &[]);
            }
            Expression::Infix(op, left_expr, right_expr) => {
                self.compile_expression(left_expr);
                self.compile_expression(right_expr);

                match op {
                    Operator::Plus => self.emit(Opcode::OpAdd, &[]),
                    Operator::Minus => self.emit(Opcode::OpSub, &[]),
                    Operator::Multiply => self.emit(Opcode::OpMul, &[]),
                    Operator::Divide => self.emit(Opcode::OpDiv, &[]),
                    Operator::Equal => self.emit(Opcode::OpEqual, &[]),
                    Operator::NotEqual => self.emit(Opcode::OpNotEqual, &[]),
                    Operator::GreaterThan => self.emit(Opcode::OpGreaterThan, &[]),
                    _ => todo!("{} operator not implement", op),
                };
            }
            Expression::Literal(Literal::Number(val)) => {
                let position = self.add_constant(*val);
                self.emit(Opcode::Constant, &[position as u16]);
            }
            Expression::Literal(Literal::Boolean(val)) => match val {
                true => {
                    self.emit(Opcode::OpTrue, &[]);
                }
                false => {
                    self.emit(Opcode::OpFalse, &[]);
                }
            },
            Expression::If {
                condition,
                consequence,
                alternative,
            } => {
                self.compile_expression(condition);

                let op_jump_pos = self.emit(Opcode::OpJumpNotTruthy, &[9999]);

                self.compile(&consequence);

                if self.is_last_instruction_pop() {
                    self.remove_last_pop();
                }

                match alternative {
                    None => {
                        let after_consequence_pos = self.instructions.len();
                        self.change_operand(op_jump_pos, after_consequence_pos);
                    }
                    Some(alt) => {
                        let jump_pos = self.emit(Opcode::OpJump, &[9999]);

                        let after_consequence_pos = self.instructions.len();
                        self.change_operand(op_jump_pos, after_consequence_pos);

                        self.compile(&alt);

                        if self.is_last_instruction_pop() {
                            self.remove_last_pop();
                        }

                        let after_alternative_pos = self.instructions.len();
                        self.change_operand(jump_pos, after_alternative_pos);
                    }
                }
            }
            _ => {
                todo!("TODO: not implemented.")
            }
        }
    }

    fn add_constant(&mut self, val: i64) -> usize {
        self.constants.push(Object::Number(val.clone()));

        self.constants.len() - 1
    }

    fn emit(&mut self, op_code: Opcode, operands: &[u16]) -> usize {
        let pos = self.instructions.len();
        let val = op_code.make(operands);

        for v in val {
            self.instructions.push(v);
        }

        self.set_last_instructions(op_code);

        pos
    }

    fn set_last_instructions(&mut self, op_code: Opcode) {
        self.previous_instruction = self.last_instruction.take();

        self.last_instruction = Some(EmittedInstruction {
            opcode: op_code,
            position: self.instructions.len(),
        });
    }

    fn is_last_instruction_pop(&self) -> bool {
        matches!(
            &self.last_instruction,
            Some(EmittedInstruction {
                opcode: Opcode::OpPop,
                ..
            })
        )
    }

    fn remove_last_pop(&mut self) {
        self.instructions.pop();
        self.last_instruction = self.previous_instruction.take();
    }

    fn replace_instructions(&mut self, position: usize, new_instructions: Instructions) {
        for i in 0..new_instructions.len() {
            self.instructions[position + i] = new_instructions[i];
        }
    }

    fn change_operand(&mut self, op_pos: usize, operand: usize) {
        let op = Opcode::try_from_primitive(
            *self
                .instructions
                .get(op_pos)
                .unwrap_or_else(|| panic!("invalid op_pos: {}", op_pos)),
        )
        .unwrap_or_else(|val| panic!("invalid everything: {}", val))
        .make(&[operand as u16]);

        self.replace_instructions(op_pos, op);
    }

    pub fn to_bytecode(&self) -> Bytecode {
        Bytecode {
            instructions: self.instructions.clone(),
            constants: self.constants.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use crate::{
        ast::parser::Parser,
        compiler::{
            code::{Instructions, Opcode, format_instructions},
            compiler::Compiler,
        },
        eval::object::Object,
        intern::interner::Interner,
    };

    #[rstest]
    #[case::arithmetic("1 + 2", vec![Object::Number(1), Object::Number(2)], vec![Opcode::Constant.make(&[0]),Opcode::Constant.make(&[1]),Opcode::OpAdd.make(&[]),Opcode::OpPop.make(&[])])]
    #[case::true_boolean("true", vec![], vec![Opcode::OpTrue.make(&[]), Opcode::OpPop.make(&[])])]
    #[case::false_boolean("false", vec![], vec![Opcode::OpFalse.make(&[]), Opcode::OpPop.make(&[])])]
    #[case::equal("1 == 2", vec![Object::Number(1), Object::Number(2)], vec![Opcode::Constant.make(&[0]), Opcode::Constant.make(&[1]), Opcode::OpEqual.make(&[]), Opcode::OpPop.make(&[])])]
    #[case::greater_than("1 > 2", vec![Object::Number(1), Object::Number(2)], vec![Opcode::Constant.make(&[0]), Opcode::Constant.make(&[1]), Opcode::OpGreaterThan.make(&[]), Opcode::OpPop.make(&[])])]
    #[case::less_than("1 < 2", vec![Object::Number(2), Object::Number(1)], vec![Opcode::Constant.make(&[0]), Opcode::Constant.make(&[1]), Opcode::OpGreaterThan.make(&[]), Opcode::OpPop.make(&[])])]
    #[case::not_equal("1 != 2", vec![Object::Number(1), Object::Number(2)], vec![Opcode::Constant.make(&[0]), Opcode::Constant.make(&[1]), Opcode::OpNotEqual.make(&[]), Opcode::OpPop.make(&[])])]
    #[case::if_expression("if (true) { 10 }; 3333; ",
        vec![Object::Number(10), Object::Number(3333)],
        vec![Opcode::OpTrue.make(&[]), Opcode::OpJumpNotTruthy.make(&[7]), Opcode::Constant.make(&[0]), Opcode::OpPop.make(&[]), Opcode::Constant.make(&[1]), Opcode::OpPop.make(&[])]
    )]
    #[case::if_else_expression("if (true) { 10 } else { 20 }; 3333; ",
        vec![Object::Number(10), Object::Number(20),Object::Number(3333)],
        vec![Opcode::OpTrue.make(&[]), Opcode::OpJumpNotTruthy.make(&[10]), Opcode::Constant.make(&[0]), Opcode::OpJump.make(&[13]), Opcode::Constant.make(&[1]), Opcode::OpPop.make(&[]), Opcode::Constant.make(&[2]), Opcode::OpPop.make(&[])]
    )]
    fn test_compiler(
        #[case] input: &'static str,
        #[case] expected_constants: Vec<Object>,
        #[case] expected_instructions: Vec<Instructions>,
    ) {
        let mut interner = Interner::new();

        let program = Parser::build_ast(&input, &mut interner);
        let mut compiler = Compiler::new();
        compiler.compile(&program.statements);

        let bytecode = compiler.to_bytecode();

        assert_eq!(&expected_instructions.concat(), &bytecode.instructions);
        assert_eq!(&expected_constants, &bytecode.constants);
    }
}
