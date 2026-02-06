pub mod error;
pub mod linker;

pub use error::{LinkerError, LinkerErrorKind};
pub use linker::{LabelMap, Linker};

use std::fs::{self, File};
use std::io::Write;
use atlas_isa::{ObjectFile, SymbolKind};

/// Link object files into a final executable binary
pub fn link(object_files: &[&str], output: &str) -> Result<(), LinkerError> {
    // Read all object files
    let mut all_instructions = Vec::new();
    let mut linker = Linker::new();
    let mut current_address = 0u16;
    
    for obj_path in object_files {
        let bytes = fs::read(obj_path).map_err(|e| {
            LinkerError::new(
                LinkerErrorKind::Io,
                format!("Failed to read object file '{}': {}", obj_path, e),
                0,
                Some(obj_path.to_string()),
            )
        })?;
        
        let obj_file = ObjectFile::from_bytes(&bytes).map_err(|e| {
            LinkerError::new(
                LinkerErrorKind::ObjectFile,
                format!("Failed to parse object file '{}': {}", obj_path, e),
                0,
                Some(obj_path.to_string()),
            )
        })?;
        
        // Register labels with their addresses
        for symbol in &obj_file.symbols {
            if matches!(symbol.kind, SymbolKind::Import) {
                continue;
            }

            let symbol_address = symbol.address.ok_or_else(|| {
                LinkerError::new(
                    LinkerErrorKind::ObjectFile,
                    format!("Symbol '{}' is missing an address", symbol.name),
                    0,
                    Some(obj_path.to_string()),
                )
            })?;

            let absolute_address = current_address + symbol_address;
            linker.register_label(symbol.name.clone(), absolute_address);
        }

        for instr in &obj_file.instructions {
            let mut instruction = instr.clone();
            if instruction.source_file().is_none() {
                instruction = instruction.with_source_file(Some(obj_path.to_string()));
            }
            all_instructions.push(instruction);
        }

        current_address = current_address.saturating_add((obj_file.instructions.len() * 2) as u16);
    }
    
    // Resolve all label references
    let resolved_instructions = linker.resolve_labels(all_instructions)?;
    
    // Encode all instructions to binary
    let mut bytes = Vec::new();
    for instr in resolved_instructions {
        let encoded = instr.encode().map_err(|e| {
            LinkerError::new(
                LinkerErrorKind::Encoding,
                e.message.clone(),
                e.line,
                instr.source_file().map(|s| s.to_string()),
            )
        })?;
        bytes.extend_from_slice(&encoded.to_be_bytes());
    }
    
    // Write binary output
    let mut file = File::create(output).map_err(|e| {
        LinkerError::new(
            LinkerErrorKind::Io,
            format!("Failed to create output file '{}': {}", output, e),
            0,
            Some(output.to_string()),
        )
    })?;
    file.write_all(&bytes).map_err(|e| {
        LinkerError::new(
            LinkerErrorKind::Io,
            format!("Failed to write to output file '{}': {}", output, e),
            0,
            Some(output.to_string()),
        )
    })?;
    
    Ok(())
}