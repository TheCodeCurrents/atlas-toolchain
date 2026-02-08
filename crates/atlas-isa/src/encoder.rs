use crate::ParsedInstruction;
use crate::encoding_error::EncodingError;
use crate::operands::{BranchOperand, MOffset, Operand, RegisterPairIdentifier, XOperand};
use crate::opcode::{AluOp, ImmOp, MemOp, BranchCond, StackOp, PeekPokeOp, XTypeOp};


impl ParsedInstruction {
    pub fn encode(&self) -> Result<u16, EncodingError> {
        match &self {
            ParsedInstruction::A { op, dest, source, line: _, source_file: _ } => {
                // A-type: [15:12]=0000, [11:8]=dest, [7:4]=source, [3:0]=op
                let encoded = ((*dest as u16) << 8)
                    | ((*source as u16) << 4)
                    | (*op as u16);
                Ok(encoded)
            }
            ParsedInstruction::I { op, dest, immediate, line, source_file: _ } => {
                let imm_val = match immediate {
                    Operand::Immediate(val) => {
                        if *val > 0xFF {
                            return Err(EncodingError {
                                line: *line,
                                message: format!("Immediate value 0x{:x} exceeds 8-bit range", val),
                            });
                        }
                        *val
                    }
                    Operand::Label(name) => {
                        return Err(EncodingError {
                            line: *line,
                            message: format!("Cannot encode unresolved label reference: '{}'", name),
                        });
                    }
                };
                let type_field = 1 + *op as u16;

                let encoded: u16 = ((type_field) << 12)
                    | ((*dest as u16) << 8)
                    | (imm_val as u16);

                Ok(encoded)
            }
            ParsedInstruction::M { op, dest, base, offset, line: _, source_file: _ } => {
                let type_field = 6 + *op as u16;

                let offset_val = match offset {
                    MOffset::Offset8(val) => *val,
                    MOffset::SR(reg) => *reg,
                };

                let encoded: u16 = ((type_field) << 12)
                    | ((*dest as u16) << 8)
                    | ((*base as u16) << 4)
                    | ((offset_val as u16) & 0xF);

                Ok(encoded)
            }
            ParsedInstruction::BI { absolute, cond, operand, line, source_file: _ } => {
                let address = match operand {
                    BranchOperand::Immediate(addr) => {
                        if *addr > 0xFF {
                            return Err(EncodingError {
                                line: *line,
                                message: format!("Branch address 0x{:x} exceeds 8-bit range", addr),
                            });
                        }
                        *addr
                    }
                    BranchOperand::Label(name) => {
                        return Err(EncodingError {
                            line: *line,
                            message: format!("Cannot encode unresolved label reference: '{}'", name),
                        });
                    }
                };
                
                let encoded = (8 << 12)
                    | ((*absolute as u16) << 11)
                    | ((*cond as u16) << 8)
                    | (address as u16);

                Ok(encoded)
            }
            ParsedInstruction::BR { absolute, cond, source, line: _, source_file: _ } => {
                // BR-type: [15:12]=1001, [11]=abs, [10:8]=cond, [7:4]=rs_low, [3:0]=rs_high
                let encoded = (9 << 12)
                    | ((*absolute as u16) << 11)
                    | ((*cond as u16) << 8)
                    | ((source.low as u16) << 4)
                    | (source.high as u16);

                Ok(encoded)
            }
            ParsedInstruction::S { op, operand, line: _, source_file: _ } => {
                // S-type: [15:12]=1010, [11:8]=xop, [7:0]=operand (register or imm8)
                let encoded = (10 << 12)
                    | ((*op as u16) << 8)
                    | (*operand as u16);

                Ok(encoded)
            }
            ParsedInstruction::P { op, register, offset, line, source_file: _ } => {
                let offset_val = match offset {
                    Operand::Immediate(val) => {
                        if *val > 0xFF {
                            return Err(EncodingError {
                                line: *line,
                                message: format!("Peek/Poke offset 0x{:x} exceeds 8-bit range", val),
                            });
                        }
                        *val
                    }
                    Operand::Label(name) => {
                        return Err(EncodingError {
                            line: *line,
                            message: format!("Cannot encode unresolved label reference: '{}'", name),
                        });
                    }
                };
                // P-type: peek = type-field 1011, poke = type-field 1100
                // [15:12]=type, [11:8]=register, [7:0]=offset
                let type_field: u16 = match op {
                    PeekPokeOp::PEEK => 0xB, // 1011
                    PeekPokeOp::POKE => 0xC, // 1100
                };
                let encoded = (type_field << 12)
                    | ((*register as u16) << 8)
                    | (offset_val as u16);

                Ok(encoded)
            }
            ParsedInstruction::X { op, operand, line: _, source_file: _ } => {
                // X-type: [15:12]=1101 (13)
                let encoded = (13 << 12)
                    | ((*op as u16) << 8)
                    | match operand {
                        crate::operands::XOperand::Registers(source, destination) => {
                            ((*source as u16) << 4) | (*destination as u16)
                        }
                        crate::operands::XOperand::Immediate(imm) => {
                            *imm as u16
                        }
                        crate::operands::XOperand::Register(reg) => {
                            *reg as u16
                        }
                        crate::operands::XOperand::None => {
                            0
                        }
                    };

                Ok(encoded)
            }
        }
    }

    pub fn decode(encoded: u16) -> Result<ParsedInstruction, String> {
        let opcode = (encoded >> 12) & 0xF;

        match opcode {
            0 => {
                // A-type: [15:12]=0000, [11:8]=dest, [7:4]=source, [3:0]=op
                let dest = ((encoded >> 8) & 0xF) as u8;
                let source = ((encoded >> 4) & 0xF) as u8;
                let op_val = (encoded & 0xF) as u8;

                let op = match op_val {
                    0 => AluOp::ADD,
                    1 => AluOp::ADDC,
                    2 => AluOp::SUB,
                    3 => AluOp::SUBC,
                    4 => AluOp::AND,
                    5 => AluOp::OR,
                    6 => AluOp::XOR,
                    7 => AluOp::NOT,
                    8 => AluOp::SHL,
                    9 => AluOp::SHR,
                    10 => AluOp::ROL,
                    11 => AluOp::ROR,
                    12 => AluOp::CMP,
                    13 => AluOp::TST,
                    14 => AluOp::MOV,
                    15 => AluOp::NEG,
                    _ => return Err(format!("Invalid ALU operation: {}", op_val)),
                };

                Ok(ParsedInstruction::A {
                    op,
                    dest,
                    source,
                    line: 0,
                    source_file: None,
                })
            }
            1..=5 => {
                // I-type: bits [15:12]=type_field (1+operation), dest in [11:8], imm in [7:0]
                let op_val = (opcode - 1) as u8;
                let dest = ((encoded >> 8) & 0xF) as u8;
                let immediate = (encoded & 0xFF) as u16;

                let op = match op_val {
                    0 => ImmOp::LDI,
                    1 => ImmOp::ADDI,
                    2 => ImmOp::SUBI,
                    3 => ImmOp::ANDI,
                    4 => ImmOp::ORI,
                    _ => return Err(format!("Invalid immediate operation: {}", op_val)),
                };

                Ok(ParsedInstruction::I {
                    op,
                    dest,
                    immediate: Operand::Immediate(immediate),
                    line: 0,
                    source_file: None,
                })
            }
            6..=7 => {
                // M-type: bits [15:12]=type_field (6+operation), dest in [11:8], base in [7:4], offset in [3:0]
                let op_val = (opcode - 6) as u8;
                let dest = ((encoded >> 8) & 0xF) as u8;
                let base = ((encoded >> 4) & 0xF) as u8;
                let offset_val = (encoded & 0xF) as u8;

                let op = match op_val {
                    0 => MemOp::LD,
                    1 => MemOp::ST,
                    _ => return Err(format!("Invalid memory operation: {}", op_val)),
                };

                Ok(ParsedInstruction::M {
                    op,
                    dest,
                    base,
                    offset: MOffset::Offset8(offset_val),
                    line: 0,
                    source_file: None,
                })
            }
            8 => {
                // BI-type: bits [15:12]=1000, absolute in [11], condition in [10:8], address in [7:0]
                let absolute = ((encoded >> 11) & 1) != 0;
                let cond_val = ((encoded >> 8) & 0x7) as u8;
                let address = (encoded & 0xFF) as u16;

                let cond = match cond_val {
                    0 => BranchCond::Unconditional,
                    1 => BranchCond::EQ,
                    2 => BranchCond::NE,
                    3 => BranchCond::CS,
                    4 => BranchCond::CC,
                    5 => BranchCond::MI,
                    6 => BranchCond::PL,
                    7 => BranchCond::OV,
                    _ => return Err(format!("Invalid branch condition: {}", cond_val)),
                };

                Ok(ParsedInstruction::BI {
                    absolute,
                    cond,
                    operand: BranchOperand::Immediate(address),
                    line: 0,
                    source_file: None,
                })
            }
            9 => {
                // BR-type: bits [15:12]=1001, absolute in [11], condition in [10:8],
                //          [7:4]=rs_low, [3:0]=rs_high
                let absolute = ((encoded >> 11) & 1) != 0;
                let cond_val = ((encoded >> 8) & 0x7) as u8;
                let low = ((encoded >> 4) & 0xF) as u8;
                let high = (encoded & 0xF) as u8;

                let cond = match cond_val {
                    0 => BranchCond::Unconditional,
                    1 => BranchCond::EQ,
                    2 => BranchCond::NE,
                    3 => BranchCond::CS,
                    4 => BranchCond::CC,
                    5 => BranchCond::MI,
                    6 => BranchCond::PL,
                    7 => BranchCond::OV,
                    _ => return Err(format!("Invalid branch condition: {}", cond_val)),
                };

                Ok(ParsedInstruction::BR {
                    absolute,
                    cond,
                    source: RegisterPairIdentifier { high, low },
                    line: 0,
                    source_file: None,
                })
            }
            10 => {
                // S-type: bits [15:12]=1010, xop in [11:8], operand in [7:0]
                let op_val = ((encoded >> 8) & 0xF) as u8;
                let operand = (encoded & 0xFF) as u8;

                let op = match op_val {
                    0 => StackOp::PUSH,
                    1 => StackOp::POP,
                    2 => StackOp::SUBSP_IMM,
                    3 => StackOp::SUBSP_REG,
                    4 => StackOp::ADDSP_IMM,
                    5 => StackOp::ADDSP_REG,
                    _ => return Err(format!("Invalid stack operation: {}", op_val)),
                };

                Ok(ParsedInstruction::S {
                    op,
                    operand,
                    line: 0,
                    source_file: None,
                })
            }
            11 => {
                // P-type peek: [15:12]=1011, [11:8]=register, [7:0]=offset
                let register = ((encoded >> 8) & 0xF) as u8;
                let offset = (encoded & 0xFF) as u16;

                Ok(ParsedInstruction::P {
                    op: PeekPokeOp::PEEK,
                    register,
                    offset: Operand::Immediate(offset),
                    line: 0,
                    source_file: None,
                })
            }
            12 => {
                // P-type poke: [15:12]=1100, [11:8]=register, [7:0]=offset
                let register = ((encoded >> 8) & 0xF) as u8;
                let offset = (encoded & 0xFF) as u16;

                Ok(ParsedInstruction::P {
                    op: PeekPokeOp::POKE,
                    register,
                    offset: Operand::Immediate(offset),
                    line: 0,
                    source_file: None,
                })
            }
            13 => {
                // X-type: bits [15:12]=1101, operation in [11:8], operand in [7:0]
                let op_val = ((encoded >> 8) & 0xF) as u8;
                let operand_val = (encoded & 0xFF) as u8;

                let op = match op_val {
                    0 => XTypeOp::SYSC,
                    1 => XTypeOp::ERET,
                    2 => XTypeOp::HALT,
                    3 => XTypeOp::ICINV,
                    4 => XTypeOp::DCINV,
                    5 => XTypeOp::DCCLEAN,
                    6 => XTypeOp::FLUSH,
                    _ => return Err(format!("Invalid extended operation: {}", op_val)),
                };

                // Determine operand based on instruction type
                let operand = match op {
                    XTypeOp::SYSC => XOperand::Immediate(operand_val),
                    _ => {
                        if operand_val == 0 {
                            XOperand::None
                        } else {
                            let source = (operand_val >> 4) & 0xF;
                            let destination = operand_val & 0xF;
                            XOperand::Registers(source, destination)
                        }
                    }
                };

                Ok(ParsedInstruction::X {
                    op,
                    operand,
                    line: 0,
                    source_file: None,
                })
            }
            _ => Err(format!("Invalid opcode: {}", opcode)),
        }
    }
}