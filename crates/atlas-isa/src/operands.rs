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

/// Branch operand - can be an immediate address or a label reference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchOperand {
    /// Direct immediate address (resolved)
    Immediate(u8),
    /// Reference to a label (to be resolved by linker)
    Label(String),
}

/// Operand for extended (X-type) instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum XOperand {
    None,
    Immediate(u8),
    Register(RegisterIdentifier),
    Registers(RegisterIdentifier, RegisterIdentifier),
}
