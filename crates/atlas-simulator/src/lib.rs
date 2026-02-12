
    pub mod system;
    pub mod cpu;
    pub mod bus;
    pub mod architectures;


    pub trait Clockable {
        fn tick(&mut self);
    }

    #[cfg(test)]
    mod tests {
        use atlas_isa::opcode::ImmOp;
        use atlas_isa::operands::Operand;
        use atlas_isa::ParsedInstruction;

        use crate::architectures::atlas8::core::Atlas8Core;
        use crate::bus::BusMaster;
        use crate::system::{Addr, Data};

        /// Minimal stub bus for unit-testing instructions that don't touch memory.
        struct NullBus;

        impl BusMaster for NullBus {
            fn read(&self, _addr: Addr, _size: usize) -> Data { 0 }
            fn write(&mut self, _addr: Addr, _data: Data) {}
        }

        #[test]
        fn ldi_r1_0x55() {
            let mut core = Atlas8Core::new(Box::new(NullBus));

            core.execute_instruction(ParsedInstruction::I {
                op: ImmOp::LDI,
                dest: 1,
                immediate: Operand::Immediate(0x55),
                line: 0,
                source_file: None,
            });

            assert_eq!(core.regs[1], 0x55);
        }
    }