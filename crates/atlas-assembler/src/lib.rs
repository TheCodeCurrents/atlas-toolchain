pub mod lexer;
pub mod parser;
pub mod error;

pub use lexer::Lexer;
pub use parser::Parser;
pub use error::AssemblerError;

use atlas_isa::EncodingError;
use atlas_isa::operands::Operand;
use atlas_isa::ParsedInstruction;
use atlas_files::{ObjectFile, Symbol, SymbolBinding, FileFormat};
use atlas_files::formats::obj::{Section, Relocation};
use std::collections::BTreeMap;
use std::fs;
use crate::parser::ParsedItem;
use crate::parser::symbols::UnresolvedReference;


/// Try to encode an instruction. If it contains an unresolved label, emit a
/// placeholder (all-zero) encoding and return the label name so the caller can
/// record a relocation.
fn encode_or_placeholder(instr: &ParsedInstruction) -> Result<(u16, Option<String>), atlas_isa::EncodingError> {
    // Check if the instruction contains a label operand.  If so, substitute
    // Immediate(0) so the encoder succeeds and return the label name.
    match instr {
        ParsedInstruction::I { op, dest, immediate: Operand::Label(label), line, source_file } => {
            let placeholder = ParsedInstruction::I {
                op: *op,
                dest: *dest,
                immediate: Operand::Immediate(0),
                line: *line,
                source_file: source_file.clone(),
            };
            Ok((placeholder.encode()?, Some(label.clone())))
        }
        ParsedInstruction::BI { absolute, cond, operand: Operand::Label(label), line, source_file } => {
            let placeholder = ParsedInstruction::BI {
                absolute: *absolute,
                cond: *cond,
                operand: Operand::Immediate(0),
                line: *line,
                source_file: source_file.clone(),
            };
            Ok((placeholder.encode()?, Some(label.clone())))
        }
        ParsedInstruction::P { op, register, offset: Operand::Label(label), line, source_file } => {
            let placeholder = ParsedInstruction::P {
                op: *op,
                register: *register,
                offset: Operand::Immediate(0),
                line: *line,
                source_file: source_file.clone(),
            };
            Ok((placeholder.encode()?, Some(label.clone())))
        }
        other => Ok((other.encode()?, None)),
    }
}


/// Assemble source file into an object file (.o format)
/// The object file contains unresolved instructions that will be linked later
pub fn assemble(src: &str, output: &str) -> Result<(), AssemblerError> {
    let source = fs::read_to_string(src).map_err(|e| AssemblerError::IoError {
        operation: format!("Failed to read input file '{}'", src),
        source: e,
    })?;
    
    // ── Pass 1: parse everything, collect items & symbols ──────────────
    let mut parser = Parser::new(&source);
    
    // Collect all parsed items first (resolves the borrow issue)
    let mut items: Vec<ParsedItem> = Vec::new();
    for result in &mut parser {
        items.push(result?);
    }

    // Now we can freely access parser.symbols()
    let symbols_table = parser.symbols().clone();

    // ── Pass 2: encode items into section data ─────────────────────────
    let mut section_data: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut current_section = ".text".to_string();
    let mut unresolved: Vec<UnresolvedReference> = Vec::new();

    for item in items {
        match item {
            ParsedItem::SectionChange(name) => {
                current_section = name;
                section_data.entry(current_section.clone()).or_default();
            }
            ParsedItem::Instruction(instr) => {
                let instr = instr.with_source_file(Some(src.to_string()));
                let data = section_data.entry(current_section.clone()).or_default();
                let byte_offset = data.len() as u32;

                // Try to resolve local constants/labels inline first.
                let instr = resolve_local_operands(&instr, &symbols_table);

                let (encoded, maybe_label) = encode_or_placeholder(&instr)
                    .map_err(|e| AssemblerError::EncodingError(e))?;

                if let Some(label_name) = maybe_label {
                    unresolved.push(UnresolvedReference {
                        offset: byte_offset,
                        section: current_section.clone(),
                        symbol: label_name,
                        addend: 0,
                    });
                }

                data.push((encoded >> 8) as u8);
                data.push(encoded as u8);
            }
            ParsedItem::Data(bytes) => {
                let data = section_data.entry(current_section.clone()).or_default();
                data.extend_from_slice(&bytes);
            }
        }
    }

    // ── Build section list ─────────────────────────────────────────────
    let mut sections = Vec::new();
    for (name, data) in &section_data {
        sections.push(Section {
            name: name.clone(),
            start: 0,
            data: data.clone(),
        });
    }

    // ── Build symbol list ──────────────────────────────────────────────
    let mut symbols = Vec::new();

    // Defined symbols (labels & constants)
    for (name, symbol) in symbols_table.iter() {
        match symbol {
            crate::parser::symbols::Symbol::Label { offset, section } => {
                let binding = if symbols_table.is_exported(name) {
                    SymbolBinding::Global
                } else {
                    SymbolBinding::Local
                };
                symbols.push(Symbol {
                    name: name.clone(),
                    value: *offset,
                    section: Some(section.clone()),
                    binding,
                });
            }
            crate::parser::symbols::Symbol::Constant(value) => {
                let binding = if symbols_table.is_exported(name) {
                    SymbolBinding::Global
                } else {
                    SymbolBinding::Local
                };
                symbols.push(Symbol {
                    name: name.clone(),
                    value: u32::from(*value),
                    section: Some(".abs".to_string()),
                    binding,
                });
            }
        }
    }

    // Imported (undefined) symbols – section = None
    for import_name in symbols_table.imports() {
        // Only add if not already defined locally
        if symbols_table.resolve(import_name).is_none() {
            symbols.push(Symbol {
                name: import_name.clone(),
                value: 0,
                section: None,
                binding: SymbolBinding::Global,
            });
        }
    }

    // Validate exports
    for export in symbols_table.exports() {
        if symbols_table.resolve(export).is_none() {
            return Err(AssemblerError::EncodingError(EncodingError {
                line: 0,
                message: format!("Exported symbol '{}' is not defined", export),
            }));
        }
    }

    // ── Build relocation list ──────────────────────────────────────────
    // Only keep relocations for symbols that are NOT fully resolved locally.
    // A locally-resolved constant was already patched inline by
    // `resolve_local_operands`, so we don't need a relocation for it.
    let mut relocations = Vec::new();
    for uref in &unresolved {
        relocations.push(Relocation {
            offset: uref.offset,
            symbol: uref.symbol.clone(),
            addend: uref.addend,
            section: uref.section.clone(),
        });
    }

    // ── Write object file ──────────────────────────────────────────────
    let object_file = ObjectFile {
        sections,
        symbols,
        relocations,
        version: 1,
    };

    object_file.to_file(output).map_err(|e| AssemblerError::IoError {
        operation: format!("Failed to write to output file '{}'", output),
        source: e,
    })?;

    Ok(())
}

/// Try to resolve label operands that refer to locally-defined constants or
/// labels.  Returns the instruction unchanged if the operand is already
/// resolved or refers to an unknown (imported) symbol.
fn resolve_local_operands(
    instr: &ParsedInstruction,
    symbols: &crate::parser::symbols::SymbolTable,
) -> ParsedInstruction {
    match instr {
        ParsedInstruction::I { op, dest, immediate: Operand::Label(name), line, source_file } => {
            if let Some(sym) = symbols.resolve(name) {
                let value = match sym {
                    crate::parser::symbols::Symbol::Constant(v) => *v as u16,
                    crate::parser::symbols::Symbol::Label { offset, .. } => *offset as u16,
                };
                ParsedInstruction::I {
                    op: *op,
                    dest: *dest,
                    immediate: Operand::Immediate(value),
                    line: *line,
                    source_file: source_file.clone(),
                }
            } else {
                instr.clone()
            }
        }
        ParsedInstruction::BI { absolute, cond, operand: Operand::Label(name), line, source_file } => {
            if let Some(sym) = symbols.resolve(name) {
                let value = match sym {
                    crate::parser::symbols::Symbol::Constant(v) => *v as u16,
                    crate::parser::symbols::Symbol::Label { offset, .. } => *offset as u16,
                };
                ParsedInstruction::BI {
                    absolute: *absolute,
                    cond: *cond,
                    operand: Operand::Immediate(value),
                    line: *line,
                    source_file: source_file.clone(),
                }
            } else {
                instr.clone()
            }
        }
        ParsedInstruction::P { op, register, offset: Operand::Label(name), line, source_file } => {
            if let Some(sym) = symbols.resolve(name) {
                let value = match sym {
                    crate::parser::symbols::Symbol::Constant(v) => *v as u16,
                    crate::parser::symbols::Symbol::Label { offset, .. } => *offset as u16,
                };
                ParsedInstruction::P {
                    op: *op,
                    register: *register,
                    offset: Operand::Immediate(value),
                    line: *line,
                    source_file: source_file.clone(),
                }
            } else {
                instr.clone()
            }
        }
        _ => instr.clone(),
    }
}