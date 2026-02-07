pub mod args;

use args::Arguments;
use clap::Parser;

use crate::args::Command;
use atlas_isa::{BranchCond, MOffset, Operand, ParsedInstruction, XOperand};
use atlas_files::{ObjectFile, SymbolKind};

fn print_hex_dump(bytes: &[u8]) {
    let mut offset = 0usize;
    while offset < bytes.len() {
        let end = usize::min(offset + 16, bytes.len());
        let chunk = &bytes[offset..end];

        print!("{:08x}  ", offset);
        for i in 0..16 {
            if i < chunk.len() {
                print!("{:02x} ", chunk[i]);
            } else {
                print!("   ");
            }
        }
        print!(" |");
        for &b in chunk {
            let ch = if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' };
            print!("{}", ch);
        }
        println!("|");

        offset += 16;
    }
}

fn branch_mnemonic(cond: BranchCond) -> &'static str {
    match cond {
        BranchCond::Unconditional => "br",
        BranchCond::EQ => "beq",
        BranchCond::NE => "bne",
        BranchCond::CS => "bcs",
        BranchCond::CC => "bcc",
        BranchCond::MI => "bmi",
        BranchCond::PL => "bpl",
    }
}

fn format_instruction(instr: &ParsedInstruction) -> String {
    match instr {
        ParsedInstruction::A { op, dest, source, .. } => {
            format!("{} r{}, r{}", format!("{:?}", op).to_lowercase(), dest, source)
        }
        ParsedInstruction::I { op, dest, immediate, .. } => {
            let imm_str = match immediate {
                Operand::Immediate(val) => format!("0x{:04x}", val),
                Operand::Label(name) => name.clone(),
            };
            format!(
                "{} r{}, {}",
                format!("{:?}", op).to_lowercase(),
                dest,
                imm_str
            )
        }
        ParsedInstruction::M { op, dest, base, offset, .. } => {
            let off_str = match offset {
                MOffset::Offset8(val) => format!("0x{:02x}", val),
                MOffset::SR(reg) => format!("r{}", reg),
            };
            format!(
                "{} r{}, [r{} + {}]",
                format!("{:?}", op).to_lowercase(),
                dest,
                base,
                off_str
            )
        }
        ParsedInstruction::BI { absolute, cond, operand, .. } => {
            let op_str = branch_mnemonic(*cond);
            let target = match operand {
                Operand::Immediate(addr) => format!("0x{:04x}", addr),
                Operand::Label(name) => name.clone(),
            };
            let mode = if *absolute { "abs" } else { "rel" };
            format!("{} {} ({})", op_str, target, mode)
        }
        ParsedInstruction::BR { absolute, cond, source, .. } => {
            let op_str = branch_mnemonic(*cond);
            let mode = if *absolute { "abs" } else { "rel" };
            format!("{} r{}, r{} ({})", op_str, source.high, source.low, mode)
        }
        ParsedInstruction::S { op, register, .. } => {
            format!("{} r{}", format!("{:?}", op).to_lowercase(), register)
        }
        ParsedInstruction::P { op, register, offset, .. } => {
            let off_str = match offset {
                Operand::Immediate(val) => format!("0x{:04x}", val),
                Operand::Label(name) => name.clone(),
            };
            format!(
                "{} r{}, {}",
                format!("{:?}", op).to_lowercase(),
                register,
                off_str
            )
        }
        ParsedInstruction::X { op, operand, .. } => {
            let op_str = format!("{:?}", op).to_lowercase();
            match operand {
                XOperand::None => op_str,
                XOperand::Immediate(imm) => format!("{} 0x{:02x}", op_str, imm),
                XOperand::Register(reg) => format!("{} r{}", op_str, reg),
                XOperand::Registers(r1, r2) => format!("{} r{}, r{}", op_str, r1, r2),
            }
        }
    }
}

fn print_disassembly(bytes: &[u8]) {
    if bytes.len() % 2 != 0 {
        eprintln!("Warning: output size is not aligned to 16-bit instructions.");
    }

    for (index, chunk) in bytes.chunks(2).enumerate() {
        if chunk.len() < 2 {
            break;
        }
        let encoded = u16::from_be_bytes([chunk[0], chunk[1]]);
        let addr = (index * 2) as u16;
        match ParsedInstruction::decode(encoded) {
            Ok(instr) => {
                println!("{:04x}: {:04x}  {}", addr, encoded, format_instruction(&instr));
            }
            Err(err) => {
                println!("{:04x}: {:04x}  <decode error: {}>", addr, encoded, err);
            }
        }
    }
}

fn print_object_file(obj: &ObjectFile) {
    eprintln!(
        "Object file: {} instructions, {} symbols",
        obj.instructions.len(),
        obj.symbols.len()
    );

    if !obj.symbols.is_empty() {
        eprintln!("Symbols:");
        for symbol in &obj.symbols {
            let kind = match symbol.kind {
                SymbolKind::Local => "local",
                SymbolKind::Export => "export",
                SymbolKind::Import => "import",
                SymbolKind::Constant => "const",
            };
            let addr = match symbol.address {
                Some(value) => format!("0x{:04x}", value),
                None => "None".to_string(),
            };
            eprintln!("  {:<6} {:<20} {}", kind, symbol.name, addr);
        }
    }

    eprintln!("Instructions:");
    for (index, instr) in obj.instructions.iter().enumerate() {
        let addr = (index * 2) as u16;
        let line = instr.line();
        println!("{:04x}: {:<28} ; line {}", addr, format_instruction(instr), line);
    }
}

fn main() {
    let args = Arguments::parse();

    if args.verbose {
        eprintln!("Verbose mode enabled");
    }

    let is_link = matches!(&args.command, Command::Ld { .. });

    let output_path = match &args.command {
        Command::Asm { output, .. } => output.clone(),
        Command::Ld { output, .. } => output.clone(),
        Command::Inspect { .. } => {
            eprintln!("Inspect command does not produce an output file to read for verbose mode.");
            std::process::exit(1);
        }
    };

    let result = match args.command {
        Command::Asm { input, output } => {
            if args.verbose {
                eprintln!("Assembling {} -> {}", input, output);
            }
            atlas_assembler::assemble(&input, &output)
                .map_err(|e| format!("{}", e))
        },
        Command::Ld { inputs, output } => {
            if args.verbose {
                eprintln!("Linking {:?} -> {}", inputs, output);
            }
            let input_refs: Vec<&str> = inputs.iter().map(|s| s.as_str()).collect();
            atlas_linker::link(&input_refs, &output)
                .map_err(|e| format!("{}", e))
        },
        Command::Inspect { .. } => {
            eprintln!("Inspect command is not implemented yet.");
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    if args.verbose {
        match std::fs::read(&output_path) {
            Ok(bytes) => {
                if is_link {
                    eprintln!("Disassembly of {}:", output_path);
                    print_disassembly(&bytes);
                } else {
                    match ObjectFile::from_bytes(&bytes) {
                        Ok(obj) => {
                            eprintln!("Object listing of {}:", output_path);
                            print_object_file(&obj);
                        }
                        Err(_) => {
                            eprintln!("Hex dump of {}:", output_path);
                            print_hex_dump(&bytes);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read output for verbose dump: {}", e);
            }
        }
    }
}