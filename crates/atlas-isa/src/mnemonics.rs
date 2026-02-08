//! Mnemonic to opcode and family mappings
//!
//! This module provides conversions between instruction mnemonics and their
//! corresponding opcodes and instruction families.

use crate::instruction::{Mnemonic, InstructionFormat};

impl Mnemonic {
    /// Get the mnemonic string for the instruction
    pub fn mnemonic(&self) -> &'static str {
        match self {
            // A-type
            Mnemonic::ADD => "add",
            Mnemonic::ADDC => "addc",
            Mnemonic::SUB => "sub",
            Mnemonic::SUBC => "subc",
            Mnemonic::AND => "and",
            Mnemonic::OR => "or",
            Mnemonic::XOR => "xor",
            Mnemonic::NOT => "not",
            Mnemonic::SHL => "shl",
            Mnemonic::SHR => "shr",
            Mnemonic::ROL => "rol",
            Mnemonic::ROR => "ror",
            Mnemonic::CMP => "cmp",
            Mnemonic::TST => "tst",
            Mnemonic::MOV => "mov",
            Mnemonic::NEG => "neg",

            // I-type
            Mnemonic::LDI => "ldi",
            Mnemonic::ADDI => "addi",
            Mnemonic::SUBI => "subi",
            Mnemonic::ANDI => "andi",
            Mnemonic::ORI => "ori",

            // M-type
            Mnemonic::LD => "ld",
            Mnemonic::ST => "st",

            // B*-types
            Mnemonic::BR => "br",
            Mnemonic::BEQ => "beq",
            Mnemonic::BNE => "bne",
            Mnemonic::BCS => "bcs",
            Mnemonic::BCC => "bcc",
            Mnemonic::BMI => "bmi",
            Mnemonic::BPL => "bpl",
            Mnemonic::BOV => "bov",

            // S-type
            Mnemonic::PUSH => "push",
            Mnemonic::POP => "pop",
            Mnemonic::SUBSP => "subsp",
            Mnemonic::ADDSP => "addsp",

            // P-type
            Mnemonic::POKE => "poke",
            Mnemonic::PEEK => "peek",

            // X-type
            Mnemonic::SYSC => "sysc",
            Mnemonic::ERET => "eret",
            Mnemonic::HALT => "halt",
            Mnemonic::ICINV => "icinv",
            Mnemonic::DCINV => "dcinv",
            Mnemonic::DCCLEAN => "dcclean",
            Mnemonic::FLUSH => "flush",
            // Virtual instructions
            Mnemonic::NOP => "nop",
            Mnemonic::INC => "inc",
            Mnemonic::DEC => "dec",
        }
    }

    pub fn get_type(&self) -> InstructionFormat {
        match self {
            // A-type
            Mnemonic::ADD
            | Mnemonic::ADDC
            | Mnemonic::SUB
            | Mnemonic::SUBC
            | Mnemonic::AND
            | Mnemonic::OR
            | Mnemonic::XOR
            | Mnemonic::NOT
            | Mnemonic::SHL
            | Mnemonic::SHR
            | Mnemonic::ROL
            | Mnemonic::ROR
            | Mnemonic::CMP
            | Mnemonic::TST
            | Mnemonic::MOV
            | Mnemonic::NEG => InstructionFormat::A,

            // I-type
            Mnemonic::LDI
            | Mnemonic::ADDI
            | Mnemonic::SUBI
            | Mnemonic::ANDI
            | Mnemonic::ORI => InstructionFormat::I,

            // M-type
            Mnemonic::LD | Mnemonic::ST => InstructionFormat::M,

            // B*-types
            Mnemonic::BR
            | Mnemonic::BEQ
            | Mnemonic::BNE
            | Mnemonic::BCS
            | Mnemonic::BCC
            | Mnemonic::BMI
            | Mnemonic::BPL
            | Mnemonic::BOV => InstructionFormat::B,
            // S-type
            Mnemonic::PUSH
            | Mnemonic::POP
            | Mnemonic::SUBSP
            | Mnemonic::ADDSP => InstructionFormat::S,

            // P-type
            Mnemonic::POKE | Mnemonic::PEEK => InstructionFormat::P,

            // X-type
            Mnemonic::SYSC
            | Mnemonic::ERET
            | Mnemonic::HALT
            | Mnemonic::ICINV
            | Mnemonic::DCINV
            | Mnemonic::DCCLEAN
            | Mnemonic::FLUSH => InstructionFormat::X,
            // Virtual instructions
            Mnemonic::NOP
            | Mnemonic::INC
            | Mnemonic::DEC => InstructionFormat::Virtual,
        }
    }

    pub fn from_str(mnemonic: &str) -> Option<Self> {
        let mnemonic = mnemonic.to_lowercase();
        match mnemonic.as_str() {
            // A-type
            "add" => Some(Mnemonic::ADD),
            "addc" => Some(Mnemonic::ADDC),
            "sub" => Some(Mnemonic::SUB),
            "subc" => Some(Mnemonic::SUBC),
            "and" => Some(Mnemonic::AND),
            "or" => Some(Mnemonic::OR),
            "xor" => Some(Mnemonic::XOR),
            "not" => Some(Mnemonic::NOT),
            "shl" => Some(Mnemonic::SHL),
            "shr" => Some(Mnemonic::SHR),
            "rol" => Some(Mnemonic::ROL),
            "ror" => Some(Mnemonic::ROR),
            "cmp" => Some(Mnemonic::CMP),
            "tst" => Some(Mnemonic::TST),
            "mov" => Some(Mnemonic::MOV),
            "neg" => Some(Mnemonic::NEG),

            // I-type
            "ldi" => Some(Mnemonic::LDI),
            "addi" => Some(Mnemonic::ADDI),
            "subi" => Some(Mnemonic::SUBI),
            "andi" => Some(Mnemonic::ANDI),
            "ori" => Some(Mnemonic::ORI),

            // M-type
            "ld" => Some(Mnemonic::LD),
            "st" => Some(Mnemonic::ST),

            // B*-types
            "br" => Some(Mnemonic::BR),
            "beq" => Some(Mnemonic::BEQ),
            "bne" => Some(Mnemonic::BNE),
            "bcs" => Some(Mnemonic::BCS),
            "bcc" => Some(Mnemonic::BCC),
            "bmi" => Some(Mnemonic::BMI),
            "bpl" => Some(Mnemonic::BPL),
            "bov" => Some(Mnemonic::BOV),

            // S-type
            "push" => Some(Mnemonic::PUSH),
            "pop" => Some(Mnemonic::POP),
            "subsp" => Some(Mnemonic::SUBSP),
            "addsp" => Some(Mnemonic::ADDSP),

            // P-type
            "poke" => Some(Mnemonic::POKE),
            "peek" => Some(Mnemonic::PEEK),

            // X-type
            "sysc" => Some(Mnemonic::SYSC),
            "eret" => Some(Mnemonic::ERET),
            "halt" => Some(Mnemonic::HALT),
            "icinv" => Some(Mnemonic::ICINV),
            "dcinv" => Some(Mnemonic::DCINV),
            "dcclean" => Some(Mnemonic::DCCLEAN),
            "flush" => Some(Mnemonic::FLUSH),

            // Virtual instructions
            "nop" => Some(Mnemonic::NOP),
            "inc" => Some(Mnemonic::INC),
            "dec" => Some(Mnemonic::DEC),
            _ => None,
        }
    }
}