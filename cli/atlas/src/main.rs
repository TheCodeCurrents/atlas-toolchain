pub mod args;

use args::Arguments;
use clap::Parser;

use crate::args::Command;
use atlas_files::{ObjectFile, FileFormat};
use atlas_inspect::inspect_obj;

fn main() {
    let args = Arguments::parse();

    if args.verbose {
        eprintln!("Verbose mode enabled");
    }

    let is_link = matches!(&args.command, Command::Ld { .. });

    let output_path = match &args.command {
        Command::Asm { output, .. } => output.clone(),
        Command::Ld { output, .. } => output.clone(),
        Command::Inspect { .. } => {
            eprintln!("Inspect command does not produce an output file to read for verbose mode.");
            std::process::exit(1);
        }
    };

    let result = match args.command {
        Command::Asm { input, output } => {
            if args.verbose {
                eprintln!("Assembling {} -> {}", input, output);
            }
            let res = atlas_assembler::assemble(&input, &output)
                .map_err(|e| format!("{}", e));
            if args.verbose && res.is_ok() {
                match ObjectFile::from_file(&output) {
                    Ok(obj) => inspect_obj(&obj),
                    Err(e) => eprintln!("Failed to inspect object file '{}': {}", output, e),
                }
            }
            res
        },
        Command::Ld { inputs, output } => {
            if args.verbose {
                eprintln!("Linking {:?} -> {}", inputs, output);
            }
            let input_refs: Vec<&str> = inputs.iter().map(|s| s.as_str()).collect();
            atlas_linker::link(&input_refs, &output)
                .map_err(|e| format!("{}", e))
        },
        Command::Inspect { .. } => {
            eprintln!("Inspect command is not implemented yet.");
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}