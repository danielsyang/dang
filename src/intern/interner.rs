use std::collections::HashMap;

#[derive(Debug)]
pub struct Interner {
    map: HashMap<String, Symbol>,
    names: Vec<String>,
}

impl Interner {
    pub fn new() -> Self {
        Interner {
            map: HashMap::new(),
            names: Vec::new(),
        }
    }

    pub fn intern(&mut self, val: &str) -> Symbol {
        let symbol = self.map.get(val).copied();

        match symbol {
            None => {
                let size = self.names.len();
                self.names.push(val.to_string());
                let val_symbol = Symbol(size as u32);

                self.map.insert(val.to_string(), val_symbol);

                val_symbol
            }
            Some(symbol) => symbol,
        }
    }

    pub fn resolve(&self, n: Symbol) -> &str {
        &self.names[n.0 as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Symbol(u32);

pub trait PrettyDisplay {
    fn pretty(&self, f: &mut std::fmt::Formatter, interner: &Interner) -> std::fmt::Result;
}

pub struct WithInterner<'a, T: ?Sized> {
    pub value: &'a T,
    pub interner: &'a Interner,
}

impl<'a, T: PrettyDisplay + ?Sized> std::fmt::Display for WithInterner<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.value.pretty(f, self.interner)
    }
}
