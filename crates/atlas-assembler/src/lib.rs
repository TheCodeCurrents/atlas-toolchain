pub mod lexer;
pub mod parser;
pub mod error;

pub use lexer::Lexer;
pub use parser::Parser;
pub use error::AssemblerError;

use atlas_isa::EncodingError;
use atlas_files::{ObjectFile, Symbol, SymbolBinding, FileFormat};
use std::fs::{self, File};


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
                let value = u32::try_from(*address).map_err(|_| {
                    AssemblerError::EncodingError(EncodingError {
                        line: 0,
                        message: format!(
                            "Symbol address out of range for '{}': {}",
                            name, address
                        ),
                    })
                })?;
                let binding = if parser.symbols().is_exported(name) {
                    SymbolBinding::Global
                } else {
                    SymbolBinding::Local
                };
                symbols.push(Symbol {
                    name: name.clone(),
                    value,
                    section: Some(".text".to_string()), // or appropriate section
                    binding,
                });
            }
            crate::parser::symbols::Symbol::External => {
                symbols.push(Symbol {
                    name: name.clone(),
                    value: 0,
                    section: None,
                    binding: SymbolBinding::Global,
                });
            }
            crate::parser::symbols::Symbol::Constant(value) => {
                let binding = if parser.symbols().is_exported(name) {
                    SymbolBinding::Global
                } else {
                    SymbolBinding::Local
                };
                symbols.push(Symbol {
                    name: name.clone(),
                    value: u32::from(*value),
                    section: Some(".const".to_string()), // or appropriate section
                    binding,
                });
            }
        }
    }

    for export in parser.symbols().exports() {
        if parser.symbols().resolve(export).is_none() {
            return Err(AssemblerError::EncodingError(EncodingError {
                line: 0,
                message: format!("Exported symbol '{}' is not defined", export),
            }));
        }
    }

    // Create object file with unresolved instructions
    let object_file = ObjectFile {
        sections: vec![], // TODO: fill with actual sections
        symbols,
        relocations: vec![], // TODO: fill with actual relocations
        version: 1,
    };

    // Serialize object file to file
    object_file.to_file(output).map_err(|e| AssemblerError::IoError {
        operation: format!("Failed to write to output file '{}'", output),
        source: e,
    })?;

    Ok(())
}