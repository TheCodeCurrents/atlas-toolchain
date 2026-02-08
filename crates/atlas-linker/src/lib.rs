// TODO: add support for linking libs
// TODO: add support for multiple sections (code, data, bss) and section-specific linking

pub mod error;
pub mod linker;

pub use error::{LinkerError, LinkerErrorKind};
pub use linker::{LabelMap, Linker};

use std::fs::{self, File};
use std::io::Write;
use atlas_files::{ObjectFile, FileFormat};

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

        let obj_file = ObjectFile::from_file(obj_path).map_err(|e| {
            LinkerError::new(
                LinkerErrorKind::ObjectFile,
                format!("Failed to parse object file '{}': {}", obj_path, e),
                0,
                Some(obj_path.to_string()),
            )
        })?;

        // Register labels with their addresses
        for symbol in &obj_file.symbols {
            // Only register defined, local/global symbols (not imports)
            if symbol.section.is_none() {
                continue;
            }

            let symbol_address = symbol.value;
            let absolute_address = current_address + (symbol_address as u16);
            linker.register_label(symbol.name.clone(), absolute_address);
        }

        // TODO: Handle instructions and section data as appropriate for your format
        // This is a placeholder for further integration.
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