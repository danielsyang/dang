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
                    self.emit(Opcode::OpPop, &[(self.instructions.len() - 1) as u16]);
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
            Expression::Infix(op, left_expr, right_expr) => {
                self.compile_expression(left_expr);
                self.compile_expression(right_expr);

                match op {
                    Operator::Plus => self.emit(Opcode::OpAdd, &[]),
                    Operator::Minus => self.emit(Opcode::OpSub, &[]),
                    Operator::Multiply => self.emit(Opcode::OpMul, &[]),
                    Operator::Divide => self.emit(Opcode::OpDiv, &[]),
                    _ => todo!("{} operator not implement", op),
                };
            }
            Expression::Literal(Literal::Number(val)) => {
                let position = self.add_constant(*val);
                self.emit(Opcode::Constant, &[position as u16]);
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
    use crate::{
        ast::parser::Parser,
        compiler::{
            code::{Instructions, Opcode},
            compiler::Compiler,
        },
        eval::object::Object,
        intern::interner::Interner,
    };

    struct CompilerTestCase {
        input: &'static str,
        expected_constants: Vec<Object>,
        expected_instructions: Vec<Instructions>,
    }

    #[test]
    fn test_integer_arithmatic() {
        let tests: Vec<CompilerTestCase> = vec![CompilerTestCase {
            input: "1 + 2",
            expected_constants: vec![Object::Number(1), Object::Number(2)],
            expected_instructions: vec![
                Opcode::Constant.make(&[0]),
                Opcode::Constant.make(&[1]),
                Opcode::OpAdd.make(&[]),
                Opcode::OpPop.make(&[]),
            ],
        }];

        run_compiler_tests(tests);
    }

    fn run_compiler_tests(tests: Vec<CompilerTestCase>) {
        for tt in tests {
            let mut interner = Interner::new();

            let program = Parser::build_ast(&tt.input, &mut interner);
            let mut compiler = Compiler::new();
            compiler.compile(&program);

            let bytecode = compiler.to_bytecode();

            assert_eq!(&tt.expected_instructions.concat(), &bytecode.instructions);
            assert_eq!(&tt.expected_constants, &bytecode.constants);
        }
    }
}
