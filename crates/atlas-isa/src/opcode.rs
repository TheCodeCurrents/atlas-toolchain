use crate::Mnemonic;
use serde::{Deserialize, Serialize};

/// ALU operation codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AluOp {
    ADD = 0,
    ADDC,
    SUB,
    SUBC,
    AND,
    OR,
    XOR,
    NOT,
    SHL,
    SHR,
    ROL,
    ROR,
    CMP,
    TST,
    MOV,
    NEG,
}

impl AluOp {
    /// Check if the operation is a comparison operation
    pub fn from_instruction(instruction: Mnemonic) -> Option<AluOp> {
        match instruction {
            Mnemonic::ADD => Some(AluOp::ADD),
            Mnemonic::ADDC => Some(AluOp::ADDC),
            Mnemonic::SUB => Some(AluOp::SUB),
            Mnemonic::SUBC => Some(AluOp::SUBC),
            Mnemonic::AND => Some(AluOp::AND),
            Mnemonic::OR => Some(AluOp::OR),
            Mnemonic::XOR => Some(AluOp::XOR),
            Mnemonic::NOT => Some(AluOp::NOT),
            Mnemonic::SHL => Some(AluOp::SHL),
            Mnemonic::SHR => Some(AluOp::SHR),
            Mnemonic::ROL => Some(AluOp::ROL),
            Mnemonic::ROR => Some(AluOp::ROR),
            Mnemonic::CMP => Some(AluOp::CMP),
            Mnemonic::TST => Some(AluOp::TST),
            Mnemonic::MOV => Some(AluOp::MOV),
            Mnemonic::NEG => Some(AluOp::NEG),
            _ => None,
        }
    }
}

/// Immediate operation codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImmOp {
    LDI = 0,
    ADDI,
    SUBI,
    ANDI,
    ORI,
}

impl ImmOp {
    /// Check if the operation is a valid immediate operation
    pub fn from_instruction(instruction: Mnemonic) -> Option<ImmOp> {
        match instruction {
            Mnemonic::LDI => Some(ImmOp::LDI),
            Mnemonic::ADDI => Some(ImmOp::ADDI),
            Mnemonic::SUBI => Some(ImmOp::SUBI),
            Mnemonic::ANDI => Some(ImmOp::ANDI),
            Mnemonic::ORI => Some(ImmOp::ORI),
            _ => None,
        }
    }
}

/// Memory operation codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemOp {
    LD = 0,
    ST,
}

impl MemOp {
    pub fn from_instruction(instruction: Mnemonic) -> Option<MemOp> {
        match instruction {
            Mnemonic::LD => Some(MemOp::LD),
            Mnemonic::ST => Some(MemOp::ST),
            _ => None,
        }
    }
}

/// Branch condition codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchCond {
    Unconditional = 0,
    EQ,
    NE,
    CS,
    CC,
    MI,
    PL,
}

impl BranchCond {
    pub fn from_instruction(instruction: Mnemonic) -> Option<BranchCond> {
        match instruction {
            Mnemonic::BR => Some(BranchCond::Unconditional),
            Mnemonic::BEQ => Some(BranchCond::EQ),
            Mnemonic::BNE => Some(BranchCond::NE),
            Mnemonic::BCS => Some(BranchCond::CS),
            Mnemonic::BCC => Some(BranchCond::CC),
            Mnemonic::BMI => Some(BranchCond::MI),
            Mnemonic::BPL => Some(BranchCond::PL),
            _ => None,
        }
    }
}

/// Stack operation codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StackOp {
    PUSH = 0,
    POP,
    SUBSP,
    ADDSP,
}

impl StackOp {
    pub fn from_instruction(instruction: Mnemonic) -> Option<StackOp> {
        match instruction {
            Mnemonic::PUSH => Some(StackOp::PUSH),
            Mnemonic::POP => Some(StackOp::POP),
            Mnemonic::SUBSP => Some(StackOp::SUBSP),
            Mnemonic::ADDSP => Some(StackOp::ADDSP),
            _ => None,
        }
    }
}

/// Port operation codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortOp {
    POKE = 0,
    PEEK,
}

impl PortOp {
    pub fn from_instruction(instruction: Mnemonic) -> Option<PortOp> {
        match instruction {
            Mnemonic::POKE => Some(PortOp::POKE),
            Mnemonic::PEEK => Some(PortOp::PEEK),
            _ => None,
        }
    }
}

/// Extended operation codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum XTypeOp {
    SYSC = 0,
    ERET,
    HALT,
    ICINV,
    DCINV,
    DCCLEAN,
    FLUSH,
}

impl XTypeOp {
    pub fn from_instruction(instruction: Mnemonic) -> Option<XTypeOp> {
        match instruction {
            Mnemonic::SYSC => Some(XTypeOp::SYSC),
            Mnemonic::ERET => Some(XTypeOp::ERET),
            Mnemonic::HALT => Some(XTypeOp::HALT),
            Mnemonic::ICINV => Some(XTypeOp::ICINV),
            Mnemonic::DCINV => Some(XTypeOp::DCINV),
            Mnemonic::DCCLEAN => Some(XTypeOp::DCCLEAN),
            Mnemonic::FLUSH => Some(XTypeOp::FLUSH),
            _ => None,
        }
    }
}
