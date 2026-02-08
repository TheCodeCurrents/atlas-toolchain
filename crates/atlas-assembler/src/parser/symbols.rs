use std::collections::{HashMap, HashSet};
use atlas_isa::ParsedInstruction;

#[derive(Debug, Clone)]
pub enum Symbol {
    Label { offset: u32, section: String },
    Constant(u16),
}

/// An item emitted by the parser: either an instruction or raw data bytes.
#[derive(Debug)]
pub enum ParsedItem {
    Instruction(ParsedInstruction),
    Data(Vec<u8>),
    SectionChange(String),
}

/// Tracks a location in the section data that references a symbol and needs
/// to be patched by the linker.
#[derive(Debug, Clone)]
pub struct UnresolvedReference {
    /// Byte-offset within the current section where the reference lives.
    pub offset: u32,
    /// The section this reference is in.
    pub section: String,
    /// Name of the referenced symbol.
    pub symbol: String,
    /// Addend (usually 0).
    pub addend: i32,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
    exports: HashSet<String>,
    imports: HashSet<String>,
    /// Relocations collected during parsing.
    unresolved: Vec<UnresolvedReference>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            exports: HashSet::new(),
            imports: HashSet::new(),
            unresolved: Vec::new(),
        }
    }

    pub fn insert(&mut self, name: String, symbol: Symbol) {
        self.symbols.insert(name, symbol);
    }

    pub fn export(&mut self, name: String) {
        self.exports.insert(name);
    }

    pub fn import(&mut self, name: String) {
        self.imports.insert(name);
    }

    pub fn is_exported(&self, name: &str) -> bool {
        self.exports.contains(name)
    }

    pub fn is_imported(&self, name: &str) -> bool {
        self.imports.contains(name)
    }

    pub fn exports(&self) -> impl Iterator<Item = &String> {
        self.exports.iter()
    }

    pub fn imports(&self) -> impl Iterator<Item = &String> {
        self.imports.iter()
    }

    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, Symbol> {
        self.symbols.iter()
    }

    /// Record that a label reference at `offset` in `section` needs relocation.
    pub fn add_unresolved(&mut self, reference: UnresolvedReference) {
        self.unresolved.push(reference);
    }

    pub fn unresolved_references(&self) -> &[UnresolvedReference] {
        &self.unresolved
    }
}
