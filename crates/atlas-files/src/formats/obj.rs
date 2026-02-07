use serde::{Deserialize, Serialize};

use atlas_isa::ParsedInstruction;

/// Object file format - contains unresolved instructions that can be linked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectFile {
    /// Instructions with potentially unresolved label references
    pub instructions: Vec<ParsedInstruction>,

    /// Labels exported from this object file (for linking)
    pub exports: Vec<String>,

    /// Symbols defined or referenced by this object file
    #[serde(default)]
    pub symbols: Vec<Symbol>,
}

impl ObjectFile {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            exports: Vec::new(),
            symbols: Vec::new(),
        }
    }

    pub fn with_instructions(instructions: Vec<ParsedInstruction>) -> Self {
        Self {
            instructions,
            exports: Vec::new(),
            symbols: Vec::new(),
        }
    }

    /// Serialize to binary format
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self).map_err(|e| format!("Serialization failed: {}", e))
    }

    /// Deserialize from binary format
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        bincode::deserialize(bytes).map_err(|e| format!("Deserialization failed: {}", e))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub address: Option<u16>,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolKind {
    Local,
    Export,
    Import,
    /// A named constant (value, not an address)
    Constant,
}
