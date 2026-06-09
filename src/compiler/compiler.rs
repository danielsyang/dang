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
    eval::{object::Object, program::Program},
};

pub struct Compiler {
    instructions: Instructions,
    constants: Vec<Object>,
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
        }
    }

    pub fn compile(&mut self, program: &Program) {
        for sttm in &program.statements {
            match sttm {
                Statement::Expression(expr) => {
                    self.compile_expression(expr);
                    self.emit(Opcode::OpPop, &[]);
                }
                _ => todo!("TODO: not implemented."),
            }
        }
    }

    fn _compile_statement(&self, _stmt: &Statement) {
        todo!("TODO: not implemented.")
    }

    fn compile_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Infix(Operator::LessThan, left_expr, right_expr) => {
                self.compile_expression(right_expr);
                self.compile_expression(left_expr);
                self.emit(Opcode::OpGreaterThan, &[])
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
                true => self.emit(Opcode::OpTrue, &[]),
                false => self.emit(Opcode::OpFalse, &[]),
            },
            _ => {
                todo!("TODO: not implemented.")
            }
        }
    }

    fn add_constant(&mut self, val: i64) -> usize {
        self.constants.push(Object::Number(val.clone()));

        self.constants.len() - 1
    }

    fn emit(&mut self, op_code: Opcode, position: &[u16]) {
        let val = op_code.make(position);

        for v in val {
            self.instructions.push(v);
        }
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
            code::{Instructions, Opcode},
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
    fn test_compiler(
        #[case] input: &'static str,
        #[case] expected_constants: Vec<Object>,
        #[case] expected_instructions: Vec<Instructions>,
    ) {
        let mut interner = Interner::new();

        let program = Parser::build_ast(&input, &mut interner);
        let mut compiler = Compiler::new();
        compiler.compile(&program);

        let bytecode = compiler.to_bytecode();

        assert_eq!(&expected_instructions.concat(), &bytecode.instructions);
        assert_eq!(&expected_constants, &bytecode.constants);
    }
}
