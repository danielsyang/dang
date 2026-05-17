use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::eval::{env::Environment, eval_block, object::Object};

use super::expression::Expression;

pub type Block = Vec<Statement>;
pub type Identifier = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Statement {
    Let(Identifier, Expression),
    Assignment(Identifier, Expression),
    Return(Expression),
    Expression(Expression),
    Error(String),
    While { condition: Expression, body: Block },
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Let(identifier, exp) => write!(f, "Let {} {}", identifier, exp),
            Statement::Assignment(identifier, exp) => write!(f, "= {} {}", identifier, exp),
            Statement::Return(exp) => {
                write!(f, "Return {}", exp)
            }
            Statement::Expression(exp) => {
                write!(f, "{}", exp)
            }
            Statement::Error(s) => write!(f, "error: ( {} )", s),
            Statement::While { condition, body } => {
                write!(f, "while ( {} ) {{ {:?} }}", condition, body)
            }
        }
    }
}

impl Statement {
    pub fn eval(&self, env: &Rc<RefCell<Environment>>) -> Object {
        match self {
            Statement::Expression(exp) => exp.eval(env),
            Statement::Return(r) => {
                let result = r.eval(env);
                Object::Return(Box::new(result))
            }
            Statement::Let(ident, exp) => {
                let val = exp.eval(env);
                env.borrow_mut().set(ident.clone(), val.clone());

                val
            }
            Statement::Assignment(ident, exp) => {
                let val = exp.eval(env);

                let existing = env.borrow().get(ident.clone());

                match existing {
                    Some(_) => {
                        env.borrow_mut().set(ident.clone(), val.clone());
                        val
                    }
                    None => Object::Error(format!("Identifier not found: {}", ident)),
                }
            }
            Statement::Error(s) => Object::Error(s.clone()),
            Statement::While { condition, body } => {
                loop {
                    let assertion = condition.eval(env);

                    match assertion {
                        Object::Boolean(true) => {
                            eval_block(body, env);
                        }
                        _ => break,
                    }
                }

                Object::None
            }
        }
    }
}
