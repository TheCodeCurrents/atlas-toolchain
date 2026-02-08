use atlas_isa::ParsedInstruction;
use std::collections::BTreeMap;
use crate::formatting::{format_instruction, dim, bold, yellow};

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
