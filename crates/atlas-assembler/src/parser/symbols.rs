use std::collections::{HashMap, HashSet};

// TODO: Add support for constants/macros
#[derive(Debug, Clone)]
pub enum Symbol {
    Label(u32),
    External,
    // Constant(String of tokens),
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
    exports: HashSet<String>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            exports: HashSet::new(),
        }
    }

    pub fn insert(&mut self, name: String, symbol: Symbol) {
        self.symbols.insert(name, symbol);
    }

    pub fn export(&mut self, name: String) {
        self.exports.insert(name);
    }

    pub fn is_exported(&self, name: &str) -> bool {
        self.exports.contains(name)
    }

    pub fn exports(&self) -> impl Iterator<Item = &String> {
        self.exports.iter()
    }

    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, Symbol> {
        self.symbols.iter()
    }
}
