use std::ops::Range;

use crate::system::{Addr, Data};

pub struct BusMapping {
    pub range: Range<Addr>,
    pub device: Box<dyn BusDevice>,
}

pub struct Bus {
    pub mappings: Vec<BusMapping>,
}

impl Bus {
    pub fn read(&self, addr: Addr, size: usize) -> Data {

        for mapping in &self.mappings {
            if mapping.range.contains(&addr) {
                return mapping.device.read(addr - mapping.range.start, size)
            }
        }

        panic!("Bus fault at {:#x}", addr);
    }

    pub fn write(&mut self, addr: Addr, data: Data, size: usize) {

        for mapping in &mut self.mappings {
            if mapping.range.contains(&addr) {
                mapping.device.write(addr - mapping.range.start, data);
                return;
            }
        }
    }
}


pub trait BusDevice {
    fn read(&self, addr: Addr, size: usize) -> Data;
    fn write(&mut self, addr: Addr, data: Data);
}

pub trait BusMaster {
    fn read(&self, addr: Addr, size: usize) -> Data;
    fn write(&mut self, addr: Addr, data: Data);
}