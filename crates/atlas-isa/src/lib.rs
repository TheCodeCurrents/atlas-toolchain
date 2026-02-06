
//! Atlas ISA - Instruction Set Architecture definitions
//!
//! This crate provides type definitions and utilities for the Atlas instruction set architecture.
//! It includes instruction definitions, opcode mappings, and operand specifications.

pub mod instruction;
pub mod mnemonics;
pub mod opcode;
pub mod operands;
pub mod encoder;
pub mod encoding_error;
pub mod object_file;

// Re-export commonly used types
pub use instruction::{Mnemonic, ParsedInstruction};
pub use opcode::{AluOp, BranchCond, ImmOp, MemOp, PortOp, StackOp, XTypeOp};
pub use operands::{BranchOperand, MOffset, RegisterIdentifier, RegisterPairIdentifier, XOperand};
pub use encoding_error::EncodingError;
pub use object_file::{ObjectFile, Symbol, SymbolKind};