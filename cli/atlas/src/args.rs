use clap:: {
    Parser,
    Subcommand,
};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Arguments {
    /// enable verbose output
    #[arg(short = 'v', long = "verbose", global = true)]
    pub verbose: bool,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Asm {
        /// input assembly file
        #[arg(value_name = "INPUT")]
        input: String,
        
        /// output object file (.o)
        #[arg(value_name = "OUTPUT")]
        output: String,
    },
    Ld {
        /// input object files (.o)
        #[arg(value_name = "INPUTS", required = true)]
        inputs: Vec<String>,
        
        /// output binary file
        #[arg(short = 'o', long = "output", value_name = "OUTPUT")]
        output: String,
    },
    Inspect {
        /// input file to inspect
        #[arg(value_name = "INPUT")]
        input: String,

        /// format of the output
        #[arg(short = 'f', long = "format")]
        format: Option<String>,
    },
}