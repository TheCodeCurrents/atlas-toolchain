pub mod lexer;
pub mod parser;
pub mod error;

pub use lexer::Lexer;
pub use parser::Parser;
pub use error::AssemblerError;

use atlas_isa::{EncodingError, ObjectFile, Symbol, SymbolKind};
use std::fs::{self, File};
use std::io::Write;

/// Assemble source file into an object file (.o format)
/// The object file contains unresolved instructions that will be linked later
pub fn assemble(src: &str, output: &str) -> Result<(), AssemblerError> {
    let source = fs::read_to_string(src).map_err(|e| AssemblerError::IoError {
        operation: format!("Failed to read input file '{}'", src),
        source: e,
    })?;
    
    // Create parser (which includes lexer)
    let mut parser = Parser::new(&source);
    
    // Collect all parsed instructions (labels may be unresolved)
    let mut instructions = Vec::new();
    for result in &mut parser {
        let instr = result?;
        instructions.push(instr.with_source_file(Some(src.to_string())));
    }

    // Collect symbols from parser
    let mut symbols = Vec::new();
    for (name, symbol) in parser.symbols().iter() {
        match symbol {
            crate::parser::symbols::Symbol::Label(address) => {
                let address = u8::try_from(*address).map_err(|_| {
                    AssemblerError::EncodingError(EncodingError {
                        line: 0,
                        message: format!(
                            "Symbol address out of range for '{}': {}",
                            name, address
                        ),
                    })
                })?;
                symbols.push(Symbol {
                    name: name.clone(),
                    address,
                    kind: SymbolKind::Local,
                });
            }
            crate::parser::symbols::Symbol::External => {
                symbols.push(Symbol {
                    name: name.clone(),
                    address: 0,
                    kind: SymbolKind::Import,
                });
            }
        }
    }
    
    // Create object file with unresolved instructions
    let mut object_file = ObjectFile::with_instructions(instructions);
    object_file.symbols = symbols;
    
    // Serialize object file to bytes
    let bytes = object_file.to_bytes().map_err(|e| AssemblerError::EncodingError(
        atlas_isa::EncodingError {
            line: 0,
            message: format!("Failed to serialize object file: {}", e),
        }
    ))?;
    
    // Write object file
    let mut file = File::create(output).map_err(|e| AssemblerError::IoError {
        operation: format!("Failed to create output file '{}'", output),
        source: e,
    })?;
    file.write_all(&bytes).map_err(|e| AssemblerError::IoError {
        operation: format!("Failed to write to output file '{}'", output),
        source: e,
    })?;
    
    Ok(())
}