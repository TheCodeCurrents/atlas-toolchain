use crate::opcode::{AluOp, BranchCond, ImmOp, MemOp, PortOp, StackOp, XTypeOp};
use crate::operands::{BranchOperand, MOffset, RegisterIdentifier, RegisterPairIdentifier, XOperand};
use serde::{Deserialize, Serialize};

/// Instruction by mnemonic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mnemonic {
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
pub enum ParsedInstruction {
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

impl ParsedInstruction {
    pub fn line(&self) -> usize {
        match self {
            ParsedInstruction::A { line, .. } => *line,
            ParsedInstruction::I { line, .. } => *line,
            ParsedInstruction::M { line, .. } => *line,
            ParsedInstruction::BI { line, .. } => *line,
            ParsedInstruction::BR { line, .. } => *line,
            ParsedInstruction::S { line, .. } => *line,
            ParsedInstruction::P { line, .. } => *line,
            ParsedInstruction::X { line, .. } => *line,
        }
    }

    pub fn source_file(&self) -> Option<&str> {
        match self {
            ParsedInstruction::A { source_file, .. } => source_file.as_deref(),
            ParsedInstruction::I { source_file, .. } => source_file.as_deref(),
            ParsedInstruction::M { source_file, .. } => source_file.as_deref(),
            ParsedInstruction::BI { source_file, .. } => source_file.as_deref(),
            ParsedInstruction::BR { source_file, .. } => source_file.as_deref(),
            ParsedInstruction::S { source_file, .. } => source_file.as_deref(),
            ParsedInstruction::P { source_file, .. } => source_file.as_deref(),
            ParsedInstruction::X { source_file, .. } => source_file.as_deref(),
        }
    }

    pub fn with_source_file(self, source_file: Option<String>) -> Self {
        match self {
            ParsedInstruction::A { op, dest, source, line, .. } => {
                ParsedInstruction::A { op, dest, source, line, source_file }
            }
            ParsedInstruction::I { op, dest, immediate, line, .. } => {
                ParsedInstruction::I { op, dest, immediate, line, source_file }
            }
            ParsedInstruction::M { op, dest, base, offset, line, .. } => {
                ParsedInstruction::M { op, dest, base, offset, line, source_file }
            }
            ParsedInstruction::BI { absolute, cond, operand, line, .. } => {
                ParsedInstruction::BI { absolute, cond, operand, line, source_file }
            }
            ParsedInstruction::BR { absolute, cond, source, line, .. } => {
                ParsedInstruction::BR { absolute, cond, source, line, source_file }
            }
            ParsedInstruction::S { op, register, line, .. } => {
                ParsedInstruction::S { op, register, line, source_file }
            }
            ParsedInstruction::P { op, register, offset, line, .. } => {
                ParsedInstruction::P { op, register, offset, line, source_file }
            }
            ParsedInstruction::X { op, operand, line, .. } => {
                ParsedInstruction::X { op, operand, line, source_file }
            }
        }
    }
}
