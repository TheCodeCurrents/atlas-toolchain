use crate::traits::{Bus, Device};

/// System bus that dispatches reads/writes to registered devices by address
pub struct SystemBus {
    devices: Vec<Box<dyn Device>>,
}

impl SystemBus {
    pub fn new() -> Self {
        Self { devices: Vec::new() }
    }

    pub fn add_device(&mut self, device: Box<dyn Device>) {
        self.devices.push(device);
    }
}

impl Bus for SystemBus {
    fn read(&self, addr: u32) -> u8 {
        self.devices
            .iter()
            .find(|d| d.contains(addr))
            .map(|d| d.read(addr))
            .unwrap_or(0xFF)
    }

    fn write(&mut self, addr: u32, val: u8) {
        if let Some(d) = self.devices.iter_mut().find(|d| d.contains(addr)) {
            d.write(addr, val);
        }
    }
}
