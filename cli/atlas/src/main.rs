pub mod args;

use args::Arguments;
use clap::Parser;

use crate::args::Command;

fn main() {
    let args = Arguments::parse();

    let result = match args.command {
        Command::Asm { input, output } => {
            atlas_assembler::assemble(&input, &output)
                .map_err(|e| format!("{}", e))
        },
        Command::Ld { inputs, output } => {
            let input_refs: Vec<&str> = inputs.iter().map(|s| s.as_str()).collect();
            atlas_linker::link(&input_refs, &output)
                .map_err(|e| format!("{}", e))
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}