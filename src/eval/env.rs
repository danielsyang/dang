use crate::{
    eval::object::Object,
    intern::interner::{Interner, Symbol},
};
use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use super::builtin_functions;

#[derive(Debug)]
pub struct Environment {
    pub store: HashMap<Symbol, Object>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(interner: &mut Interner) -> Self {
        builtin_functions(interner)
    }

    pub fn new_enclosed(parent_env: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            store: HashMap::new(),
            parent: Some(parent_env),
        }))
    }

    pub fn get(&self, name: Symbol) -> Option<Object> {
        let value_exists = self.store.get(&name).cloned();

        match value_exists {
            Some(value) => Some(value),
            None => match &self.parent {
                None => None,
                Some(parent) => parent.borrow().get(name),
            },
        }
    }

    pub fn set(&mut self, name: Symbol, val: Object) {
        self.store.insert(name, val);
    }
}
