use crate::traits::Device;

/// Generic RAM device with configurable base address and size
pub struct Ram {
    base: u32,
    data: Vec<u8>,
}

impl Ram {
    pub fn new(base: u32, size: usize) -> Self {
        Self {
            base,
            data: vec![0; size],
        }
    }

    pub fn load(&mut self, offset: usize, bytes: &[u8]) {
        self.data[offset..offset + bytes.len()].copy_from_slice(bytes);
    }
}

impl Device for Ram {
    fn contains(&self, addr: u32) -> bool {
        let offset = addr.wrapping_sub(self.base);
        (offset as usize) < self.data.len()
    }

    fn read(&self, addr: u32) -> u8 {
        self.data[(addr - self.base) as usize]
    }

    fn write(&mut self, addr: u32, val: u8) {
        self.data[(addr - self.base) as usize] = val;
    }
}
