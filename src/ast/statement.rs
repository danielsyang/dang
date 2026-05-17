use std::{cell::RefCell, rc::Rc};

use crate::{
    eval::{env::Environment, eval_block, object::Object},
    intern::interner::{Interner, PrettyDisplay, Symbol, WithInterner},
};

use super::expression::Expression;

pub type Block = Vec<Statement>;
pub type Identifier = Symbol;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Statement {
    Let(Identifier, Expression),
    Assignment(Identifier, Expression),
    Return(Expression),
    Expression(Expression),
    Error(String),
    While { condition: Expression, body: Block },
}

impl PrettyDisplay for Statement {
    fn pretty(&self, f: &mut std::fmt::Formatter, interner: &Interner) -> std::fmt::Result {
        match self {
            Statement::Let(identifier, exp) => {
                write!(
                    f,
                    "Let {} {}",
                    interner.resolve(*identifier),
                    WithInterner {
                        value: exp,
                        interner
                    }
                )
            }
            Statement::Assignment(identifier, exp) => {
                write!(
                    f,
                    "= {} {}",
                    interner.resolve(*identifier),
                    WithInterner {
                        value: exp,
                        interner
                    }
                )
            }
            Statement::Return(exp) => {
                write!(
                    f,
                    "Return {}",
                    WithInterner {
                        value: exp,
                        interner
                    }
                )
            }
            Statement::Expression(exp) => {
                write!(
                    f,
                    "{}",
                    WithInterner {
                        value: exp,
                        interner
                    }
                )
            }
            Statement::Error(s) => write!(f, "error: ( {} )", s),
            Statement::While { condition, body } => {
                let body_str = body
                    .iter()
                    .map(|s| WithInterner { value: s, interner }.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(
                    f,
                    "while ( {} ) {{ [{}] }}",
                    WithInterner {
                        value: condition,
                        interner
                    },
                    body_str
                )
            }
        }
    }
}

impl Statement {
    pub fn eval(&self, env: &Rc<RefCell<Environment>>, interner: &Interner) -> Object {
        match self {
            Statement::Expression(exp) => exp.eval(env, interner),
            Statement::Return(r) => {
                let result = r.eval(env, interner);
                Object::Return(Box::new(result))
            }
            Statement::Let(ident, exp) => {
                let val = exp.eval(env, interner);
                env.borrow_mut().set(*ident, val.clone());

                val
            }
            Statement::Assignment(ident, exp) => {
                let val = exp.eval(env, interner);

                let existing = env.borrow().get(*ident);

                match existing {
                    Some(_) => {
                        env.borrow_mut().set(*ident, val.clone());
                        val
                    }
                    None => Object::Error(format!(
                        "Identifier not found: {}",
                        interner.resolve(*ident)
                    )),
                }
            }
            Statement::Error(s) => Object::Error(s.clone()),
            Statement::While { condition, body } => {
                loop {
                    let assertion = condition.eval(env, interner);

                    match assertion {
                        Object::Boolean(true) => {
                            eval_block(body, env, interner);
                        }
                        _ => break,
                    }
                }

                Object::None
            }
        }
    }
}
