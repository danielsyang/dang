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

    // For testing
    #[allow(dead_code)]
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

        Ok(())
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
        let definition = opcode.lookup();

        match opcode {
            Opcode::Constant => {
                let (operands, bytes_read) =
                    Opcode::read_operands(&definition, &self.instructions[ip..]);

                ip += bytes_read;

                let const_index = operands[0] as usize;
                match self.constants.get(const_index) {
                    Some(val) => self.push(val.clone())?,
                    None => return Err(VmError::UnexpectedError),
                }
            }
            Opcode::OpAdd => self.execute_binary_operation(|x, y| x + y)?,
            Opcode::OpSub => self.execute_binary_operation(|x, y| x - y)?,
            Opcode::OpMul => self.execute_binary_operation(|x, y| x * y)?,
            Opcode::OpDiv => self.execute_binary_operation(|x, y| x / y)?,
            Opcode::OpEqual => self.execute_equal_not_equal_operator(|x, y| x == y)?,
            Opcode::OpNotEqual => self.execute_equal_not_equal_operator(|x, y| x != y)?,
            Opcode::OpGreaterThan => self.execute_greater_operator()?,
            Opcode::OpPop => {
                self.pop();
            }
            Opcode::OpMinus => self.execute_minus_operator()?,
            Opcode::OpTrue => self.push(Object::Boolean(true))?,
            Opcode::OpFalse => self.push(Object::Boolean(false))?,
            Opcode::OpJump => {
                let (operands, _) = Opcode::read_operands(&definition, &self.instructions[ip..]);
                ip = operands[0] as usize;
            }
            Opcode::OpJumpNotTruthy => {
                let (operands, bytes_read) =
                    Opcode::read_operands(&definition, &self.instructions[ip..]);

                ip += bytes_read;

                if !self.pop_check_if_truthy() {
                    ip = operands[0] as usize
                }
            }
            Opcode::OpNone => self.push(Object::None)?,
            Opcode::OpBang => self.execute_bang_operator()?,
        };

        Ok(ip)
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

    fn execute_equal_not_equal_operator(
        &mut self,
        op: impl Fn(Object, Object) -> bool,
    ) -> Result<(), VmError> {
        let right = self.pop();
        let left = self.pop();
        let result = op(left, right);

        self.push(Object::Boolean(result))
    }

    fn execute_greater_operator(&mut self) -> Result<(), VmError> {
        let right = self.pop();
        let left = self.pop();

        match (left, right) {
            (Object::Number(left_val), Object::Number(right_val)) => {
                self.push(Object::Boolean(left_val > right_val))
            }
            _ => Err(VmError::InvalidOperation),
        }
    }

    fn execute_bang_operator(&mut self) -> Result<(), VmError> {
        let obj = self.pop();
        match obj {
            Object::Boolean(v) => self.push(Object::Boolean(!v)),
            Object::None => self.push(Object::Boolean(true)),
            _ => Err(VmError::InvalidOperation),
        }
    }

    fn execute_minus_operator(&mut self) -> Result<(), VmError> {
        let obj = self.pop();

        match obj {
            Object::Number(v) => self.push(Object::Number(-v)),
            _ => Err(VmError::InvalidOperation),
        }
    }

    fn pop_check_if_truthy(&mut self) -> bool {
        let condition = self.pop();

        matches!(condition, Object::Boolean(true))
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
    #[case::true_val("true", Object::Boolean(true))]
    #[case::false_val("false", Object::Boolean(false))]
    #[case::bang_false_val("!true", Object::Boolean(false))]
    #[case::bang_true_val("!false", Object::Boolean(true))]
    #[case::minus_operation("-5 + -5", Object::Number(-10))]
    #[case::eq_true_integer("1 == 1", Object::Boolean(true))]
    #[case::eq_false_integer("2 == 1", Object::Boolean(false))]
    #[case::eq_true_boolean("true == true", Object::Boolean(true))]
    #[case::eq_false_boolean("false == true", Object::Boolean(false))]
    #[case::not_eq_true_integer("2 != 1", Object::Boolean(true))]
    #[case::not_eq_false_integer("2 != 2", Object::Boolean(false))]
    #[case::not_eq_true_boolean("true != false", Object::Boolean(true))]
    #[case::not_eq_false_boolean("false != false", Object::Boolean(false))]
    #[case::greater_true_integer("2 > 1", Object::Boolean(true))]
    #[case::greater_false_integer("1 > 2", Object::Boolean(false))]
    #[case::greater_eq_true_integer("(2 > 1) == true", Object::Boolean(true))]
    #[case::greater_eq_false_integer("(1 > 2) == true", Object::Boolean(false))]
    #[case::less_true_integer("1 < 2", Object::Boolean(true))]
    #[case::less_false_integer("2 < 1", Object::Boolean(false))]
    #[case::less_eq_true_integer("(1 < 2) == true", Object::Boolean(true))]
    #[case::less_eq_false_integer("(2 < 1) == true", Object::Boolean(false))]
    #[case::if_expression("if (true) { 10 }", Object::Number(10))]
    #[case::if_else_expression("if (true) { 10 } else { 20 }", Object::Number(10))]
    #[case::else_expression("if (false) { 10 } else { 20 }", Object::Number(20))]
    #[case::if_false_expression("if (false) { 10 }", Object::None)]
    #[case::if_none_expression("!(if (false) { 5; })", Object::Boolean(true))]
    fn test_vm(#[case] input: &str, #[case] expected: Object) {
        let mut interner = Interner::new();

        let program = Parser::build_ast(&input, &mut interner);
        let mut compiler = Compiler::new();
        compiler.compile(&program.statements);

        let bytecode = compiler.to_bytecode();

        let mut vm = VM::new(&bytecode);
        vm.run();

        assert_eq!(vm.last_popped_element(), &expected);
    }
}
