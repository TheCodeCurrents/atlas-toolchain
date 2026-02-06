use crate::ParsedInstruction;
use crate::encoding_error::EncodingError;
use crate::operands::{BranchOperand, MOffset, RegisterPairIdentifier, XOperand};
use crate::opcode::{AluOp, ImmOp, MemOp, BranchCond, StackOp, PortOp, XTypeOp};


impl ParsedInstruction {
    pub fn encode(&self) -> Result<u16, EncodingError> {
        match &self {
            ParsedInstruction::A { op, dest, source, line: _, source_file: _ } => {
                let encoded = ((*dest as u16) << 12) 
                    | (*source as u16)
                    | ((*op as u16));
                Ok(encoded)
            }
            ParsedInstruction::I { op, dest, immediate, line: _, source_file: _ } => {
                let type_field = 1 + *op as u16;

                let encoded: u16 = ((type_field) << 12)
                    | ((*dest as u16) << 8)
                    | (*immediate as u16);

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
                    BranchOperand::Immediate(addr) => *addr,
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
                let encoded = (9 << 12)
                    | ((*absolute as u16) << 11)
                    | ((*cond as u16) << 8)
                    | ((source.high as u16) << 4)
                    | (source.low as u16);

                Ok(encoded)
            }
            ParsedInstruction::S { op, register, line: _, source_file: _ } => {
                let encoded = (10 << 12)
                    | ((*op as u16) << 8)
                    | (*register as u16);

                Ok(encoded)
            }
            ParsedInstruction::P { op, register, offset, line: _, source_file: _ } => {
                let encoded = (11 << 12)
                    | ((*op as u16) << 11)
                    | ((*register as u16) << 8)
                    | (*offset as u16);

                Ok(encoded)
            }
            ParsedInstruction::X { op, operand, line: _, source_file: _ } => {
                let encoded = (12 << 12)
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
                // A-type: bits [15:12]=0000, operation in [4:0], source in [7:0], dest in [15:12]
                let dest = ((encoded >> 12) & 0xF) as u8;
                let source = (encoded & 0xFF) as u8;
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
                let immediate = (encoded & 0xFF) as u8;

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
                    immediate,
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
                let address = (encoded & 0xFF) as u8;

                let cond = match cond_val {
                    0 => BranchCond::Unconditional,
                    1 => BranchCond::EQ,
                    2 => BranchCond::NE,
                    3 => BranchCond::CS,
                    4 => BranchCond::CC,
                    5 => BranchCond::MI,
                    6 => BranchCond::PL,
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
                // BR-type: bits [15:12]=1001, absolute in [11], condition in [10:8], high in [7:4], low in [3:0]
                let absolute = ((encoded >> 11) & 1) != 0;
                let cond_val = ((encoded >> 8) & 0x7) as u8;
                let high = ((encoded >> 4) & 0xF) as u8;
                let low = (encoded & 0xF) as u8;

                let cond = match cond_val {
                    0 => BranchCond::Unconditional,
                    1 => BranchCond::EQ,
                    2 => BranchCond::NE,
                    3 => BranchCond::CS,
                    4 => BranchCond::CC,
                    5 => BranchCond::MI,
                    6 => BranchCond::PL,
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
                // S-type: bits [15:12]=1010, operation in [11:8], register in [7:0]
                let op_val = ((encoded >> 8) & 0xF) as u8;
                let register = (encoded & 0xFF) as u8;

                let op = match op_val {
                    0 => StackOp::PUSH,
                    1 => StackOp::POP,
                    2 => StackOp::SUBSP,
                    3 => StackOp::ADDSP,
                    _ => return Err(format!("Invalid stack operation: {}", op_val)),
                };

                Ok(ParsedInstruction::S {
                    op,
                    register,
                    line: 0,
                    source_file: None,
                })
            }
            11 => {
                // P-type: bits [15:12]=1011, operation in [11], register in [10:8], offset in [7:0]
                let op_val = ((encoded >> 11) & 1) as u8;
                let register = ((encoded >> 8) & 0x7) as u8;
                let offset = (encoded & 0xFF) as u8;

                let op = match op_val {
                    0 => PortOp::PEEK,
                    1 => PortOp::POKE,
                    _ => return Err(format!("Invalid port operation: {}", op_val)),
                };

                Ok(ParsedInstruction::P {
                    op,
                    register,
                    offset,
                    line: 0,
                    source_file: None,
                })
            }
            12 => {
                // X-type: bits [15:12]=1100, operation in [11:8], operand in [7:0]
                let op_val = ((encoded >> 8) & 0xF) as u8;
                let operand_bits = (encoded & 0xFF) as u8;
                let source = ((operand_bits >> 4) & 0xF) as u8;
                let destination = (operand_bits & 0xF) as u8;

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

                Ok(ParsedInstruction::X {
                    op,
                    operand: XOperand::Registers(source, destination),
                    line: 0,
                    source_file: None,
                })
            }
            _ => Err(format!("Invalid opcode: {}", opcode)),
        }
    }
}