pub mod args;

use args::Arguments;
use clap::Parser;

use crate::args::Command;
use atlas_files::{ObjectFile, FileFormat};
use atlas_inspect::{inspect_obj, disassemble, build_label_map, print_asm_summary, print_link_summary};
use std::collections::BTreeMap;

fn main() {
    let args = Arguments::parse();

    let result = match args.command {
        Command::Asm { input, output } => {
            let res = atlas_assembler::assemble(&input, &output)
                .map_err(|e| format!("{}", e));
            if res.is_ok() {
                match ObjectFile::from_file(&output) {
                    Ok(obj) => {
                        if args.verbose {
                            println!();
                            inspect_obj(&obj);
                            let labels = build_label_map(&obj);
                            for sec in &obj.sections {
                                if sec.name == ".text" {
                                    println!();
                                    disassemble(&sec.data, &labels);
                                }
                            }
                            println!();
                        }
                        print_asm_summary(&input, &output, &obj);
                    }
                    Err(e) => eprintln!("Failed to read object file '{}': {}", output, e),
                }
            }
            res
        },
        Command::Ld { inputs, output } => {
            let input_refs: Vec<&str> = inputs.iter().map(|s| s.as_str()).collect();
            let res = atlas_linker::link(&input_refs, &output)
                .map_err(|e| format!("{}", e));
            if res.is_ok() {
                // Build a combined label map from all input object files
                let mut labels = BTreeMap::new();
                let mut text_base: u32 = 0;
                for inp in &inputs {
                    if let Ok(obj) = ObjectFile::from_file(inp) {
                        for sym in &obj.symbols {
                            if let Some(ref sec) = sym.section {
                                if sec == ".text" {
                                    labels.insert((text_base + sym.value) as u16, sym.name.clone());
                                } else if sec == ".abs" {
                                    labels.insert(sym.value as u16, sym.name.clone());
                                }
                            }
                        }
                        for sec in &obj.sections {
                            if sec.name == ".text" {
                                text_base += sec.data.len() as u32;
                            }
                        }
                    }
                }

                // Read back the linked output for disassembly
                let data = if output.ends_with(".hex") {
                    atlas_files::hex::read_hex_file(&output).unwrap_or_default()
                } else {
                    std::fs::read(&output).unwrap_or_default()
                };

                if args.verbose {
                    println!();
                    disassemble(&data, &labels);
                    println!();
                }
                print_link_summary(&inputs, &output, data.len());
            }
            res
        },
        Command::Inspect { .. } => {
            eprintln!("Inspect command is not implemented yet.");
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("\n  error: {}", e);
        std::process::exit(1);
    }
}