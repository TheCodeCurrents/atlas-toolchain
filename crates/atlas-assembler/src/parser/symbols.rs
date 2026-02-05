use std::collections::HashMap;

// TODO: Add support for constants/macros
#[derive(Debug, Clone)]
pub enum Symbol {
    Label(u32),
    External,
    // Constant(String of tokens),
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>
}

impl SymbolTable {
    pub fn new() -> Self {
        Self { symbols: HashMap::new() }
    }

    pub fn insert(&mut self, name: String, symbol: Symbol) {
        self.symbols.insert(name, symbol);
    }

    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, Symbol> {
        self.symbols.iter()
    }
}
