use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    fmt::Display,
    hash::Hash,
    rc::Rc,
};

use crate::eval::{
    env::Environment,
    eval_block, eval_function_block,
    object::{CustomHash, HashKey, Object},
};

type Elements = Vec<Expression>;

use super::{
    literal::Literal,
    statement::{Block, Identifier},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    And,
    Or,
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Plus => write!(f, "+"),
            Operator::Minus => write!(f, "-"),
            Operator::Multiply => write!(f, "*"),
            Operator::Divide => write!(f, "/"),
            Operator::Equal => write!(f, "=="),
            Operator::NotEqual => write!(f, "!="),
            Operator::GreaterThan => write!(f, ">"),
            Operator::LessThan => write!(f, "<"),
            Operator::GreaterThanOrEqual => write!(f, ">="),
            Operator::LessThanOrEqual => write!(f, "<="),
            Operator::And => write!(f, "&&"),
            Operator::Or => write!(f, "||"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Prefix {
    Bang,
    Minus,
}

impl Display for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Prefix::Bang => write!(f, "!"),
            Prefix::Minus => write!(f, "-"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Expression {
    Error(String),
    Literal(Literal),
    Identifier(Identifier),
    Infix(Operator, Box<Expression>, Box<Expression>),
    Prefix(Prefix, Box<Expression>),
    If {
        condition: Box<Expression>,
        consequence: Block,
        alternative: Option<Block>,
    },
    Function {
        identifier: Option<Identifier>,
        parameters: Vec<Identifier>,
        body: Block,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Array(Elements),
    Index {
        left: Box<Expression>,
        index: Box<Expression>,
    },
    HashMap {
        pairs: BTreeMap<Expression, Expression>,
    },
    Dot {
        identifier: Box<Expression>,
        attribute: Identifier,
    },
}

impl Expression {
    pub fn eval(&self, env: &Rc<RefCell<Environment>>) -> Object {
        match self {
            Expression::Error(s) => Object::Error(s.clone()),
            Expression::Literal(l) => l.eval(),
            Expression::Prefix(op, exp) => {
                let right_exp = exp.eval(env);

                match op {
                    Prefix::Bang => match right_exp {
                        Object::Boolean(b) => Object::Boolean(!b),
                        _ => Object::Error(format!("expected Boolean, got: {}", right_exp)),
                    },
                    Prefix::Minus => match right_exp {
                        Object::Number(n) => Object::Number(-n),
                        _ => Object::Error(format!("expected Number, got: {}", right_exp)),
                    },
                }
            }
            Expression::Infix(op, left_exp, right_exp) => {
                let mut left = left_exp.eval(env);
                let mut right = right_exp.eval(env);

                loop {
                    match (&left, &right) {
                        (Object::Return(l), _) => {
                            left = l.as_ref().clone();
                        }
                        (_, Object::Return(r)) => {
                            right = r.as_ref().clone();
                        }
                        _ => break,
                    };
                }

                match (op, &left, &right) {
                    (Operator::Plus, _, _) => match (&left, &right) {
                        (Object::Number(l), Object::Number(r)) => Object::Number(l + r),
                        _ => Object::Error(format!(
                            "Can only perform operation + on numbers, got: {} and {} ",
                            &left, &right,
                        )),
                    },
                    (Operator::Minus, _, _) => match (&left, &right) {
                        (Object::Number(l), Object::Number(r)) => Object::Number(l - r),
                        _ => Object::Error(format!(
                            "Can only perform operation {} on numbers, got: {} and {} ",
                            op, &left, &right,
                        )),
                    },

                    (Operator::Multiply, _, _) => match (&left, &right) {
                        (Object::Number(l), Object::Number(r)) => Object::Number(l * r),
                        _ => Object::Error(format!(
                            "Can only perform operation {} on numbers, got: {} and {} ",
                            op, left, right,
                        )),
                    },

                    (Operator::Divide, _, _) => match (&left, &right) {
                        (Object::Number(l), Object::Number(r)) => Object::Number(l / r),
                        _ => Object::Error(format!(
                            "Can only perform operation {} on numbers, got: {} and {} ",
                            op, left, right,
                        )),
                    },

                    (Operator::GreaterThan, _, _) => match (&left, &right) {
                        (Object::Number(l), Object::Number(r)) => Object::Boolean(l > r),
                        _ => Object::Error(format!(
                            "Can only perform operation {} on numbers, got: {} and {} ",
                            op, left, right,
                        )),
                    },

                    (Operator::LessThan, _, _) => match (&left, &right) {
                        (Object::Number(l), Object::Number(r)) => Object::Boolean(l < r),
                        _ => Object::Error(format!(
                            "Can only perform operation {} on numbers, got: {} and {} ",
                            op, left, right,
                        )),
                    },

                    (Operator::GreaterThanOrEqual, _, _) => match (&left, &right) {
                        (Object::Number(l), Object::Number(r)) => Object::Boolean(l >= r),
                        _ => Object::Error(format!(
                            "Can only perform operation {} on numbers, got: {} and {} ",
                            op, left, right,
                        )),
                    },

                    (Operator::LessThanOrEqual, _, _) => match (&left, &right) {
                        (Object::Number(l), Object::Number(r)) => Object::Boolean(l <= r),
                        _ => Object::Error(format!(
                            "Can only perform operation {} on numbers, got: {} and {} ",
                            op, left, right,
                        )),
                    },

                    (Operator::Equal, _, _) => match (&left, &right) {
                        (Object::Number(l), Object::Number(r)) => Object::Boolean(l == r),
                        (Object::Boolean(l), Object::Boolean(r)) => Object::Boolean(l == r),
                        _ => Object::Error(format!(
                            "Can only perform operation {} on (numbers | boolean), got: {} and {} ",
                            op, left, right,
                        )),
                    },
                    (Operator::NotEqual, _, _) => match (&left, &right) {
                        (Object::Number(l), Object::Number(r)) => Object::Boolean(l != r),
                        (Object::Boolean(l), Object::Boolean(r)) => Object::Boolean(l != r),
                        _ => Object::Error(format!(
                            "Can only perform operation {} on (numbers | boolean), got: {} and {} ",
                            op, left, right,
                        )),
                    },
                    (Operator::And, _, _) => match (&left, &right) {
                        (Object::Boolean(l), Object::Boolean(r)) => Object::Boolean(*l && *r),
                        _ => Object::Error(format!(
                            "Can only perform operation {} on (numbers | boolean), got: {} and {} ",
                            op, left, right,
                        )),
                    },
                    (Operator::Or, _, _) => match (&left, &right) {
                        (Object::Boolean(l), Object::Boolean(r)) => Object::Boolean(*l || *r),
                        _ => Object::Error(format!(
                            "Can only perform operation {} on (numbers | boolean), got: {} and {} ",
                            op, left, right,
                        )),
                    },
                }
            }
            Expression::If {
                condition,
                consequence,
                alternative,
            } => {
                let condition_result = condition.eval(env);
                match (condition_result, alternative) {
                    (Object::Boolean(true), _) => eval_block(consequence, env),
                    (Object::Boolean(false), Some(alt)) => eval_block(alt, env),
                    (Object::Boolean(false), None) => Object::None,
                    (_, _) => Object::Error(String::from("condition did not evaluate to boolean")),
                }
            }
            Expression::Identifier(ident) => match &env.borrow_mut().get(ident.clone()) {
                Some(obj) => obj.clone(),
                None => Object::Error(format!("identifier not found: {}", ident)),
            },
            Expression::Function {
                identifier,
                parameters,
                body,
            } => {
                let fun = match identifier {
                    Some(i) => Object::Function {
                        name: Some(i.clone()),
                        parameters: parameters.to_vec().clone(),
                        body: body.to_vec().clone(),
                        env: env.clone(),
                    },
                    None => Object::Function {
                        name: None,
                        parameters: parameters.to_vec().clone(),
                        body: body.to_vec().clone(),
                        env: env.clone(),
                    },
                };

                if let Some(i) = identifier {
                    env.borrow_mut().set(i.clone(), fun.clone())
                }

                fun
            }
            Expression::Call {
                function,
                arguments,
            } => {
                let func = function.eval(env);
                let args = arguments
                    .iter()
                    .map(|arg| arg.eval(env))
                    .collect::<Vec<_>>();

                match (func, &args) {
                    (
                        Object::Function {
                            name: _name,
                            parameters,
                            body,
                            env: current_env,
                        },
                        _,
                    ) => {
                        let next_env = Environment::new_enclosed(current_env.clone());
                        let mut has_error = false;
                        let mut error_idx = 0;
                        for (idx, param) in parameters.iter().enumerate() {
                            if let Some(arg) = args.get(idx) {
                                next_env.borrow_mut().set(param.clone(), arg.clone());
                            } else {
                                error_idx = idx;
                                has_error = true;
                                break;
                            }
                        }

                        if has_error {
                            return Object::Error(format!("Missing parameter: {}", error_idx));
                        }

                        match eval_function_block(&body, &next_env) {
                            Some(r) => r,
                            None => Object::None,
                        }
                    }
                    (Object::Builtin { func }, _) => func(args),
                    (_, _) => Object::Error(format!("not a valid call {} ", self)),
                }
            }
            Expression::Array(elements) => {
                let arr = elements
                    .iter()
                    .map(|el| el.eval(env))
                    .collect::<Vec<Object>>();

                Object::Array(arr)
            }
            Expression::Index { index, left } => {
                let left_exp = left.eval(env);
                let index_exp = index.eval(env);

                match (&left_exp, &index_exp) {
                    (Object::Array(arr), Object::Array(index)) => {
                        if index.len() != 1 {
                            return Object::Error(format!("invalid index, got {:?}", index));
                        }

                        match index.first().unwrap() {
                            Object::Number(n) => match arr.get(*n as usize) {
                                Some(obj) => obj.clone(),
                                None => Object::None,
                            },
                            _ => Object::Error(format!("invalid index, got {:?}", index)),
                        }
                    }
                    (Object::HashMap { pairs }, Object::Array(index)) => {
                        if index.len() != 1 {
                            return Object::Error(format!("invalid index, got {:?}", index));
                        }

                        match index.first().unwrap().hash() {
                            Some(hk) => match pairs.get(&hk) {
                                Some(v) => v.clone(),
                                None => Object::None,
                            },
                            None => {
                                Object::Error(format!("Index is not hashable, got {:?}", index))
                            }
                        }
                    }
                    _ => Object::Error(format!(
                        "not supported, got: {:?}, {:?}",
                        left_exp, index_exp
                    )),
                }
            }
            Expression::HashMap { pairs } => {
                let mut hm: HashMap<HashKey, Object> = HashMap::new();

                for (k, v) in pairs {
                    let key_obj = k.eval(env);

                    if key_obj.hash().is_none() {
                        return Object::Error(format!("Key is not hashable, got {}", key_obj));
                    }

                    let key = key_obj.hash().unwrap();
                    let val = v.eval(env);

                    hm.insert(key, val);
                }

                Object::HashMap { pairs: hm }
            }
            Expression::Dot {
                identifier,
                attribute,
            } => {
                // For now "dot" operations only works on hashMaps
                let hashmap = identifier.eval(env);

                match hashmap {
                    Object::HashMap { pairs } => {
                        match pairs.get(&HashKey::new(attribute.clone())) {
                            Some(v) => v.clone(),
                            None => Object::None,
                        }
                    }
                    _ => Object::Error(format!(
                        "Cannot read {:?} propertie of {}",
                        attribute, identifier
                    )),
                }
            }
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Error(s) => write!(f, "error: ( {} ) ", s),
            Expression::Literal(Literal::Number(v)) => write!(f, "Number ({})", v),
            Expression::Literal(Literal::String(s)) => write!(f, "String ({})", s),
            Expression::Literal(Literal::Boolean(b)) => write!(f, "Bool ({})", b),
            Expression::Identifier(i) => write!(f, "Ident ({})", i),
            Expression::Infix(op, left, right) => {
                write!(f, "{} Left {} , Right {}", op, left, right)
            }
            Expression::Prefix(pr, exp) => write!(f, "{} {}", pr, exp),
            Expression::If {
                condition,
                consequence,
                alternative,
            } => {
                let consequence_block = consequence
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                match alternative {
                    Some(alt) => {
                        let alt_block = alt
                            .iter()
                            .map(|c| c.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        write!(
                            f,
                            "If {} {{ {} }} else {}",
                            condition, consequence_block, alt_block
                        )
                    }
                    None => {
                        write!(f, "If {} {{ {} }}", condition, consequence_block)
                    }
                }
            }
            Expression::Function {
                identifier,
                parameters,
                body,
            } => match identifier {
                Some(i) => write!(
                    f,
                    "Fn {} ( {} ) {}",
                    i.clone(),
                    parameters.join(", "),
                    body.iter()
                        .map(|b| b.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                None => write!(
                    f,
                    "Fn ( {} ) {}",
                    parameters.join(", "),
                    body.iter()
                        .map(|b| b.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            },

            Expression::Call {
                function,
                arguments,
            } => write!(
                f,
                "Call {} , {}",
                function,
                arguments
                    .iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),

            Expression::Array(elements) => write!(
                f,
                "[ {} ]",
                elements
                    .iter()
                    .map(|el| el.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),

            Expression::Index { index, left } => {
                write!(f, "({} [{}])", left, index)
            }

            Expression::HashMap { pairs } => {
                let expr = pairs
                    .iter()
                    .map(|(k, v)| format!("{} : {}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "{{ {} }}", expr)
            }
            Expression::Dot {
                identifier,
                attribute,
            } => {
                write!(f, "{} of {}", attribute, identifier)
            }
        }
    }
}
