
pub struct Cpu {
    pub registers: RegisterFile,
    pub flags: StatusFlags,
    pub mode: Mode,
    pub halted: bool,
}

pub struct StatusFlags {
    pub zero: bool,
    pub carry: bool,
    pub negative: bool,
    pub overflow: bool,
}

pub enum Mode {
    User,
    Supervisor,
}

pub struct RegisterFile {
    raw: [u8; 16],
}

impl RegisterFile {
    pub fn get(&self, index: usize) -> u8 {
        self.raw[index]
    }

    pub fn set(&mut self, index: usize, value: u8) {
        self.raw[index] = value;
    }

    pub fn tr(&self) -> u16 {
        u16::from_le_bytes([self.get(10), self.get(11)])
    }

    pub fn set_tr(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.set(10, bytes[0]);
        self.set(11, bytes[1]);
    }

    pub fn sp(&self) -> u16 {
        u16::from_le_bytes([self.get(12), self.get(13)])
    }

    pub fn set_sp(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.set(12, bytes[0]);
        self.set(13, bytes[1]);
    }

    pub fn pc(&self) -> u16 {
        u16::from_le_bytes([self.get(14), self.get(15)])
    }

    pub fn set_pc(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.set(14, bytes[0]);
        self.set(15, bytes[1]);
    }
}