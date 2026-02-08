use std::collections::HashMap;
use atlas_isa::{Operand, ParsedInstruction};

use crate::error::{LinkerError, LinkerErrorKind};

/// Represents a label and its address in the output binary
#[derive(Debug, Clone)]
pub struct LabelMap {
    labels: HashMap<String, LabelInfo>,
}

#[derive(Debug, Clone)]
pub struct LabelInfo {
    pub address: u16,
    pub source_file: Option<String>,
}

impl LabelMap {
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
        }
    }

    /// Insert a label and its resolved address
    pub fn insert(&mut self, label: String, address: u16) {
        self.labels.insert(
            label,
            LabelInfo {
                address,
                source_file: None,
            },
        );
    }

    /// Insert a label with its resolved address and source file
    pub fn insert_with_source(&mut self, label: String, address: u16, source_file: String) {
        self.labels.insert(
            label,
            LabelInfo {
                address,
                source_file: Some(source_file),
            },
        );
    }

    /// Look up a label's address
    pub fn get(&self, label: &str) -> Option<u16> {
        self.labels.get(label).map(|info| info.address)
    }

    /// Look up full label metadata
    pub fn get_info(&self, label: &str) -> Option<&LabelInfo> {
        self.labels.get(label)
    }
}

pub struct Linker {
    pub label_map: LabelMap,
}

impl Linker {
    pub fn new() -> Self {
        Self {
            label_map: LabelMap::new(),
        }
    }

    /// Register a label with its resolved address
    pub fn register_label(&mut self, label: String, address: u16) {
        self.label_map.insert(label, address);
    }

    /// Register a label with its resolved address and source file
    pub fn register_label_with_source(&mut self, label: String, address: u16, source_file: String) {
        self.label_map.insert_with_source(label, address, source_file);
    }

    /// Resolve all label references in instructions to actual addresses
    /// This converts BranchOperand::Label to BranchOperand::Immediate with the resolved address
    pub fn resolve_labels(&self, instructions: Vec<ParsedInstruction>) -> Result<Vec<ParsedInstruction>, LinkerError> {
        instructions
            .into_iter()
            .map(|instr| self.resolve_instruction(instr))
            .collect()
    }

    /// Resolve labels in a single instruction
    fn resolve_instruction(&self, instr: ParsedInstruction) -> Result<ParsedInstruction, LinkerError> {
        match instr {
            ParsedInstruction::I { op, dest, immediate, line, source_file } => {
                let resolved = self.resolve_operand(&immediate, line, &source_file)?;
                Ok(ParsedInstruction::I {
                    op,
                    dest,
                    immediate: resolved,
                    line,
                    source_file,
                })
            }
            ParsedInstruction::BI { absolute, cond, operand, line, source_file } => {
                let resolved_operand = self.resolve_operand(&operand, line, &source_file)?;
                Ok(ParsedInstruction::BI {
                    absolute,
                    cond,
                    operand: resolved_operand,
                    line,
                    source_file,
                })
            }
            ParsedInstruction::P { op, register, offset, line, source_file } => {
                let resolved = self.resolve_operand(&offset, line, &source_file)?;
                Ok(ParsedInstruction::P {
                    op,
                    register,
                    offset: resolved,
                    line,
                    source_file,
                })
            }
            other => Ok(other),
        }
    }

    /// Resolve an `Operand` – if it's a label, look it up in the label map.
    fn resolve_operand(
        &self,
        operand: &Operand,
        line: usize,
        source_file: &Option<String>,
    ) -> Result<Operand, LinkerError> {
        match operand {
            Operand::Immediate(_) => Ok(operand.clone()),
            Operand::Label(label) => {
                let addr = self.label_map.get(label).ok_or_else(|| {
                    LinkerError::new(
                        LinkerErrorKind::UnresolvedLabel,
                        format!("Unresolved label: '{}'", label),
                        line,
                        source_file.clone(),
                    )
                })?;
                Ok(Operand::Immediate(addr))
            }
        }
    }
}