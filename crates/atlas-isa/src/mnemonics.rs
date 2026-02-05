//! Mnemonic to opcode and family mappings
//!
//! This module provides conversions between instruction mnemonics and their
//! corresponding opcodes and instruction families.

use crate::instruction::{Instruction, InstructionFormat};

impl Instruction {
    /// Get the mnemonic string for the instruction
    pub fn mnemonic(&self) -> &'static str {
        match self {
            // A-type
            Instruction::ADD => "add",
            Instruction::ADDC => "addc",
            Instruction::SUB => "sub",
            Instruction::SUBC => "subc",
            Instruction::AND => "and",
            Instruction::OR => "or",
            Instruction::XOR => "xor",
            Instruction::NOT => "not",
            Instruction::SHL => "shl",
            Instruction::SHR => "shr",
            Instruction::ROL => "rol",
            Instruction::ROR => "ror",
            Instruction::CMP => "cmp",
            Instruction::TST => "tst",
            Instruction::MOV => "mov",
            Instruction::NEG => "neg",

            // I-type
            Instruction::LDI => "ldi",
            Instruction::ADDI => "addi",
            Instruction::SUBI => "subi",
            Instruction::ANDI => "andi",
            Instruction::ORI => "ori",

            // M-type
            Instruction::LD => "ld",
            Instruction::ST => "st",

            // B*-types
            Instruction::BR => "br",
            Instruction::BEQ => "beq",
            Instruction::BNE => "bne",
            Instruction::BCS => "bcs",
            Instruction::BCC => "bcc",
            Instruction::BMI => "bmi",
            Instruction::BPL => "bpl",

            // S-type
            Instruction::PUSH => "push",
            Instruction::POP => "pop",
            Instruction::SUBSP => "subsp",
            Instruction::ADDSP => "addsp",

            // P-type
            Instruction::POKE => "poke",
            Instruction::PEEK => "peek",

            // X-type
            Instruction::SYSC => "sysc",
            Instruction::ERET => "eret",
            Instruction::HALT => "halt",
            Instruction::ICINV => "icinv",
            Instruction::DCINV => "dcinv",
            Instruction::DCCLEAN => "dcclean",
            Instruction::FLUSH => "flush",
            // Virtual instructions
            Instruction::NOP => "nop",
        }
    }

    pub fn get_type(&self) -> InstructionFormat {
        match self {
            // A-type
            Instruction::ADD
            | Instruction::ADDC
            | Instruction::SUB
            | Instruction::SUBC
            | Instruction::AND
            | Instruction::OR
            | Instruction::XOR
            | Instruction::NOT
            | Instruction::SHL
            | Instruction::SHR
            | Instruction::ROL
            | Instruction::ROR
            | Instruction::CMP
            | Instruction::TST
            | Instruction::MOV
            | Instruction::NEG => InstructionFormat::A,

            // I-type
            Instruction::LDI
            | Instruction::ADDI
            | Instruction::SUBI
            | Instruction::ANDI
            | Instruction::ORI => InstructionFormat::I,

            // M-type
            Instruction::LD | Instruction::ST => InstructionFormat::M,

            // B*-types
            Instruction::BR
            | Instruction::BEQ
            | Instruction::BNE
            | Instruction::BCS
            | Instruction::BCC
            | Instruction::BMI
            | Instruction::BPL => InstructionFormat::B,
            // S-type
            Instruction::PUSH
            | Instruction::POP
            | Instruction::SUBSP
            | Instruction::ADDSP => InstructionFormat::S,

            // P-type
            Instruction::POKE | Instruction::PEEK => InstructionFormat::P,

            // X-type
            Instruction::SYSC
            | Instruction::ERET
            | Instruction::HALT
            | Instruction::ICINV
            | Instruction::DCINV
            | Instruction::DCCLEAN
            | Instruction::FLUSH => InstructionFormat::X,
            // Virtual instructions
            Instruction::NOP => InstructionFormat::Virtual,
        }
    }

    pub fn from_str(mnemonic: &str) -> Option<Self> {
        let mnemonic = mnemonic.to_lowercase();
        match mnemonic.as_str() {
            // A-type
            "add" => Some(Instruction::ADD),
            "addc" => Some(Instruction::ADDC),
            "sub" => Some(Instruction::SUB),
            "subc" => Some(Instruction::SUBC),
            "and" => Some(Instruction::AND),
            "or" => Some(Instruction::OR),
            "xor" => Some(Instruction::XOR),
            "not" => Some(Instruction::NOT),
            "shl" => Some(Instruction::SHL),
            "shr" => Some(Instruction::SHR),
            "rol" => Some(Instruction::ROL),
            "ror" => Some(Instruction::ROR),
            "cmp" => Some(Instruction::CMP),
            "tst" => Some(Instruction::TST),
            "mov" => Some(Instruction::MOV),
            "neg" => Some(Instruction::NEG),

            // I-type
            "ldi" => Some(Instruction::LDI),
            "addi" => Some(Instruction::ADDI),
            "subi" => Some(Instruction::SUBI),
            "andi" => Some(Instruction::ANDI),
            "ori" => Some(Instruction::ORI),

            // M-type
            "ld" => Some(Instruction::LD),
            "st" => Some(Instruction::ST),

            // B*-types
            "br" => Some(Instruction::BR),
            "beq" => Some(Instruction::BEQ),
            "bne" => Some(Instruction::BNE),
            "bcs" => Some(Instruction::BCS),
            "bcc" => Some(Instruction::BCC),
            "bmi" => Some(Instruction::BMI),
            "bpl" => Some(Instruction::BPL),

            // S-type
            "push" => Some(Instruction::PUSH),
            "pop" => Some(Instruction::POP),
            "subsp" => Some(Instruction::SUBSP),
            "addsp" => Some(Instruction::ADDSP),

            // P-type
            "poke" => Some(Instruction::POKE),
            "peek" => Some(Instruction::PEEK),

            // X-type
            "sysc" => Some(Instruction::SYSC),
            "eret" => Some(Instruction::ERET),
            "halt" => Some(Instruction::HALT),
            "icinv" => Some(Instruction::ICINV),
            "dcinv" => Some(Instruction::DCINV),
            "dcclean" => Some(Instruction::DCCLEAN),
            "flush" => Some(Instruction::FLUSH),

            // Virtual instructions
            "nop" => Some(Instruction::NOP),
            _ => None,
        }
    }
}