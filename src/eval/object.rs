use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    ast::statement::{Block, Identifier},
    eval::env::Environment,
    intern::interner::{Interner, PrettyDisplay, WithInterner},
};

type BuiltinFunction = fn(Vec<Object>, &Interner) -> Object;
type Elements = Vec<Object>;

// Mute "warning: function pointer comparisons do not produce meaningful results since their addresses are not guaranteed to be unique" for now
// Builtins are never compared by value; the derived PartialEq only compares fn pointers here.
#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    None,
    Number(i64),
    String(String),
    Boolean(bool),
    Return(Box<Object>),
    Error(String),
    Function {
        name: Option<Identifier>,
        parameters: Vec<Identifier>,
        body: Block,
        env: Rc<RefCell<Environment>>,
    },
    Array(Elements),
    Builtin {
        func: BuiltinFunction,
    },
    HashMap {
        pairs: HashMap<HashKey, Object>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HashKey {
    value: String,
}

impl HashKey {
    pub fn new(s: String) -> Self {
        Self { value: s }
    }
}

impl Display for HashKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

pub trait CustomHash {
    fn hash(&self) -> Option<HashKey>;
}

impl CustomHash for Object {
    fn hash(&self) -> Option<HashKey> {
        match self {
            Object::Boolean(b) => Some(HashKey::new(if *b {
                String::from("true")
            } else {
                String::from("false")
            })),
            Object::Number(n) => Some(HashKey::new(format!("{}", n))),
            Object::String(s) => Some(HashKey::new(s.clone())),
            _ => None,
        }
    }
}

impl PrettyDisplay for Object {
    fn pretty(&self, f: &mut std::fmt::Formatter, interner: &Interner) -> std::fmt::Result {
        match self {
            Object::None => write!(f, "None"),
            Object::Number(n) => write!(f, "{}", n),
            Object::String(s) => write!(f, "\"{}\"", s),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Return(r) => write!(
                f,
                "{}",
                WithInterner {
                    value: r.as_ref(),
                    interner
                }
            ),
            Object::Error(s) => write!(f, "error: {}", s),
            Object::Function {
                name,
                parameters,
                body,
                env: _,
            } => match name {
                Some(n) => write!(
                    f,
                    "Fn {} ( {} ) {{ {} }}",
                    interner.resolve(*n),
                    parameters
                        .iter()
                        .map(|p| interner.resolve(*p))
                        .collect::<Vec<_>>()
                        .join(", "),
                    body.iter()
                        .map(|sttm| WithInterner {
                            value: sttm,
                            interner
                        }
                        .to_string())
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
                None => write!(
                    f,
                    "Fn ( {} ) {{ {} }}",
                    parameters
                        .iter()
                        .map(|p| interner.resolve(*p))
                        .collect::<Vec<_>>()
                        .join(", "),
                    body.iter()
                        .map(|sttm| WithInterner {
                            value: sttm,
                            interner
                        }
                        .to_string())
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
            },
            Object::Builtin { func: _ } => write!(f, ""),
            Object::Array(elements) => write!(
                f,
                "[ {} ]",
                elements
                    .iter()
                    .map(|elem| WithInterner {
                        value: elem,
                        interner
                    }
                    .to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Object::HashMap { pairs } => write!(
                f,
                "{{ {} }}",
                pairs
                    .iter()
                    .map(|(k, v)| { format!("{} : {}", k, WithInterner { value: v, interner }) })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Object;
    use crate::eval::object::CustomHash;

    #[test]
    fn test_string_hash() {
        let hello_one = Object::String(String::from("Hello world"));
        let hello_two = Object::String(String::from("Hello world"));

        let diff_one = Object::String(String::from("My Name is Jonny"));
        let diff_two = Object::String(String::from("My Name is Konny"));

        assert_eq!(
            hello_one.hash().unwrap().value,
            hello_two.hash().unwrap().value
        );
        assert_ne!(
            diff_one.hash().unwrap().value,
            diff_two.hash().unwrap().value
        );
    }

    #[test]
    fn test_number_hash() {
        let hello_one = Object::Number(-1);
        let hello_two = Object::Number(-1);

        let diff_one = Object::Number(2);
        let diff_two = Object::Number(3);

        assert_eq!(
            hello_one.hash().unwrap().value,
            hello_two.hash().unwrap().value
        );
        assert_ne!(
            diff_one.hash().unwrap().value,
            diff_two.hash().unwrap().value
        );
    }
}
