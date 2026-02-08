
//! Atlas Inspect - Object file inspection and disassembly utilities.
//!
//! This crate provides tools for analyzing and displaying object files, including
//! disassembly, symbol tables, and various formatting utilities.

pub mod formatting;
pub mod output;
pub mod disassembly;

// Re-export commonly used functions for convenience
pub use output::{print_asm_summary, print_link_summary, inspect_obj, build_label_map};
pub use disassembly::disassemble;
pub use formatting::{use_colour, bold, dim, green, cyan, yellow};