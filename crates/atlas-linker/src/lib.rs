// TODO: add support for linking libs

pub mod error;
pub mod linker;

pub use error::{LinkerError, LinkerErrorKind};
pub use linker::{LabelMap, Linker};

use std::collections::BTreeMap;
use atlas_files::{ObjectFile, FileFormat, SymbolBinding};

/// Link object files into a final executable binary.
///
/// The linker:
/// 1. Reads every object file.
/// 2. Concatenates same-named sections (e.g. all `.text` sections).
/// 3. Builds a global symbol table (adjusting symbol offsets to account for
///    section placement).
/// 4. Applies relocations – patching the raw bytes wherever an unresolved
///    label reference was left by the assembler.
/// 5. Writes the final flat binary to `output`.
pub fn link(object_files: &[&str], output: &str) -> Result<(), LinkerError> {
    let mut linker = Linker::new();

    // ── 1. Load all object files ───────────────────────────────────────
    let mut loaded: Vec<(String, ObjectFile)> = Vec::new();
    for obj_path in object_files {
        let obj_file = ObjectFile::from_file(obj_path).map_err(|e| {
            LinkerError::new(
                LinkerErrorKind::Io,
                format!("Failed to read/parse object file '{}': {}", obj_path, e),
                0,
                Some(obj_path.to_string()),
            )
        })?;
        loaded.push((obj_path.to_string(), obj_file));
    }

    // ── 2. Merge sections & build section-base-address map ─────────────
    // We merge all sections with the same name, appending data in input
    // order.  `section_bases` records, per (file-index, section-name), the
    // byte offset within the merged section where that file's contribution
    // starts.
    let mut merged_sections: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    // (file_idx, section_name) -> base offset within merged section
    let mut section_bases: BTreeMap<(usize, String), u32> = BTreeMap::new();

    for (file_idx, (_path, obj)) in loaded.iter().enumerate() {
        for section in &obj.sections {
            let merged = merged_sections.entry(section.name.clone()).or_default();
            let base = merged.len() as u32;
            section_bases.insert((file_idx, section.name.clone()), base);
            merged.extend_from_slice(&section.data);
        }
    }

    // ── 3. Build global symbol table ───────────────────────────────────
    for (file_idx, (path, obj)) in loaded.iter().enumerate() {
        for symbol in &obj.symbols {
            // Skip undefined / import symbols (section == None)
            let section_name = match &symbol.section {
                Some(s) => s.clone(),
                None => continue,
            };

            // Absolute constants (e.g. .imm values) are not relocated
            if section_name == ".abs" {
                linker.register_label(symbol.name.clone(), symbol.value as u16);
                continue;
            }

            let base = section_bases
                .get(&(file_idx, section_name.clone()))
                .copied()
                .unwrap_or(0);
            let absolute_address = base + symbol.value;

            // For global symbols, check for duplicates
            if matches!(symbol.binding, SymbolBinding::Global) {
                if let Some(existing) = linker.label_map.get(&symbol.name) {
                    return Err(LinkerError::new(
                        LinkerErrorKind::DuplicateSymbol,
                        format!(
                            "Duplicate global symbol '{}' (first defined at 0x{:04x}, also in '{}')",
                            symbol.name, existing, path
                        ),
                        0,
                        Some(path.clone()),
                    ));
                }
            }
            linker.register_label(symbol.name.clone(), absolute_address as u16);
        }
    }

    // ── 4. Apply relocations ───────────────────────────────────────────
    for (file_idx, (path, obj)) in loaded.iter().enumerate() {
        for reloc in &obj.relocations {
            let section_name = &reloc.section;
            let base = section_bases
                .get(&(file_idx, section_name.clone()))
                .copied()
                .unwrap_or(0);
            let patch_offset = (base + reloc.offset) as usize;

            // Resolve the symbol
            let symbol_value = linker.label_map.get(&reloc.symbol).ok_or_else(|| {
                LinkerError::new(
                    LinkerErrorKind::UnresolvedLabel,
                    format!("Unresolved symbol '{}' referenced in '{}'", reloc.symbol, path),
                    0,
                    Some(path.clone()),
                )
            })?;

            let final_value = (symbol_value as i32 + reloc.addend) as u16;

            // Patch the merged section data.
            // Instructions are 16-bit big-endian.  The relocation offset
            // points to the start of the 2-byte instruction.  We need to
            // patch the lower 8 bits of the instruction word (the
            // immediate/address field) while keeping the upper bits
            // (opcode, etc.) intact.
            let section_data = merged_sections.get_mut(section_name).ok_or_else(|| {
                LinkerError::new(
                    LinkerErrorKind::ObjectFile,
                    format!("Section '{}' not found for relocation", section_name),
                    0,
                    Some(path.clone()),
                )
            })?;

            if patch_offset + 1 >= section_data.len() {
                return Err(LinkerError::new(
                    LinkerErrorKind::ObjectFile,
                    format!(
                        "Relocation offset 0x{:x} out of bounds for section '{}' (size {})",
                        patch_offset,
                        section_name,
                        section_data.len()
                    ),
                    0,
                    Some(path.clone()),
                ));
            }

            // Read current instruction word (big-endian)
            let hi = section_data[patch_offset];
            let _lo = section_data[patch_offset + 1];

            // Keep the upper byte (opcode + flags) and replace the lower
            // byte with the resolved address/immediate.
            // This works for I-type (imm in [7:0]), BI-type (addr in
            // [7:0]), and P-type (offset in [7:0]).
            if final_value > 0xFF {
                return Err(LinkerError::new(
                    LinkerErrorKind::Encoding,
                    format!(
                        "Resolved value 0x{:04x} for symbol '{}' exceeds 8-bit immediate field",
                        final_value, reloc.symbol
                    ),
                    0,
                    Some(path.clone()),
                ));
            }

            section_data[patch_offset] = hi;
            section_data[patch_offset + 1] = final_value as u8;
        }
    }

    // ── 5. Write output ──────────────────────────────────────────────
    // Output sections in a deterministic order: .text first, then the rest.
    let mut output_bytes: Vec<u8> = Vec::new();
    if let Some(text) = merged_sections.get(".text") {
        output_bytes.extend_from_slice(text);
    }
    for (name, data) in &merged_sections {
        if name == ".text" {
            continue;
        }
        output_bytes.extend_from_slice(data);
    }

    // Choose format based on file extension
    let write_result = if output.ends_with(".hex") {
        atlas_files::hex::write_hex_file(output, &output_bytes, 0x0000)
    } else {
        // Raw binary (default for .bin or any other extension)
        use std::io::Write;
        std::fs::File::create(output).and_then(|mut f| f.write_all(&output_bytes))
    };

    write_result.map_err(|e| {
        LinkerError::new(
            LinkerErrorKind::Io,
            format!("Failed to write output file '{}': {}", output, e),
            0,
            Some(output.to_string()),
        )
    })?;

    Ok(())
}