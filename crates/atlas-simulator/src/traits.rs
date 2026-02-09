/// A clockable component (CPU, timer, peripheral, etc.)
pub trait Clockable {
    type Error;

    /// Advance by one tick. Returns cycles consumed.
    fn tick(&mut self, bus: &mut dyn Bus) -> Result<u64, Self::Error>;
}

/// Address-mapped system bus
pub trait Bus {
    fn read(&self, addr: u32) -> u8;
    fn write(&mut self, addr: u32, val: u8);
}

/// A device mapped onto the bus
pub trait Device {
    /// Whether this device responds to the given address
    fn contains(&self, addr: u32) -> bool;
    fn read(&self, addr: u32) -> u8;
    fn write(&mut self, addr: u32, val: u8);
}
