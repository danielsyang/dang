use std::{any::Any, fmt::Debug};

use crate::eval::{
    environment::Environment,
    object::{None, Object, RETURN_OBJ},
};

pub trait Node {
    fn token_literal(&self) -> String;
    fn string(&self) -> String;

    fn eval_node(&self, env: &mut Environment) -> Box<dyn Object>;
}
// TOOD: Remove downcasting
pub trait Statement: Node + AToAny {
    fn statement_node(&self);

    fn clone_statement(&self) -> Box<dyn Statement>;
}

impl Debug for dyn Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string())
    }
}

pub trait Expression: Node {
    fn expression_node(&self);

    fn eval_expression(&self, env: &mut Environment) -> Box<dyn Object>;

    fn clone_expression(&self) -> Box<dyn Expression>;
}

impl Debug for dyn Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string())
    }
}

pub struct Program {
    pub statements: Vec<Box<dyn Statement>>,
}

impl Program {
    pub fn eval_statements(&self, env: &mut Environment) -> Box<dyn Object> {
        let mut result: Box<dyn Object> = Box::new(None::new());
        for stmt in self.statements.iter() {
            result = stmt.eval_node(env);

            if result.kind() == RETURN_OBJ {
                break;
            }
        }

        result
    }
}

pub trait AToAny: 'static {
    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static> AToAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
