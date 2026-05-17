use crate::eval::object::Object;
use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use super::builtin_functions;

#[derive(Debug)]
pub struct Environment {
    pub store: HashMap<String, Object>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        builtin_functions()
    }

    pub fn new_enclosed(parent_env: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            store: HashMap::new(),
            parent: Some(parent_env),
        }))
    }

    pub fn get(&self, name: String) -> Option<Object> {
        let value_exists = self.store.get(name.as_str()).cloned();

        match value_exists {
            Some(value) => Some(value),
            None => match &self.parent {
                None => None,
                Some(parent) => parent.borrow().get(name),
            },
        }
    }

    pub fn set(&mut self, name: String, val: Object) {
        self.store.insert(name, val);
    }
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        let mut next_store = Self::new();

        for (k, v) in self.store.iter() {
            next_store.set(k.to_string(), v.clone());
        }

        return next_store;
    }
}
