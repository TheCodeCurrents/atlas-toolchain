use crate::error::SimulatorError;
use crate::traits::{Bus, Clockable};

use super::cpu::Cpu;

impl Clockable for Cpu {
    type Error = SimulatorError;

    fn tick(&mut self, bus: &mut dyn Bus) -> Result<u64, Self::Error> {
        todo!("fetch-decode-execute cycle")
    }
}
