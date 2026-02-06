# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] 2026-02-06

### Added

- Generic `Operand` type (`Immediate(u16)` / `Label(String)`) usable in I-type, P-type, and BI-type instructions
- `.imm` directive for named immediate constants (`label: .imm value`)
- Assembler enforces r0-is-zero: errors on any instruction that writes to r0 (except `cmp`, `tst`, and `nop`)
- Multi-file linking with cross-file `.import` / `.export` symbol resolution
- Verbose disassembly output for both `asm` and `ld` commands
- Comprehensive test assembly programs (single-file and multi-file)

### Changed

- Widened address space from `u8` to `u16` throughout the toolchain (registers, symbols, operands, label maps) to match the actual 16-bit virtual address space
- Renamed `PortOp` to `PeekPokeOp` across the entire codebase
- Renamed `BranchOperand` to `Operand` (generic); `BranchOperand` kept as type alias
- A-type encoding layout fixed: `[15:12]=0, [11:8]=dest, [7:4]=source, [3:0]=op`
- Encoder now validates that `u16` operand values fit in 8-bit instruction fields at encode time

### Fixed

- Parser lookahead bug: iterator's `next()` bypassed the `pending` token buffer, causing stale tokens after label definitions
- A-type encoding placed `dest` in bits [15:12] (the type identifier field), causing non-r0 destinations to decode as wrong instruction types
- P-type decoder had `PeekPokeOp::PEEK` and `PeekPokeOp::POKE` swapped relative to the enum discriminants
- X-type parser did not accept `Token::EoF` as a valid instruction terminator, causing errors on the last instruction in a file without a trailing newline

## [0.1.0] 2026-02-05

### Added

- README & CHANGELOG
- Moved in code from previous repo
    - basic clap setup
    - basic lexer
    - basic parser
    - basic assembler
    - basic linker