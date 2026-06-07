use crate::{
    compiler::{
        code::{Instructions, Opcode},
        compiler::Bytecode,
    },
    eval::object::Object,
};

#[derive(Debug)]
pub enum VmError {
    StackOverflow,
    InvalidOperation,
    UnexpectedError,
}

pub struct VM<'a> {
    constants: &'a Vec<Object>,
    instructions: &'a Instructions,

    stack: Vec<Object>,
    last_popped_stack_elem: Option<Object>,
}

const MAX_STACK_SIZE: usize = 2048;

impl<'a> VM<'a> {
    pub fn new(bytecode: &'a Bytecode) -> Self {
        Self {
            constants: &bytecode.constants,
            instructions: &bytecode.instructions,

            stack: vec![],
            last_popped_stack_elem: None,
        }
    }

    pub fn stack_top(&self) -> &Object {
        match self.stack.last() {
            None => &Object::None,
            Some(val) => val,
        }
    }

    pub fn run(&mut self) {
        let mut ip = 0;

        while ip < self.instructions.len() {
            let position = self
                .instructions
                .get(ip)
                .unwrap_or_else(|| panic!("Something went wrong with self.instructions"));

            ip += 1;

            let operation = Opcode::try_from(*position);

            match operation {
                Ok(opcode) => {
                    ip = self
                        .execute_operation(opcode, ip)
                        .unwrap_or_else(|err| panic!("Invalid operation {:?}", err));
                }
                Err(err) => {
                    panic!("Operation not implemented yet: {}", err)
                }
            };
        }
    }

    pub fn last_popped_element(&self) -> &Object {
        match &self.last_popped_stack_elem {
            Some(elem) => elem,
            None => &Object::None,
        }
    }

    fn push(&mut self, obj: Object) -> Result<(), VmError> {
        if self.stack.len() >= MAX_STACK_SIZE {
            return Err(VmError::StackOverflow);
        }

        self.stack.push(obj);

        return Ok(());
    }

    fn pop(&mut self) -> Object {
        match self.stack.pop() {
            Some(val) => {
                self.last_popped_stack_elem = Some(val.clone());

                val
            }
            None => panic!("Nothing to pop, something went wrong"),
        }
    }

    fn execute_operation(&mut self, opcode: Opcode, mut ip: usize) -> Result<usize, VmError> {
        let mut err: Option<VmError> = None;

        match opcode {
            Opcode::Constant => {
                let const_index =
                    u16::from_be_bytes([self.instructions[ip], self.instructions[ip + 1]]);
                ip += 2;

                match self.constants.get(const_index as usize) {
                    Some(val) => self.push(val.clone())?,
                    None => {
                        err = Some(VmError::UnexpectedError);
                    }
                }
            }
            Opcode::OpAdd => self.execute_binary_operation(|x, y| x + y)?,
            Opcode::OpSub => self.execute_binary_operation(|x, y| x - y)?,
            Opcode::OpMul => self.execute_binary_operation(|x, y| x * y)?,
            Opcode::OpDiv => self.execute_binary_operation(|x, y| x / y)?,
            Opcode::OpPop => {
                self.pop();
            }
        };

        match err {
            Some(err) => Err(err),
            None => Ok(ip),
        }
    }

    fn execute_binary_operation(&mut self, op: impl Fn(i64, i64) -> i64) -> Result<(), VmError> {
        let right = self.pop();
        let left = self.pop();

        match (left, right) {
            (Object::Number(left_val), Object::Number(right_val)) => {
                let result = op(left_val, right_val);
                self.push(Object::Number(result))
            }
            _ => Err(VmError::InvalidOperation),
        }
    }
}

#[cfg(test)]
mod test {
    use super::VM;
    use crate::{
        ast::parser::Parser, compiler::compiler::Compiler, eval::object::Object,
        intern::interner::Interner,
    };
    use rstest::rstest;

    #[rstest]
    #[case::base("1", Object::Number(1))]
    #[case::addition("1 + 2", Object::Number(3))]
    #[case::subtraction("5 - 1", Object::Number(4))]
    #[case::multiplication("4 * 4", Object::Number(16))]
    #[case::division("8 / 4", Object::Number(2))]
    #[case::all_in_one("50 / 2 * 2 + 10 - 5", Object::Number(55))]
    #[case::parenthesis("5 * (2 + 10)", Object::Number(60))]
    fn test_integer_arithmetic(#[case] input: &str, #[case] expected: Object) {
        let mut interner = Interner::new();

        let program = Parser::build_ast(&input, &mut interner);
        let mut compiler = Compiler::new();
        compiler.compile(&program);

        let bytecode = compiler.to_bytecode();

        let mut vm = VM::new(&bytecode);
        vm.run();

        assert_eq!(vm.last_popped_element(), &expected);
    }
}
