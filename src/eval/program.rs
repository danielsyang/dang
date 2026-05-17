use std::{cell::RefCell, rc::Rc};

use crate::ast::statement::Statement;

use super::{env::Environment, object::Object};

pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn eval_statements(&self, env: &Rc<RefCell<Environment>>) -> Object {
        let mut result = Object::None;
        for stmt in self.statements.iter() {
            result = stmt.eval(env);

            if let Object::Return(_) = result {
                break;
            }
        }

        result
    }
}
