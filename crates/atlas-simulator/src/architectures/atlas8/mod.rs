use atlas_isa::ParsedInstruction;

use crate::{bus::BusMaster, cpu::CPU, system::Addr};


pub struct Atlas8Core {
    pub pc: Addr,
    pub bus: Box<dyn BusMaster>
}

impl CPU for Atlas8Core {
    fn tick(&mut self) {
        let inst_bytes = self.bus.read(self.pc, 2);
        self.pc += 2;

        let inst = if let Ok(inst) = ParsedInstruction::decode(inst_bytes as u16) {
            inst
        } else {
            panic!("Invalid instruction: {:#x}", inst_bytes);
        };
        
        print!("tick! Instruction: {:?}", inst);
    }
}