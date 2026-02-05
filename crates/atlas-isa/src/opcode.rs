use crate::Instruction;
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
    pub fn from_instruction(instruction: Instruction) -> Option<AluOp> {
        match instruction {
            Instruction::ADD => Some(AluOp::ADD),
            Instruction::ADDC => Some(AluOp::ADDC),
            Instruction::SUB => Some(AluOp::SUB),
            Instruction::SUBC => Some(AluOp::SUBC),
            Instruction::AND => Some(AluOp::AND),
            Instruction::OR => Some(AluOp::OR),
            Instruction::XOR => Some(AluOp::XOR),
            Instruction::NOT => Some(AluOp::NOT),
            Instruction::SHL => Some(AluOp::SHL),
            Instruction::SHR => Some(AluOp::SHR),
            Instruction::ROL => Some(AluOp::ROL),
            Instruction::ROR => Some(AluOp::ROR),
            Instruction::CMP => Some(AluOp::CMP),
            Instruction::TST => Some(AluOp::TST),
            Instruction::MOV => Some(AluOp::MOV),
            Instruction::NEG => Some(AluOp::NEG),
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
    pub fn from_instruction(instruction: Instruction) -> Option<ImmOp> {
        match instruction {
            Instruction::LDI => Some(ImmOp::LDI),
            Instruction::ADDI => Some(ImmOp::ADDI),
            Instruction::SUBI => Some(ImmOp::SUBI),
            Instruction::ANDI => Some(ImmOp::ANDI),
            Instruction::ORI => Some(ImmOp::ORI),
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
    pub fn from_instruction(instruction: Instruction) -> Option<MemOp> {
        match instruction {
            Instruction::LD => Some(MemOp::LD),
            Instruction::ST => Some(MemOp::ST),
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
    pub fn from_instruction(instruction: Instruction) -> Option<BranchCond> {
        match instruction {
            Instruction::BR => Some(BranchCond::Unconditional),
            Instruction::BEQ => Some(BranchCond::EQ),
            Instruction::BNE => Some(BranchCond::NE),
            Instruction::BCS => Some(BranchCond::CS),
            Instruction::BCC => Some(BranchCond::CC),
            Instruction::BMI => Some(BranchCond::MI),
            Instruction::BPL => Some(BranchCond::PL),
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
    pub fn from_instruction(instruction: Instruction) -> Option<StackOp> {
        match instruction {
            Instruction::PUSH => Some(StackOp::PUSH),
            Instruction::POP => Some(StackOp::POP),
            Instruction::SUBSP => Some(StackOp::SUBSP),
            Instruction::ADDSP => Some(StackOp::ADDSP),
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
    pub fn from_instruction(instruction: Instruction) -> Option<PortOp> {
        match instruction {
            Instruction::POKE => Some(PortOp::POKE),
            Instruction::PEEK => Some(PortOp::PEEK),
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
    pub fn from_instruction(instruction: Instruction) -> Option<XTypeOp> {
        match instruction {
            Instruction::SYSC => Some(XTypeOp::SYSC),
            Instruction::ERET => Some(XTypeOp::ERET),
            Instruction::HALT => Some(XTypeOp::HALT),
            Instruction::ICINV => Some(XTypeOp::ICINV),
            Instruction::DCINV => Some(XTypeOp::DCINV),
            Instruction::DCCLEAN => Some(XTypeOp::DCCLEAN),
            Instruction::FLUSH => Some(XTypeOp::FLUSH),
            _ => None,
        }
    }
}
