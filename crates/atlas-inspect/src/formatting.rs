use atlas_isa::opcode::*;
use atlas_isa::operands::*;
use std::collections::BTreeMap;

// ── Colours / style helpers ────────────────────────────────────────────────
// Respects NO_COLOR (https://no-color.org/).
pub fn use_colour() -> bool {
    std::env::var_os("NO_COLOR").is_none()
}

pub fn dim(s: &str) -> String {
    if use_colour() { format!("\x1b[2m{}\x1b[0m", s) } else { s.to_string() }
}

pub fn bold(s: &str) -> String {
    if use_colour() { format!("\x1b[1m{}\x1b[0m", s) } else { s.to_string() }
}

pub fn green(s: &str) -> String {
    if use_colour() { format!("\x1b[32m{}\x1b[0m", s) } else { s.to_string() }
}

pub fn cyan(s: &str) -> String {
    if use_colour() { format!("\x1b[36m{}\x1b[0m", s) } else { s.to_string() }
}

pub fn yellow(s: &str) -> String {
    if use_colour() { format!("\x1b[33m{}\x1b[0m", s) } else { s.to_string() }
}

// ── Instruction formatting ─────────────────────────────────────────────────

pub fn format_instruction(instr: &atlas_isa::ParsedInstruction, labels: &BTreeMap<u16, String>) -> String {
    use atlas_isa::ParsedInstruction;

    match instr {
        ParsedInstruction::A { op, dest, source, .. } => {
            // NOP detection: add r0, r0
            if matches!(op, AluOp::ADD) && *dest == 0 && *source == 0 {
                return "nop".to_string();
            }
            let mnemonic = alu_op_name(*op);
            format!("{:<8} {}, {}", mnemonic, reg_name(*dest), reg_name(*source))
        }
        ParsedInstruction::I { op, dest, immediate, .. } => {
            let mnemonic = imm_op_name(*op);
            let operand = format_operand(immediate, labels);
            format!("{:<8} {}, {}", mnemonic, reg_name(*dest), operand)
        }
        ParsedInstruction::M { op, dest, base, offset, .. } => {
            let mnemonic = mem_op_name(*op);
            let off_str = match offset {
                MOffset::Offset8(v) => format!("{}", *v as i8 as i32),
                MOffset::SR(r) => reg_name(*r).to_string(),
            };
            format!("{:<8} {}, [{}, {}]", mnemonic, reg_name(*dest), reg_name(*base), off_str)
        }
        ParsedInstruction::BI { cond, operand, .. } => {
            let mnemonic = branch_cond_name(*cond);
            let target = format_operand(operand, labels);
            format!("{:<8} {}", mnemonic, target)
        }
        ParsedInstruction::BR { cond, source, .. } => {
            let mnemonic = branch_cond_name(*cond);
            format!("{:<8} {}, {}", mnemonic, reg_name(source.high), reg_name(source.low))
        }
        ParsedInstruction::S { op, operand, .. } => {
            let mnemonic = stack_op_name(*op);
            match op {
                StackOp::PUSH | StackOp::POP | StackOp::SUBSP_REG | StackOp::ADDSP_REG => {
                    format!("{:<8} {}", mnemonic, reg_name(*operand))
                }
                StackOp::SUBSP_IMM | StackOp::ADDSP_IMM => {
                    format!("{:<8} 0x{:02x}", mnemonic, operand)
                }
            }
        }
        ParsedInstruction::P { op, register, offset, .. } => {
            let mnemonic = port_op_name(*op);
            let operand = format_operand(offset, labels);
            format!("{:<8} {}, {}", mnemonic, reg_name(*register), operand)
        }
        ParsedInstruction::X { op, operand, .. } => {
            let mnemonic = x_op_name(*op);
            match operand {
                XOperand::None => mnemonic.to_string(),
                XOperand::Immediate(v) => format!("{:<8} 0x{:02x}", mnemonic, v),
                XOperand::Register(r) => format!("{:<8} {}", mnemonic, reg_name(*r)),
                XOperand::Registers(a, b) => {
                    // If both are r0 and the instruction doesn't logically use
                    // operands (e.g. halt), treat as no-operand.
                    if *a == 0 && *b == 0 {
                        mnemonic.to_string()
                    } else {
                        format!("{:<8} {}, {}", mnemonic, reg_name(*a), reg_name(*b))
                    }
                }
            }
        }
    }
}

pub fn format_operand(op: &Operand, labels: &BTreeMap<u16, String>) -> String {
    match op {
        Operand::Immediate(v) => {
            if let Some(name) = labels.get(v) {
                name.clone()
            } else {
                format!("0x{:02x}", v)
            }
        }
        Operand::Label(name) => format!("<{}>", name),
    }
}

pub fn reg_name(r: u8) -> &'static str {
    match r {
        0 => "r0", 1 => "r1", 2 => "r2", 3 => "r3",
        4 => "r4", 5 => "r5", 6 => "r6", 7 => "r7",
        8 => "r8", 9 => "r9", 10 => "tr", 11 => "r11",
        12 => "sp", 13 => "r13", 14 => "pc", 15 => "r15",
        _ => "r?",
    }
}

pub fn alu_op_name(op: AluOp) -> &'static str {
    match op {
        AluOp::ADD => "add", AluOp::ADDC => "addc", AluOp::SUB => "sub", AluOp::SUBC => "subc",
        AluOp::AND => "and", AluOp::OR => "or", AluOp::XOR => "xor", AluOp::NOT => "not",
        AluOp::SHL => "shl", AluOp::SHR => "shr", AluOp::ROL => "rol", AluOp::ROR => "ror",
        AluOp::CMP => "cmp", AluOp::TST => "tst", AluOp::MOV => "mov", AluOp::NEG => "neg",
    }
}

pub fn imm_op_name(op: ImmOp) -> &'static str {
    match op { ImmOp::LDI => "ldi", ImmOp::ADDI => "addi", ImmOp::SUBI => "subi", ImmOp::ANDI => "andi", ImmOp::ORI => "ori" }
}

pub fn mem_op_name(op: MemOp) -> &'static str {
    match op { MemOp::LD => "ld", MemOp::ST => "st" }
}

pub fn branch_cond_name(cond: BranchCond) -> &'static str {
    match cond {
        BranchCond::Unconditional => "br", BranchCond::EQ => "beq", BranchCond::NE => "bne",
        BranchCond::CS => "bcs", BranchCond::CC => "bcc", BranchCond::MI => "bmi", BranchCond::PL => "bpl",
        BranchCond::OV => "bov",
    }
}

pub fn stack_op_name(op: StackOp) -> &'static str {
    match op {
        StackOp::PUSH => "push", StackOp::POP => "pop",
        StackOp::SUBSP_IMM | StackOp::SUBSP_REG => "subsp",
        StackOp::ADDSP_IMM | StackOp::ADDSP_REG => "addsp",
    }
}

pub fn port_op_name(op: PeekPokeOp) -> &'static str {
    match op { PeekPokeOp::POKE => "poke", PeekPokeOp::PEEK => "peek" }
}

pub fn x_op_name(op: XTypeOp) -> &'static str {
    match op {
        XTypeOp::SYSC => "sysc", XTypeOp::ERET => "eret", XTypeOp::HALT => "halt",
        XTypeOp::ICINV => "icinv", XTypeOp::DCINV => "dcinv", XTypeOp::DCCLEAN => "dcclean", XTypeOp::FLUSH => "flush",
    }
}
