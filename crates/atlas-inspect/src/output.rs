use atlas_files::{ObjectFile, SymbolBinding};
use std::collections::BTreeMap;
use crate::formatting::{bold, dim, green, cyan, yellow};

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
