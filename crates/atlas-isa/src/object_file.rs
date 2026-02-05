use serde::{Deserialize, Serialize};
use crate::ResolvedInstruction;
use crate::opcode::{AluOp, BranchCond, ImmOp, MemOp, PortOp, StackOp, XTypeOp};
use crate::operands::{BranchOperand, MOffset, RegisterIdentifier, RegisterPairIdentifier, XOperand};

/// Object file format - contains unresolved instructions that can be linked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectFile {
    /// Instructions with potentially unresolved label references
    pub instructions: Vec<ResolvedInstruction>,
    
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
    
    pub fn with_instructions(instructions: Vec<ResolvedInstruction>) -> Self {
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
        match bincode::deserialize(bytes) {
            Ok(current) => Ok(current),
            Err(_current_err) => {
                let legacy: LegacyObjectFile = bincode::deserialize(bytes)
                    .map_err(|legacy_err| format!("Deserialization failed: {}", legacy_err))?;
                let instructions = legacy
                    .instructions
                    .into_iter()
                    .map(ResolvedInstruction::from)
                    .collect();
                Ok(ObjectFile {
                    instructions,
                    exports: legacy.exports,
                    symbols: Vec::new(),
                })
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub address: u8,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolKind {
    Local,
    Export,
    Import,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyObjectFile {
    pub instructions: Vec<LegacyResolvedInstruction>,
    pub exports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum LegacyResolvedInstruction {
    A {
        op: AluOp,
        dest: RegisterIdentifier,
        source: RegisterIdentifier,
        line: usize,
    },
    I {
        op: ImmOp,
        dest: RegisterIdentifier,
        immediate: u8,
        line: usize,
    },
    M {
        op: MemOp,
        dest: RegisterIdentifier,
        base: RegisterIdentifier,
        offset: MOffset,
        line: usize,
    },
    BI {
        absolute: bool,
        cond: BranchCond,
        operand: BranchOperand,
        line: usize,
    },
    BR {
        absolute: bool,
        cond: BranchCond,
        source: RegisterPairIdentifier,
        line: usize,
    },
    S {
        op: StackOp,
        register: RegisterIdentifier,
        line: usize,
    },
    P {
        op: PortOp,
        register: RegisterIdentifier,
        offset: u8,
        line: usize,
    },
    X {
        op: XTypeOp,
        operand: XOperand,
        line: usize,
    },
}

impl From<LegacyResolvedInstruction> for ResolvedInstruction {
    fn from(value: LegacyResolvedInstruction) -> Self {
        match value {
            LegacyResolvedInstruction::A { op, dest, source, line } => {
                ResolvedInstruction::A { op, dest, source, line, source_file: None }
            }
            LegacyResolvedInstruction::I { op, dest, immediate, line } => {
                ResolvedInstruction::I { op, dest, immediate, line, source_file: None }
            }
            LegacyResolvedInstruction::M { op, dest, base, offset, line } => {
                ResolvedInstruction::M { op, dest, base, offset, line, source_file: None }
            }
            LegacyResolvedInstruction::BI { absolute, cond, operand, line } => {
                ResolvedInstruction::BI { absolute, cond, operand, line, source_file: None }
            }
            LegacyResolvedInstruction::BR { absolute, cond, source, line } => {
                ResolvedInstruction::BR { absolute, cond, source, line, source_file: None }
            }
            LegacyResolvedInstruction::S { op, register, line } => {
                ResolvedInstruction::S { op, register, line, source_file: None }
            }
            LegacyResolvedInstruction::P { op, register, offset, line } => {
                ResolvedInstruction::P { op, register, offset, line, source_file: None }
            }
            LegacyResolvedInstruction::X { op, operand, line } => {
                ResolvedInstruction::X { op, operand, line, source_file: None }
            }
        }
    }
}
