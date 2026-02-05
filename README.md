# Atlas Toolchain

This is the Rust based toolchain for the Atlas ecosystem. It aims to be
a all in one solution for working with the ecosystem from outside.

The CLI usages include:

- compiler      (planned)
- assembler     (partially complete)
- linker        (partially complete)
- binutils      (planned)
- simulator     (planned)
- emulator      (planned)
- formatter     (planned)
- syntax check  (planned)

## cli interface
atlas asm        # assemble
atlas ld         # link
atlas objdump    # inspect binaries
atlas nm         # symbols
atlas sim        # cycle-accurate simulator
atlas emu        # system emulator
atlas run        # assemble+link+run
atlas fmt        # formatter
atlas check      # syntax + semantic checks
atlas lint       # style / footguns
atlas test       # golden tests / traces

# AI Usage

In this repo AI should only be used to assist in Planning, never in
writing code.