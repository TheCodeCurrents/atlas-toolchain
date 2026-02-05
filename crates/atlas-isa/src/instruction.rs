use crate::opcode::{AluOp, BranchCond, ImmOp, MemOp, PortOp, StackOp, XTypeOp};
use crate::operands::{BranchOperand, MOffset, RegisterIdentifier, RegisterPairIdentifier, XOperand};
use serde::{Deserialize, Serialize};

/// Instruction by mnemonic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    // A-type
    ADD,
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

    // I-type
    LDI,
    ADDI,
    SUBI,
    ANDI,
    ORI,

    // M-type
    LD,
    ST,

    // B*-types
    BR,
    BEQ,
    BNE,
    BCS,
    BCC,
    BMI,
    BPL,

    // S-type
    PUSH,
    POP,
    SUBSP,
    ADDSP,

    // P-type
    POKE,
    PEEK,

    // X-type
    SYSC,
    ERET,
    HALT,
    ICINV,
    DCINV,
    DCCLEAN,
    FLUSH,

    // Virtual instructions
    NOP,
}

pub enum InstructionFormat {
    A,
    I,
    M,
    B,
    S,
    P,
    X,
    Virtual
}

/// Resolved instruction with all operands specified, format is optimized for encoding and simulation
/// ! Note: Not every possible combination of fields is valid for a given instruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolvedInstruction {
    A {
        op: AluOp,
        dest: RegisterIdentifier,
        source: RegisterIdentifier,
        line: usize,
        #[serde(default)]
        source_file: Option<String>,
    },
    I {
        op: ImmOp,
        dest: RegisterIdentifier,
        immediate: u8,
        line: usize,
        #[serde(default)]
        source_file: Option<String>,
    },
    M {
        op: MemOp,
        dest: RegisterIdentifier,
        base: RegisterIdentifier,
        offset: MOffset,
        line: usize,
        #[serde(default)]
        source_file: Option<String>,
    },
    BI {
        absolute: bool,
        cond: BranchCond,
        operand: BranchOperand,
        line: usize,
        #[serde(default)]
        source_file: Option<String>,
    },
    BR {
        absolute: bool,
        cond: BranchCond,
        source: RegisterPairIdentifier,
        line: usize,
        #[serde(default)]
        source_file: Option<String>,
    },
    S {
        op: StackOp,
        register: RegisterIdentifier,
        line: usize,
        #[serde(default)]
        source_file: Option<String>,
    },
    P {
        op: PortOp,
        register: RegisterIdentifier,
        offset: u8,
        line: usize,
        #[serde(default)]
        source_file: Option<String>,
    },
    X {
        op: XTypeOp,
        operand: XOperand,
        line: usize,
        #[serde(default)]
        source_file: Option<String>,
    },
}

impl ResolvedInstruction {
    pub fn line(&self) -> usize {
        match self {
            ResolvedInstruction::A { line, .. } => *line,
            ResolvedInstruction::I { line, .. } => *line,
            ResolvedInstruction::M { line, .. } => *line,
            ResolvedInstruction::BI { line, .. } => *line,
            ResolvedInstruction::BR { line, .. } => *line,
            ResolvedInstruction::S { line, .. } => *line,
            ResolvedInstruction::P { line, .. } => *line,
            ResolvedInstruction::X { line, .. } => *line,
        }
    }

    pub fn source_file(&self) -> Option<&str> {
        match self {
            ResolvedInstruction::A { source_file, .. } => source_file.as_deref(),
            ResolvedInstruction::I { source_file, .. } => source_file.as_deref(),
            ResolvedInstruction::M { source_file, .. } => source_file.as_deref(),
            ResolvedInstruction::BI { source_file, .. } => source_file.as_deref(),
            ResolvedInstruction::BR { source_file, .. } => source_file.as_deref(),
            ResolvedInstruction::S { source_file, .. } => source_file.as_deref(),
            ResolvedInstruction::P { source_file, .. } => source_file.as_deref(),
            ResolvedInstruction::X { source_file, .. } => source_file.as_deref(),
        }
    }

    pub fn with_source_file(self, source_file: Option<String>) -> Self {
        match self {
            ResolvedInstruction::A { op, dest, source, line, .. } => {
                ResolvedInstruction::A { op, dest, source, line, source_file }
            }
            ResolvedInstruction::I { op, dest, immediate, line, .. } => {
                ResolvedInstruction::I { op, dest, immediate, line, source_file }
            }
            ResolvedInstruction::M { op, dest, base, offset, line, .. } => {
                ResolvedInstruction::M { op, dest, base, offset, line, source_file }
            }
            ResolvedInstruction::BI { absolute, cond, operand, line, .. } => {
                ResolvedInstruction::BI { absolute, cond, operand, line, source_file }
            }
            ResolvedInstruction::BR { absolute, cond, source, line, .. } => {
                ResolvedInstruction::BR { absolute, cond, source, line, source_file }
            }
            ResolvedInstruction::S { op, register, line, .. } => {
                ResolvedInstruction::S { op, register, line, source_file }
            }
            ResolvedInstruction::P { op, register, offset, line, .. } => {
                ResolvedInstruction::P { op, register, offset, line, source_file }
            }
            ResolvedInstruction::X { op, operand, line, .. } => {
                ResolvedInstruction::X { op, operand, line, source_file }
            }
        }
    }
}
