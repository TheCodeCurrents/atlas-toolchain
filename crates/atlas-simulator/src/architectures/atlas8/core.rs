use atlas_isa::opcode::{AluOp, BranchCond, ImmOp, MemOp, PeekPokeOp, StackOp, XTypeOp};
use atlas_isa::operands::{MOffset, Operand, XOperand};
use atlas_isa::ParsedInstruction;
use crate::{bus::BusMaster, cpu::CPU, system::Addr};

/// Status register flags
#[derive(Debug, Clone, Copy, Default)]
pub struct StatusFlags {
    pub zero: bool,     // Z
    pub carry: bool,    // C
    pub negative: bool, // N
    pub overflow: bool, // V
}

pub struct Atlas8Core {
    /// 16 × 8-bit registers (R0-R15)
    pub regs: [u8; 16],
    /// Program counter (kept in sync with R14:R15)
    pub pc: Addr,
    /// Status register
    pub sr: StatusFlags,
    /// Bus for memory access
    pub bus: Box<dyn BusMaster>,
    /// Whether the CPU has been halted
    pub halted: bool,
    /// Supervisor mode
    pub supervisor: bool,
}

impl Atlas8Core {
    pub fn new(bus: Box<dyn BusMaster>) -> Self {
        Self {
            regs: [0; 16],
            pc: 0,
            sr: StatusFlags::default(),
            bus,
            halted: false,
            supervisor: true,
        }
    }

    // ── Register helpers ─────────────────────────────────────────────

    /// Read an 8-bit register. R0 always returns 0.
    fn reg(&self, r: u8) -> u8 {
        if r == 0 { 0 } else { self.regs[r as usize] }
    }

    /// Write an 8-bit register. Writes to R0 are silently discarded.
    fn set_reg(&mut self, r: u8, val: u8) {
        if r != 0 {
            self.regs[r as usize] = val;
        }
    }

    /// Read a 16-bit special-purpose register pair (little-endian: low reg first).
    fn reg_pair(&self, hi: u8, lo: u8) -> u16 {
        (self.regs[hi as usize] as u16) << 8 | self.regs[lo as usize] as u16
    }

    /// Write a 16-bit value into a register pair.
    fn set_reg_pair(&mut self, hi: u8, lo: u8, val: u16) {
        self.regs[hi as usize] = (val >> 8) as u8;
        self.regs[lo as usize] = (val & 0xFF) as u8;
    }

    /// Read the Stack Pointer (R12:R13).
    fn sp(&self) -> u16 {
        self.reg_pair(12, 13)
    }

    /// Write the Stack Pointer.
    fn set_sp(&mut self, val: u16) {
        self.set_reg_pair(12, 13, val);
    }

    /// Sync the PC into R14:R15.
    fn sync_pc_to_regs(&mut self) {
        let pc = self.pc as u16;
        self.regs[14] = (pc >> 8) as u8;
        self.regs[15] = (pc & 0xFF) as u8;
    }

    /// Sync R14:R15 into the PC.
    fn sync_regs_to_pc(&mut self) {
        self.pc = self.reg_pair(14, 15) as Addr;
    }

    // ── Flag helpers ─────────────────────────────────────────────────

    /// Set Z and N flags based on an 8-bit result.
    fn set_zn(&mut self, result: u8) {
        self.sr.zero = result == 0;
        self.sr.negative = (result & 0x80) != 0;
    }

    /// Set all arithmetic flags for an addition: result, carry, overflow.
    fn set_flags_add(&mut self, a: u8, b: u8, result: u16) {
        let r = result as u8;
        self.set_zn(r);
        self.sr.carry = result > 0xFF;
        // Signed overflow: both operands same sign, result different sign
        self.sr.overflow = ((a ^ r) & (b ^ r) & 0x80) != 0;
    }

    /// Set all arithmetic flags for a subtraction (a - b).
    fn set_flags_sub(&mut self, a: u8, b: u8, result: u16) {
        let r = result as u8;
        self.set_zn(r);
        // Carry (borrow): set when b > a (unsigned)
        self.sr.carry = (result & 0x100) != 0;
        // Signed overflow
        self.sr.overflow = ((a ^ b) & (a ^ r) & 0x80) != 0;
    }

    // ── Branch condition evaluation ──────────────────────────────────

    fn condition_met(&self, cond: BranchCond) -> bool {
        match cond {
            BranchCond::Unconditional => true,
            BranchCond::EQ => self.sr.zero,
            BranchCond::NE => !self.sr.zero,
            BranchCond::CS => self.sr.carry,
            BranchCond::CC => !self.sr.carry,
            BranchCond::MI => self.sr.negative,
            BranchCond::PL => !self.sr.negative,
            BranchCond::OV => self.sr.overflow,
        }
    }

    // ── Memory helpers (byte-level) ──────────────────────────────────

    fn mem_read_byte(&self, addr: u16) -> u8 {
        self.bus.read(addr as Addr, 1) as u8
    }

    fn mem_write_byte(&mut self, addr: u16, val: u8) {
        self.bus.write(addr as Addr, val as u64);
    }

    // ── SPR code resolution for M-type offset field ──────────────────

    /// Resolve the M-type offset field. Values −6, −7, −8 (encoded as
    /// 0xA, 0x9, 0x8 in 4-bit two's complement) select a special-purpose
    /// register pair; otherwise it is a plain signed 4-bit offset.
    fn resolve_m_offset(&self, base_val: u8, offset: &MOffset) -> u16 {
        match offset {
            MOffset::Offset8(raw) => {
                // 4-bit signed: sign-extend from bit 3
                let sext = if *raw & 0x8 != 0 {
                    *raw | 0xF0 // sign-extend to 8 bits
                } else {
                    *raw
                };
                (base_val as u16).wrapping_add(sext as i8 as i16 as u16)
            }
            MOffset::SR(spr_reg) => {
                // SPR codes map to register pairs
                let pair_val = match *spr_reg {
                    // These are the negative-encoded SPR selectors
                    r => self.reg_pair(r, r + 1),
                };
                (base_val as u16).wrapping_add(pair_val)
            }
        }
    }
}

impl CPU for Atlas8Core {
    fn tick(&mut self) {
        if self.halted {
            return;
        }

        let inst_bytes = self.bus.read(self.pc, 2);
        self.pc += 2;
        self.sync_pc_to_regs();

        let inst = match ParsedInstruction::decode(inst_bytes as u16) {
            Ok(inst) => inst,
            Err(_) => {
                panic!("Invalid instruction at {:#06x}: {:#06x}", self.pc - 2, inst_bytes);
            }
        };

        self.execute_instruction(inst);
    }
}

impl Atlas8Core {
    pub fn execute_instruction(&mut self, inst: ParsedInstruction) {
        match inst {
            // ═══════════════════════════════════════════════════════════
            //  A-type: ALU register-register
            // ═══════════════════════════════════════════════════════════
            ParsedInstruction::A { op, dest, source, .. } => {
                let d = self.reg(dest);
                let s = self.reg(source);

                match op {
                    AluOp::ADD => {
                        let result = d as u16 + s as u16;
                        self.set_flags_add(d, s, result);
                        self.set_reg(dest, result as u8);
                    }
                    AluOp::ADDC => {
                        let c = self.sr.carry as u16;
                        let result = d as u16 + s as u16 + c;
                        self.set_flags_add(d, s, result);
                        self.set_reg(dest, result as u8);
                    }
                    AluOp::SUB => {
                        let result = (d as u16).wrapping_sub(s as u16);
                        self.set_flags_sub(d, s, result);
                        self.set_reg(dest, result as u8);
                    }
                    AluOp::SUBC => {
                        let c = self.sr.carry as u16;
                        let result = (d as u16).wrapping_sub(s as u16).wrapping_sub(c);
                        self.set_flags_sub(d, s, result);
                        self.set_reg(dest, result as u8);
                    }
                    AluOp::AND => {
                        let result = d & s;
                        self.set_zn(result);
                        self.set_reg(dest, result);
                    }
                    AluOp::OR => {
                        let result = d | s;
                        self.set_zn(result);
                        self.set_reg(dest, result);
                    }
                    AluOp::XOR => {
                        let result = d ^ s;
                        self.set_zn(result);
                        self.set_reg(dest, result);
                    }
                    AluOp::NOT => {
                        let result = !s;
                        self.set_zn(result);
                        self.set_reg(dest, result);
                    }
                    AluOp::SHL => {
                        self.sr.carry = (s & 0x80) != 0;
                        let result = s << 1;
                        self.set_zn(result);
                        self.set_reg(dest, result);
                    }
                    AluOp::SHR => {
                        self.sr.carry = (s & 0x01) != 0;
                        let result = s >> 1;
                        self.set_zn(result);
                        self.set_reg(dest, result);
                    }
                    AluOp::ROL => {
                        let result = (s << 1) | (s >> 7);
                        self.sr.carry = (s & 0x80) != 0;
                        self.set_zn(result);
                        self.set_reg(dest, result);
                    }
                    AluOp::ROR => {
                        let result = (s >> 1) | (s << 7);
                        self.sr.carry = (s & 0x01) != 0;
                        self.set_zn(result);
                        self.set_reg(dest, result);
                    }
                    AluOp::CMP => {
                        let result = (d as u16).wrapping_sub(s as u16);
                        self.set_flags_sub(d, s, result);
                        // CMP does NOT write back to dest
                    }
                    AluOp::TST => {
                        let result = d & s;
                        self.set_zn(result);
                        // TST does NOT write back to dest
                    }
                    AluOp::MOV => {
                        self.set_reg(dest, s);
                    }
                    AluOp::NEG => {
                        let result = (-(d as i8)) as u8;
                        self.set_zn(result);
                        self.sr.carry = d != 0;
                        self.sr.overflow = d == 0x80;
                        self.set_reg(dest, result);
                    }
                }
            }

            // ═══════════════════════════════════════════════════════════
            //  I-type: Immediate operations
            // ═══════════════════════════════════════════════════════════
            ParsedInstruction::I { op, dest, immediate, .. } => {
                let imm = match immediate {
                    Operand::Immediate(v) => v as u8,
                    Operand::Label(_) => panic!("Unresolved label in simulator"),
                };
                let d = self.reg(dest);

                match op {
                    ImmOp::LDI => {
                        self.set_reg(dest, imm);
                    }
                    ImmOp::ADDI => {
                        let result = d as u16 + imm as u16;
                        self.set_flags_add(d, imm, result);
                        self.set_reg(dest, result as u8);
                    }
                    ImmOp::SUBI => {
                        let result = (d as u16).wrapping_sub(imm as u16);
                        self.set_flags_sub(d, imm, result);
                        self.set_reg(dest, result as u8);
                    }
                    ImmOp::ANDI => {
                        let result = d & imm;
                        self.set_zn(result);
                        self.set_reg(dest, result);
                    }
                    ImmOp::ORI => {
                        let result = d | imm;
                        self.set_zn(result);
                        self.set_reg(dest, result);
                    }
                }
            }

            // ═══════════════════════════════════════════════════════════
            //  M-type: Memory load / store
            // ═══════════════════════════════════════════════════════════
            ParsedInstruction::M { op, dest, base, offset, .. } => {
                let base_val = self.reg(base);
                let addr = self.resolve_m_offset(base_val, &offset);

                match op {
                    MemOp::LD => {
                        let val = self.mem_read_byte(addr);
                        self.set_reg(dest, val);
                    }
                    MemOp::ST => {
                        let val = self.reg(dest);
                        self.mem_write_byte(addr, val);
                    }
                }
            }

            // ═══════════════════════════════════════════════════════════
            //  BI-type: Branch with 8-bit immediate
            // ═══════════════════════════════════════════════════════════
            ParsedInstruction::BI { absolute, cond, operand, .. } => {
                if self.condition_met(cond) {
                    let target = match operand {
                        Operand::Immediate(addr) => addr,
                        Operand::Label(_) => panic!("Unresolved label in simulator"),
                    };
                    if absolute {
                        self.pc = target as Addr;
                    } else {
                        // Relative: offset is signed 8-bit, applied to PC
                        // (PC already advanced past this instruction)
                        let offset = target as u8 as i8;
                        self.pc = (self.pc as i64 + offset as i64) as Addr;
                    }
                    self.sync_pc_to_regs();
                }
            }

            // ═══════════════════════════════════════════════════════════
            //  BR-type: Branch with register pair target
            // ═══════════════════════════════════════════════════════════
            ParsedInstruction::BR { absolute, cond, source, .. } => {
                if self.condition_met(cond) {
                    let val = self.reg_pair(source.high, source.low);
                    if absolute {
                        self.pc = val as Addr;
                    } else {
                        let offset = val as i16;
                        self.pc = (self.pc as i64 + offset as i64) as Addr;
                    }
                    self.sync_pc_to_regs();
                }
            }

            // ═══════════════════════════════════════════════════════════
            //  S-type: Stack operations
            // ═══════════════════════════════════════════════════════════
            ParsedInstruction::S { op, operand, .. } => {
                match op {
                    StackOp::PUSH => {
                        let reg = operand & 0x0F;
                        let val = self.reg(reg);
                        let sp = self.sp().wrapping_sub(1);
                        self.set_sp(sp);
                        self.mem_write_byte(sp, val);
                    }
                    StackOp::POP => {
                        let reg = operand & 0x0F;
                        let sp = self.sp();
                        let val = self.mem_read_byte(sp);
                        self.set_sp(sp.wrapping_add(1));
                        self.set_reg(reg, val);
                    }
                    StackOp::SUBSP_IMM => {
                        let sp = self.sp().wrapping_sub(operand as u16);
                        self.set_sp(sp);
                    }
                    StackOp::SUBSP_REG => {
                        let reg = operand & 0x0F;
                        let val = self.reg(reg) as u16;
                        let sp = self.sp().wrapping_sub(val);
                        self.set_sp(sp);
                    }
                    StackOp::ADDSP_IMM => {
                        let sp = self.sp().wrapping_add(operand as u16);
                        self.set_sp(sp);
                    }
                    StackOp::ADDSP_REG => {
                        let reg = operand & 0x0F;
                        let val = self.reg(reg) as u16;
                        let sp = self.sp().wrapping_add(val);
                        self.set_sp(sp);
                    }
                }
            }

            // ═══════════════════════════════════════════════════════════
            //  P-type: Peek / Poke (SP-relative load/store)
            // ═══════════════════════════════════════════════════════════
            ParsedInstruction::P { op, register, offset, .. } => {
                let off = match offset {
                    Operand::Immediate(v) => v as u16,
                    Operand::Label(_) => panic!("Unresolved label in simulator"),
                };
                let addr = self.sp().wrapping_add(off);

                match op {
                    PeekPokeOp::PEEK => {
                        let val = self.mem_read_byte(addr);
                        self.set_reg(register, val);
                    }
                    PeekPokeOp::POKE => {
                        let val = self.reg(register);
                        self.mem_write_byte(addr, val);
                    }
                }
            }

            // ═══════════════════════════════════════════════════════════
            //  X-type: Extended / system instructions (privileged)
            // ═══════════════════════════════════════════════════════════
            ParsedInstruction::X { op, operand, .. } => {
                match op {
                    XTypeOp::SYSC => {
                        let _syscall_num = match operand {
                            XOperand::Immediate(n) => n,
                            _ => 0,
                        };
                        // Syscall handling is system-specific; trap into
                        // supervisor mode. For now this is a no-op stub.
                    }
                    XTypeOp::ERET => {
                        // Return from exception — restore PC and privilege.
                        // Full implementation requires saved-state registers;
                        // stubbed for now.
                    }
                    XTypeOp::HALT => {
                        self.halted = true;
                    }
                    // Cache control — no-ops in a simple simulator
                    XTypeOp::ICINV | XTypeOp::DCINV | XTypeOp::DCCLEAN | XTypeOp::FLUSH => {}
                }
            }
        }
    }
}