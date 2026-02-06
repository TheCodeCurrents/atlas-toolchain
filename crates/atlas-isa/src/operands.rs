use serde::{Deserialize, Serialize};

/// 8-bit register identifier
pub type RegisterIdentifier = u8;

/// Pair of registers (high and low)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterPairIdentifier {
    pub high: RegisterIdentifier,
    pub low: RegisterIdentifier,
}

/// Memory offset specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MOffset {
    /// 8-bit immediate offset
    Offset8(u8),
    /// Register offset
    SR(RegisterIdentifier),
}

/// A value that is either a resolved immediate or an unresolved label reference.
/// Used anywhere an immediate operand can also be specified via a label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operand {
    /// Resolved immediate value
    Immediate(u16),
    /// Reference to a label (to be resolved by linker)
    Label(String),
}

/// Backward-compatible alias – branches historically used this name.
pub type BranchOperand = Operand;

/// Operand for extended (X-type) instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum XOperand {
    None,
    Immediate(u8),
    Register(RegisterIdentifier),
    Registers(RegisterIdentifier, RegisterIdentifier),
}
