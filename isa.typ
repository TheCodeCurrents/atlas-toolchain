// Import unified documentation module
#include "../typst/init.typ";
#import "../typst/styling/heading.typ": *

// Document configuration
#let title = "Atlas-8 ISA Specification"
#let version = "1.0"
#let date = "January 2026"
#let author = "Jakob Flocke – Atlas Project"
#let overview = "Complete instruction set architecture specification for the Atlas-8 processor, including the base instruction set and optional ISA extensions for caching and memory management."

#doc-title(title, version: version, date: date, author: author, overview: overview)

#outline()

#pagebreak()

= Overview

The Atlas-8 instruction set architecture (ISA) defines a machine model and the set of instructions that the Atlas-8 processor can execute.
This document provides a comprehensive specification of the ISA.

The Atlas-8 is a 8-bit RISC architecture designed for efficiency and simplicity, while supporting a variety of complex features like caching and memory management through optional extensions.
It uses fixed-length 16-bit instructions and a load/store architecture.

== Machine Model

The Atlas-8 processor has the following key components:
- 16 8-bit registers (R0-R15)
  -> 10 general-purpose registers (R0-R9)
  -> 3 16-bit (2x8-bit) special-purpose registers (R10-R15)
- 64KB of random access memory
- Program Counter (PC) and Status Register (SR)
- 2 cache levels (L1 and L2) for instruction and data caching
- Memory Management Unit (MMU) for virtual memory support (maps 16-bit virtual addresses to 24-bit physical addresses)

== Instruction Set

The Atlas-8 ISA includes a base instruction set and optional extensions for caching and memory management.
The base instruction set consists of arithmetic, logical, data movement, control flow, and system instructions.
Each instruction is 16 bits long and follows a fixed format based on it's type. There are 8 primary instruction formats: R-type, I-type, J-type, S-type, and U-type.

#pagebreak()

= Machine Model

The Atlas-8 processor architecture a simplified yet powerful machine model designed to balance performance with implementation simplicity. This section details the core components and their interactions.

== Register File

The Atlas-8 contains 16 general-purpose and special-purpose registers, each 8 bits wide, providing a total of 128 bits of fast storage.

*General-Purpose Registers (R0-R9):*
These 10 registers are available for general computation and data storage. They follow standard register calling conventions:
- R0: Zero register (always reads as 0)
- R1-R3: Argument registers
- R4-R7: Return value and temporary registers
- R8-R9: Temporary and saved registers

*Special-Purpose Registers (R10-R15):*
These 6 registers are 16-bit wide (combining two 8-bit slots) and serve specific architectural functions:
- R10-R11 (16-bit): Temporary Register (TR)
- R12-R13 (16-bit): Stack Pointer (SP)
- R14-R15 (16-bit): Program Counter (PC)

Many instructions will treat either one of the pairs as a single 16-bit register for operations requiring larger data sizes.
There are also some instructions (like load/store) that have special codes that refer to the special-purpose registers directly.

== Memory Organization

The Atlas-8 has a 64 KB flat memory address space (16-bit addressing), providing 65,536 addressable bytes. Due to the MMU extension, physical memory can be expanded up to 16 MB (24-bit addressing). The memory organization is completely up to the system designer and highly flexible with the MMU extension. Only the MMUs page table structure is fixed.

0x000000-0x00FFFF: Addressable memory space (64 KB)
0x010000-0xFFFFFF: Reserved for MMU-mapped physical memory (up to 16 MB)

Memory is obviously byte-addressable, and multi-byte data (16-bit and 32-bit values) follow little-endian byte ordering.

== Cache Hierarchy

The Atlas-8 ISA includes a cache extension that introduces a two-level cache hierarchy to improve memory access performance.

*Level 1 Cache (L1):*
- split cache I- and D-cache
- 32-byte cache lines
- 4-way associative

*Level 2 Cache (L2):*
- unified cache
- 64-byte cache lines
- 8-way associative

Cache coherency is maintained through a write-through policy at L1 and write-back at L2.

== Memory Management Unit (MMU)

For systems requiring virtual memory support (enabled via MMU extension), the Atlas-8 includes a Memory Management Unit:

*Virtual to Physical Address Translation:*
- Inputs:
  - 16-bit virtual address
    - 6-bit virtual page number
    - 10-bit page offset
  - 8-bit process ID
- Output: 24-bit physical address

*Features:*
- 1 KB page size
- Up to 16 MB of addressable physical memory (24-bit addresses)
- User/supervisor privilege levels with memory protection

The MMU is optional and must be explicitly enabled through the ISA extension mechanism.

== Processor Modes

The Atlas-8 supports two privilege levels:

*User Mode:* Limited memory access and instruction set. Cannot execute privileged instructions or access system memory (0x8000-0xFFFF).

*Supervisor Mode:* Full instruction set and memory access. Required for operating system and system software.

Mode transitions occur through exception handling and return-from-exception instructions.

#pagebreak()

= Instruction Format

The instructions are encoded in a fixed length 16-bit format with 7 primary types:
- A-type: Arithmetic and Logical Instructions
- I-type: Immediate Instructions
- M-type: Memory Instructions
- BI-type: Branch Immediate Instructions
- BR-type: Branch Register Instructions
- S-type: Stack Instructions
- X-type: Extended Instructions

The type is always determined by its first 4 bits called the type field.

== A-type Instructions
The A-type instructions perform arithmetic and logical operations between a source and a destination register.
$
  r_d = r_d times r_s
$

#align(center)[
  #table(
    columns: 2,
    align: (center, left),
    fill: (x, y) =>
      if y == 0 {
        gray.lighten(40%)
      },
    [*Field*],  [*Description*],
    [15:12],  [type-field = 0000],
    [11:8],   [destination register (r_d)],
    [7:4],    [source register (r_s)],
    [3:0],    [operation code (opcode) for arithmetic/logical operation]
  )
]

This results in the following list of instructions:

#align(center)[
  #table(
    columns: 4,
    align: (center, center, left, left),
    fill: (x, y) =>
      if y == 0 {
        gray.lighten(40%)
      },
    [*Opcode*], [*Mnemonic*], [*Description*], [*Operation*],
    [0],      [add],      [Add],          [$r_d = r_d + r_s$],
    [1],      [addc],     [Add with Carry],        [$r_d = r_d + r_s + C$],
    [2],      [sub],      [Subtract],      [$r_d = r_d - r_s$],
    [3],      [subc],     [Subtract with Carry],   [$r_d = r_d - r_s - C$],
    [4],      [and],      [Bitwise AND],    [$r_d = r_d and r_s$],
    [5],      [or],       [Bitwise OR],     [$r_d = r_d or r_s$],
    [6],      [xor],      [Bitwise XOR],    [$r_d = r_d xor r_s$],
    [7],      [not],      [Bitwise NOT],    [$r_d = not r_s$],
    [8],      [shl],      [Shift Left],     [$r_d = r_s << 1$],
    [9],      [shr],      [Shift Right],    [$r_d = r_s >> 1$],
    [10],     [rol],      [Rotate Left],    [$r_d = (r_s << 1) | (r_s >> 7)$],
    [11],     [ror],      [Rotate Right],   [$r_d = (r_s >> 1) | (r_s << 7)$],
    [12],     [cmp],      [Compare],        [Set flags based on $r_d - r_s$],
    [13],     [tst],      [Test],          [Sets flags based on $r_d and r_s$],
    [14],     [mov],      [Move],          [$r_d = r_s$],
    [15],     [neg],      [Negate],        [$r_d = -r_d$]
  )
]

== I-type Instructions
I-type instructions perform operations between a register and an 8-bit immediate value.

#align(center)[
  #table(
    columns: 2,
    align: (center, left),
    fill: (x, y) =>
      if y == 0 {
        gray.lighten(40%)
      },
    [*Field*],  [*Description*],
    [15:12],  [type-field = 0001 (ldi), 0010 (addi), 0011 (subi), 0100 (andi) or 0101 (ori)],
    [11:8],   [destination register (r_d)],
    [7:0],    [immediate value (imm8)],
  )
]

This results in the following instructions: \
ldi rd, imm8
addi rd, imm8
subi rd, imm8
andi rd, imm8
ori rd, imm8

== M-type Instructions
M-type instructions handle memory load and store operations between registers and memory addresses.

#align(center)[
  #table(
    columns: 2,
    align: (center, left),
    fill: (x, y) =>
      if y == 0 {
        gray.lighten(40%)
      },
    [*Field*],  [*Description*],
    [15:12],  [type-field = 0110 (load) or 0111 (store)],
    [11:8],   [register designed / source],
    [7:4],    [base register (r_b)],
    [3:0],    [offset (4-bit signed immediate) or spr-code]
  )
]

The field [3:0] holds a 4-bit signed offset. Values −5 to +7 are used as regular signed immediates. Negative values below −5 serve as special-purpose register (SPR) selectors:
- −6 (0b1010): Temporary Register (TR, R10:R11)
- −7 (0b1001): Stack Pointer (SP, R12:R13)
- −8 (0b1000): Program Counter (PC, R14:R15)
This results in the following possible immediate values for offset:
- -5 to +7 (4-bit signed immediate)

This results in the following list of instructions:
- Load Instructions (type-field = 0010):
  - ld rd, [rb]
  - ld rd, [rb + offset]
  - lds rd, [rb + spr]
- Store Instructions (type-field = 0011):
  - st rd, [rb]
  - st rd, [rb + offset]
  - sts rd, [rb + spr]

== BI-type Instructions
The BI-type instructions perform conditional branches based on the status flags and an 8-bit signed immediate offset.

#align(center)[
  #table(
    columns: 2,
    align: (center, left),
    fill: (x, y) =>
      if y == 0 {
        gray.lighten(40%)
      },
    [*Field*],  [*Description*],
    [15:12],  [type-field = 1000 (branch immediate)],
    [11:11],  [relative (0) / absolute (1)],
    [10:8],   [condition code],
    [7:0],    [offset (8-bit signed immediate)]
  )
]

== BR-type Instructions
The BR-type instructions perform conditional branches based on the status flags and an 8-bit or 16-bit signed register offset.

#align(center)[
  #table(
    columns: 2,
    align: (center, left),
    fill: (x, y) =>
      if y == 0 {
        gray.lighten(40%)
      },
    [*Field*],  [*Description*],
    [15:12],  [type-field = 1001 (branch register)],
    [11:11],  [relative (0) / absolute (1)],
    [10:8],   [condition code],
    [7:4],    [rs low],
    [3:0],    [rs high]
  )
]

This results in the following list of condition codes:
- 000: always
- 001: equal (Z=1)
- 010: not equal (Z=0)
- 011: carry set (C=1)
- 100: carry clear (C=0)
- 101: negative (N=1)
- 110: positive (N=0)
- 111: overflow (V=1)

=== Resulting Instructions
All of the following instructions use either an 8-bit signed immediate offset (BI-type) or a register pair as 16-bit signed offset (BR-type):

#align(center)[
  #table(
    columns: 4,
    align: (center, center, left, left),
    fill: (x, y) =>
      if y == 0 {
        gray.lighten(40%)
      },
    [*Mnemonic*], [*Type*], [*Description*], [*Operation*],
    [br],     [BI-type],    [Branch unconditionally], [PC = PC + offset],
    [beq],    [BI-type],    [Branch if Equal],        [if Z=1 then PC = PC + offset],
    [bne],    [BI-type],    [Branch if Not Equal],    [if Z=0 then PC = PC + offset],
    [bcs],    [BI-type],    [Branch if Carry Set],    [if C=1 then PC = PC + offset],
    [bcc],    [BI-type],    [Branch if Carry Clear],  [if C=0 then PC = PC + offset],
    [bmi],    [BI-type],    [Branch if Negative],     [if N=1 then PC = PC + offset],
    [bpl],    [BI-type],    [Branch if Positive],     [if N=0 then PC = PC + offset],
    [bov],    [BI-type],    [Branch if Overflow],     [if V=1 then PC = PC + offset]
  )
]

== S-type Instructions
S-type instructions manage stack operations using the Stack Pointer (SP).

#align(center)[
  #table(
    columns: 2,
    align: (center, left),
    fill: (x, y) =>
      if y == 0 {
        gray.lighten(40%)
      },
    [*Field*],  [*Description*],
    [15:12],  [type-field = 1010],
    [11:8],   [extended opcode (xop)],
    [7:0],    [imm8 or register pair depending on xop]
  )
]

That results in the following list of instructions:

#align(center)[
  #table(
    columns: 4,
    align: (center, center, left, left),
    fill: (x, y) => if y == 0 { gray.lighten(40%) },
    [*xop*], [*Mnemonic*],  [*Description*],            [*Operation*],
    [0x0],    [push rs],    [Push register onto stack], [SP = SP - 2; MEM[SP] = rs],
    [0x1],    [pop rd],     [Pop register from stack],  [SP = SP + 2; rd = MEM[SP]],
    [0x2],    [subsp imm8], [allocate stack],           [SP = SP - imm8],
    [0x3],    [subsp rs],   [allocate stack],           [SP = SP - register],
    [0x4],    [addsp imm8], [deallocate stack],         [SP = SP + imm8],
    [0x5],    [addsp rs],   [deallocate stack],         [SP = SP + register],
  )
]

== P-type Instructions
P-type instructions perform Peek and Poke operations to read and write to an 8-bit unsigned offset relative to the SP.
Adding this as a separate type is a tradeoff I was willing to make to increase the offset size from 4 to 8-bit unsigned immediates.

#align(center)[
  #table(
    columns: 2,
    align: (center, left),
    fill: (x, y) =>
      if y == 0 {
        gray.lighten(40%)
      },
    [*Field*],  [*Description*],
    [15:12],  [type-field = 1011 (peek) or 1100 (poke)],
    [11:8],   [register source / destination],
    [7:0],    [offset (8-bit unsigned immediate)]
  )
]

This results in the following instructions:
- peek rd, offset
- poke rs, offset

=== possible wishes

I would liek to expand it to have a poke / peek with an 8-bit immediate, but where to put the register source / destination encoding?

== X-type Instructions

X-type instructions are used for privileged operations including system control, exception handling, and cache management. All X-type instructions are *privileged*; executing an X-type instruction in user mode raises an illegal instruction exception.

The MMU is controlled entirely through memory-mapped registers rather than special instructions, providing a clean separation between cache control and memory management.

#align(center)[
  #table(
    columns: 2,
    align: (center, left),
    fill: (x, y) => if y == 0 { gray.lighten(40%) },
    [*Field*],  [*Description*],
    [15:12],  [type-field = 1101],
    [11:8],   [operation code (opcode)],
    [7:0],    [ignored for most instructions; used for 8-bit immediate on SYSC]
  )
]

=== Encoding Conventions

- Most X-type instructions have no operands and ignore bits [7:0].
- SYSC uses bits [7:0] as an 8-bit syscall number.
- Cache control instructions (ICINV, DCINV, DCCLEAN, FLUSH) have no operands.

=== X-type Instructions

#table(
  columns: 4,
  align: (center, center, left, left),
  fill: (x, y) => if y == 0 { gray.lighten(40%) },
  [*Opcode*], [*Mnemonic*], [*Description*], [*Operation*],
  [0x0], [sysc], [Software Syscall], [Trap to supervisor with syscall number in bits [7:0]],
  [0x1], [eret], [Return from Exception], [Restore PC, SR, and previous privilege mode],
  [0x2], [halt], [Halt Processor], [Stop execution until reset or interrupt],
  [0x3], [icinv], [Invalidate Instruction Cache], [Discard all instruction cache contents],
  [0x4], [dcinv], [Invalidate Data Cache], [Discard all data cache contents],
  [0x5], [dcclean], [Clean Data Cache], [Write back all dirty data cache lines to memory],
  [0x6], [flush], [Flush Pipeline and Caches], [Clean and invalidate all cache levels and pipeline]
)

#pagebreak()

= Instruction Set Summary

This section provides a summary of all instructions defined in the Atlas-8 ISA.

#table(
  columns: 4,
  align: (center, center, left, left),
  fill: (x, y) => if y == 0 { gray.lighten(40%) },
  [*Mnemonic*], [*Type*], [*Description*], [*Operation*],
  // A-type Instructions
  [add], [A-type], [Add], [rd = rd + rs],
  [addc], [A-type], [Add with Carry], [rd = rd + rs + C],
  [sub], [A-type], [Subtract], [rd = rd - rs],
  [subc], [A-type], [Subtract with Carry], [rd = rd - rs - C],
  [and], [A-type], [Bitwise AND], [rd = rd and rs],
  [or], [A-type], [Bitwise OR], [rd = rd or rs],
  [xor], [A-type], [Bitwise XOR], [rd = rd xor rs],
  [not], [A-type], [Bitwise NOT], [rd = not rs],
  [shl], [A-type], [Shift Left], [rd = rs << 1],
  [shr], [A-type], [Shift Right], [rd = rs >> 1],
  [rol], [A-type], [Rotate Left], [rd = (rs << 1) | (rs >> 7)],
  [ror], [A-type], [Rotate Right], [rd = (rs >> 1) | (rs << 7)],
  [cmp], [A-type], [Compare], [Set flags based on rd - rs],
  [tst], [A-type], [Test], [Sets flags based on rd and rs],
  [mov], [A-type], [Move], [rd = rs],
  [neg], [A-type], [Negate], [rd = -rd],
  // I-type Instructions
  [ldi], [I-type], [Load Immediate], [rd = imm8],
  [addi], [I-type], [Add Immediate], [rd = rd + imm8],
  [subi], [I-type], [Subtract Immediate], [rd = rd - imm8],
  [andi], [I-type], [AND Immediate], [rd = rd and imm8],
  [ori], [I-type], [OR Immediate], [rd = rd or imm8],
  // M-type Instructions
  [ld], [M-type], [Load from Memory], [rd = MEM[rb + offset]],
  [st], [M-type], [Store to Memory], [MEM[rb + offset] = rd],
  // BI-type Instructions
  [br], [BI-type], [Branch unconditionally], [PC = PC + offset],
  [beq], [BI-type], [Branch if Equal], [if Z=1 then PC = PC + offset],
  [bne], [BI-type], [Branch if Not Equal], [if Z=0 then PC = PC + offset],
  [bcs], [BI-type], [Branch if Carry Set], [if C=1 then PC = PC + offset],
  [bcc], [BI-type], [Branch if Carry Clear], [if C=0 then PC = PC + offset],
  [bmi], [BI-type], [Branch if Negative], [if N=1 then PC = PC + offset],
  [bpl], [BI-type], [Branch if Positive], [if N=0 then PC = PC + offset],
  [bov], [BI-type], [Branch if Overflow], [if V=1 then PC = PC + offset],
  // BR-type Instructions
  [br], [BR-type], [Branch unconditionally], [PC = PC + rs],
  [beq], [BR-type], [Branch if Equal], [if Z=1 then PC = PC + rs],
  [bne], [BR-type], [Branch if Not Equal], [if Z=0 then PC = PC + rs],
  [bcs], [BR-type], [Branch if Carry Set], [if C=1 then PC = PC + rs],
  [bcc], [BR-type], [Branch if Carry Clear], [if C=0 then PC = PC + rs],
  [bmi], [BR-type], [Branch if Negative], [if N=1 then PC = PC + rs],
  [bpl], [BR-type], [Branch if Positive], [if N=0 then PC = PC + rs],
  [bov], [BR-type], [Branch if Overflow], [if V=1 then PC = PC + rs],
  // S-type Instructions
  [push], [S-type], [Push register onto stack], [SP = SP - 2; MEM[SP] = rs],
  [pop], [S-type], [Pop register from stack], [SP = SP + 2; rd = MEM[SP]],
  [subsp], [S-type], [Allocate stack], [SP = SP - imm8 or SP = SP - register],
  [addsp], [S-type], [Deallocate stack], [SP = SP + imm8 or SP = SP + register],
  // P-type Instructions
  [peek], [P-type], [Peek from stack], [rd = MEM[SP + offset]],
  [poke], [P-type], [Poke to stack], [MEM[SP + offset] = rs],
  // X-type Instructions
  [sysc], [X-type], [Software Syscall], [Trap to supervisor with syscall number],
  [eret], [X-type], [Return from exception], [Restore PC, SR, and previous privilege mode],
  [halt], [X-type], [Halt processor], [Stop execution until reset or interrupt],
  [icinv], [X-type], [Invalidate instruction cache], [Discard all instruction cache contents],
  [dcinv], [X-type], [Invalidate data cache], [Discard all data cache contents],
  [dcclean], [X-type], [Clean data cache], [Write back dirty cache lines to memory],
  [flush], [X-type], [Flush all caches], [Clean and invalidate all cache levels],

  // virtual instructions
  [nop], [virtual], [No Operation], [r0 = r0 + r0],
  [inc], [virtual], [Increment], [rd = rd + 1],
  [dec], [virtual], [Decrement], [rd = rd - 1]
)