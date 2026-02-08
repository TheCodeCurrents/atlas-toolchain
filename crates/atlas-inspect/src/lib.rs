
use atlas_files::{ObjectFile, SymbolBinding};
use atlas_isa::ParsedInstruction;
use atlas_isa::opcode::*;
use atlas_isa::operands::*;
use std::collections::BTreeMap;

// ── Colours / style helpers ────────────────────────────────────────────────
// Respects NO_COLOR (https://no-color.org/).
fn use_colour() -> bool {
    std::env::var_os("NO_COLOR").is_none()
}

fn dim(s: &str) -> String {
    if use_colour() { format!("\x1b[2m{}\x1b[0m", s) } else { s.to_string() }
}
fn bold(s: &str) -> String {
    if use_colour() { format!("\x1b[1m{}\x1b[0m", s) } else { s.to_string() }
}
fn green(s: &str) -> String {
    if use_colour() { format!("\x1b[32m{}\x1b[0m", s) } else { s.to_string() }
}
fn cyan(s: &str) -> String {
    if use_colour() { format!("\x1b[36m{}\x1b[0m", s) } else { s.to_string() }
}
fn yellow(s: &str) -> String {
    if use_colour() { format!("\x1b[33m{}\x1b[0m", s) } else { s.to_string() }
}

// ── Summary (non-verbose) ──────────────────────────────────────────────────

/// Print a single-line summary after assembly.
pub fn print_asm_summary(input: &str, output: &str, obj: &ObjectFile) {
    let total_bytes: usize = obj.sections.iter().map(|s| s.data.len()).sum();
    let sym_count = obj.symbols.len();
    let reloc_count = obj.relocations.len();
    println!(
        "  {} {} → {} {}",
        green("Assembled"),
        bold(input),
        bold(output),
        dim(&format!("({} bytes, {} symbols, {} relocations)", total_bytes, sym_count, reloc_count)),
    );
}

/// Print a single-line summary after linking.
pub fn print_link_summary(inputs: &[String], output: &str, byte_count: usize) {
    let joined = inputs.iter()
        .map(|s| bold(s))
        .collect::<Vec<_>>()
        .join(&dim(" + "));
    println!(
        "     {} {} → {} {}",
        green("Linked"),
        joined,
        bold(output),
        dim(&format!("({} bytes)", byte_count)),
    );
}

// ── Verbose: object file details ───────────────────────────────────────────

/// Print detailed object file information (verbose mode).
pub fn inspect_obj(obj: &ObjectFile) {
    // Sections
    println!("  {}",bold("Sections:"));
    for sec in &obj.sections {
        println!(
            "    {:<16} {} bytes",
            cyan(&sec.name),
            sec.data.len(),
        );
    }

    // Symbols – sorted: globals first, then locals, alphabetical within each
    let mut syms: Vec<_> = obj.symbols.iter().collect();
    syms.sort_by(|a, b| {
        let a_global = matches!(a.binding, SymbolBinding::Global);
        let b_global = matches!(b.binding, SymbolBinding::Global);
        b_global.cmp(&a_global).then_with(|| a.name.cmp(&b.name))
    });

    println!("\n  {}", bold("Symbols:"));
    for sym in &syms {
        let binding = match sym.binding {
            SymbolBinding::Local => dim("local "),
            SymbolBinding::Global => yellow("global"),
        };
        let section = sym.section.as_deref().unwrap_or("UND");
        let value = if sym.section.is_some() {
            format!("0x{:04x}", sym.value)
        } else {
            String::new()
        };
        println!(
            "    {} {:<20} {:<8} {}",
            binding,
            sym.name,
            cyan(section),
            dim(&value),
        );
    }

    // Relocations
    println!("\n  {}", bold("Relocations:"));
    if obj.relocations.is_empty() {
        println!("    {}", dim("(none)"));
    }
    for rel in &obj.relocations {
        let addend_str = if rel.addend != 0 {
            format!("{:+}", rel.addend)
        } else {
            String::new()
        };
        println!(
            "    {}+0x{:04x} → {}{}",
            cyan(&rel.section),
            rel.offset,
            bold(&rel.symbol),
            addend_str,
        );
    }
}

// ── Verbose: label map ─────────────────────────────────────────────────────

/// Build an address→label map from an object file's symbols.
pub fn build_label_map(obj: &ObjectFile) -> BTreeMap<u16, String> {
    let mut map = BTreeMap::new();
    for sym in &obj.symbols {
        if sym.section.is_some() {
            map.insert(sym.value as u16, sym.name.clone());
        }
    }
    map
}

// ── Verbose: disassembly ───────────────────────────────────────────────────

/// Disassemble raw bytes (big-endian 16-bit instruction words) and print them
/// in a human-readable format.
pub fn disassemble(data: &[u8], labels: &BTreeMap<u16, String>) {
    println!("  {}", bold("Disassembly of .text:"));
    if data.len() % 2 != 0 {
        println!("    {} data length ({}) is not a multiple of 2", yellow("warning:"), data.len());
    }

    for offset in (0..data.len()).step_by(2) {
        let addr = offset as u16;

        // Print label if one exists at this address
        if let Some(name) = labels.get(&addr) {
            if offset > 0 { println!(); }
            println!("  {}:", bold(name));
        }

        if offset + 1 >= data.len() {
            println!(
                "    {} {}  .byte 0x{:02x}",
                dim(&format!("{:04x}:", addr)),
                dim(&format!("{:02x}", data[offset])),
                data[offset]
            );
            break;
        }

        let word = ((data[offset] as u16) << 8) | (data[offset + 1] as u16);

        let disasm = match ParsedInstruction::decode(word) {
            Ok(instr) => format_instruction(&instr, labels),
            Err(_) => format!(".word 0x{:04x}", word),
        };

        println!(
            "    {} {}  {}",
            dim(&format!("{:04x}:", addr)),
            dim(&format!("{:04x}", word)),
            disasm,
        );
    }
}

// ── Instruction formatting ─────────────────────────────────────────────────

fn format_instruction(instr: &ParsedInstruction, labels: &BTreeMap<u16, String>) -> String {
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

fn format_operand(op: &Operand, labels: &BTreeMap<u16, String>) -> String {
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

fn reg_name(r: u8) -> &'static str {
    match r {
        0 => "r0", 1 => "r1", 2 => "r2", 3 => "r3",
        4 => "r4", 5 => "r5", 6 => "r6", 7 => "r7",
        8 => "r8", 9 => "r9", 10 => "tr", 11 => "r11",
        12 => "sp", 13 => "r13", 14 => "pc", 15 => "r15",
        _ => "r?",
    }
}

fn alu_op_name(op: AluOp) -> &'static str {
    match op {
        AluOp::ADD => "add", AluOp::ADDC => "addc", AluOp::SUB => "sub", AluOp::SUBC => "subc",
        AluOp::AND => "and", AluOp::OR => "or", AluOp::XOR => "xor", AluOp::NOT => "not",
        AluOp::SHL => "shl", AluOp::SHR => "shr", AluOp::ROL => "rol", AluOp::ROR => "ror",
        AluOp::CMP => "cmp", AluOp::TST => "tst", AluOp::MOV => "mov", AluOp::NEG => "neg",
    }
}

fn imm_op_name(op: ImmOp) -> &'static str {
    match op { ImmOp::LDI => "ldi", ImmOp::ADDI => "addi", ImmOp::SUBI => "subi", ImmOp::ANDI => "andi", ImmOp::ORI => "ori" }
}

fn mem_op_name(op: MemOp) -> &'static str {
    match op { MemOp::LD => "ld", MemOp::ST => "st" }
}

fn branch_cond_name(cond: BranchCond) -> &'static str {
    match cond {
        BranchCond::Unconditional => "br", BranchCond::EQ => "beq", BranchCond::NE => "bne",
        BranchCond::CS => "bcs", BranchCond::CC => "bcc", BranchCond::MI => "bmi", BranchCond::PL => "bpl",
        BranchCond::OV => "bov",
    }
}

fn stack_op_name(op: StackOp) -> &'static str {
    match op {
        StackOp::PUSH => "push", StackOp::POP => "pop",
        StackOp::SUBSP_IMM | StackOp::SUBSP_REG => "subsp",
        StackOp::ADDSP_IMM | StackOp::ADDSP_REG => "addsp",
    }
}

fn port_op_name(op: PeekPokeOp) -> &'static str {
    match op { PeekPokeOp::POKE => "poke", PeekPokeOp::PEEK => "peek" }
}

fn x_op_name(op: XTypeOp) -> &'static str {
    match op {
        XTypeOp::SYSC => "sysc", XTypeOp::ERET => "eret", XTypeOp::HALT => "halt",
        XTypeOp::ICINV => "icinv", XTypeOp::DCINV => "dcinv", XTypeOp::DCCLEAN => "dcclean", XTypeOp::FLUSH => "flush",
    }
}
