
use atlas_files::{ObjectFile, SymbolBinding};

pub fn inspect_obj(obj: &ObjectFile) {
    println!("Object file inspection");
    println!("=======================");
    println!("Version: {}\n", obj.version);

    // Sections
    println!("Sections:");
    for (i, sec) in obj.sections.iter().enumerate() {
        println!(
            "  [{:2}] {:<16} start=0x{:08x} size={} bytes",
            i,
            sec.name,
            sec.start,
            sec.data.len()
        );
    }

    // Symbols
    println!("\nSymbols:");
    for sym in &obj.symbols {
        let section = sym.section.as_deref().unwrap_or("UND");
        let binding = match sym.binding {
            SymbolBinding::Local => "LOCAL",
            SymbolBinding::Global => "GLOBAL",
        };

        println!(
            "  {:<20} {:<6} {:<8} value=0x{:08x}",
            sym.name,
            binding,
            section,
            sym.value
        );
    }

    // Relocations
    println!("\nRelocations:");
    for rel in &obj.relocations {
        println!(
            "  offset=0x{:08x} symbol={:<20} addend={}",
            rel.offset,
            rel.symbol,
            rel.addend
        );
    }
}
