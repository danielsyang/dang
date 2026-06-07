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
}

pub struct VM<'a> {
    constants: &'a Vec<Object>,
    instructions: &'a Instructions,

    stack: Vec<Object>,
    sp: usize,
}

const MAX_STACK_SIZE: usize = 2048;

impl<'a> VM<'a> {
    pub fn new(bytecode: &'a Bytecode) -> Self {
        Self {
            constants: &bytecode.constants,
            instructions: &bytecode.instructions,

            stack: vec![],
            sp: 0,
        }
    }

    pub fn stack_top(&self) -> &Object {
        match self.sp {
            0 => &Object::None,
            _ => {
                let val = self.stack.get(self.sp - 1);
                match val {
                    None => {
                        panic!("Not sure what to do, panicking for now.")
                    }
                    Some(v) => v,
                }
            }
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
                Ok(opcode) => match opcode {
                    Opcode::OpAdd => {
                        let right = self.pop();
                        let left = self.pop();

                        match (left, right) {
                            (Object::Number(left_val), Object::Number(right_val)) => {
                                let result = left_val + right_val;

                                match self.push(Object::Number(result)) {
                                    Ok(_) => {}
                                    Err(err) => {
                                        panic!("Something went wrong {:?}", err)
                                    }
                                };
                            }
                            _ => {
                                todo!("To be implemented")
                            }
                        }

                        ip += 1;
                    }
                    Opcode::Constant => {
                        let const_index =
                            u16::from_be_bytes([self.instructions[ip], self.instructions[ip + 1]]);
                        ip += 2;

                        match self.constants.get(const_index as usize) {
                            Some(val) => match self.push(val.clone()) {
                                Ok(_) => {}
                                Err(err) => {
                                    println!("Got an error: {:?}", err)
                                }
                            },
                            None => {
                                panic!("something went wrong, const_index: {}", const_index)
                            }
                        }
                    }
                },
                Err(err) => {
                    panic!("Operation not implemented yet: {}", err)
                }
            };
        }
    }

    fn push(&mut self, obj: Object) -> Result<(), VmError> {
        if self.sp >= MAX_STACK_SIZE {
            return Err(VmError::StackOverflow);
        }

        self.stack.push(obj.clone());
        self.sp += 1;

        return Ok(());
    }

    fn pop(&mut self) -> Object {
        match self.stack.pop() {
            Some(val) => {
                self.sp -= 1;

                val
            }
            None => panic!("Nothing to pop, something went wrong"),
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

    struct VmTestCase {
        input: &'static str,
        expected: Object,
    }

    #[test]
    fn test_integer_arithmetic() {
        let tests: Vec<VmTestCase> = vec![
            VmTestCase {
                input: "1",
                expected: Object::Number(1),
            },
            VmTestCase {
                input: "2",
                expected: Object::Number(2),
            },
            VmTestCase {
                input: "1 + 2",
                expected: Object::Number(3),
            },
        ];

        run_vm_tests(tests);
    }

    fn run_vm_tests(tests: Vec<VmTestCase>) {
        for tt in tests {
            let mut interner = Interner::new();

            let program = Parser::build_ast(&tt.input, &mut interner);
            let mut compiler = Compiler::new();
            compiler.compile(&program);

            let bytecode = compiler.to_bytecode();

            let mut vm = VM::new(&bytecode);
            vm.run();

            assert_eq!(vm.stack_top(), &tt.expected);
        }
    }
}
