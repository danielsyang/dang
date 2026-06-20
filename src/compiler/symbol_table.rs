use std::collections::HashMap;

use crate::intern::interner::Symbol;

#[derive(Debug, PartialEq)]
pub enum SymbolScope {
    GlobalScope,
}

#[derive(Debug, PartialEq)]
pub struct SymbolEntry {
    name: Symbol,
    scope: SymbolScope,
    pub index: usize,
}

pub struct SymbolTable {
    pub store: HashMap<Symbol, SymbolEntry>,
    num_definitions: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            store: HashMap::new(),
            num_definitions: 0,
        }
    }

    pub fn define(&mut self, name: &Symbol) -> &SymbolEntry {
        let symbol = SymbolEntry {
            name: *name,
            index: self.num_definitions,
            scope: SymbolScope::GlobalScope,
        };

        self.store.insert(*name, symbol);
        self.num_definitions += 1;

        self.store.get(name).unwrap_or_else(|| {
            panic!(
                "I just inserted the name {:?} , how is this possible!?",
                name
            )
        })
    }

    pub fn resolve(&self, name: &Symbol) -> Option<&SymbolEntry> {
        self.store.get(name)
    }
}

#[cfg(test)]
mod test {
    use crate::intern::interner::Symbol;

    use super::{SymbolEntry, SymbolScope, SymbolTable};

    #[test]
    fn define() {
        let mut global = SymbolTable::new();
        let symbol_a = Symbol(0);
        let symbol_b = Symbol(1);

        let a = global.define(&symbol_a);
        assert_eq!(
            a,
            &SymbolEntry {
                name: symbol_a,
                scope: SymbolScope::GlobalScope,
                index: 0
            },
        );

        let b = global.define(&symbol_b);
        assert_eq!(
            b,
            &SymbolEntry {
                name: symbol_b,
                scope: SymbolScope::GlobalScope,
                index: 1
            },
        );
    }

    #[test]
    fn resolve_global() {
        let mut global = SymbolTable::new();
        let symbol_a = Symbol(0);
        let symbol_b = Symbol(1);

        global.define(&symbol_a);
        global.define(&symbol_b);

        let expected = [
            SymbolEntry {
                name: symbol_a,
                scope: SymbolScope::GlobalScope,
                index: 0,
            },
            SymbolEntry {
                name: symbol_b,
                scope: SymbolScope::GlobalScope,
                index: 1,
            },
        ];

        for sym in expected {
            assert_eq!(global.resolve(&sym.name), Some(&sym));
        }
    }
}
