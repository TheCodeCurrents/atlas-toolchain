use std::fmt;

#[derive(Debug)]
pub enum SimulatorError {
    Halted,
    DecodeError { pc: u32, message: String },
    InvalidMemoryAccess { addr: u32 },
}

impl fmt::Display for SimulatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Halted => write!(f, "CPU is halted"),
            Self::DecodeError { pc, message } => {
                write!(f, "decode error at 0x{pc:04X}: {message}")
            }
            Self::InvalidMemoryAccess { addr } => {
                write!(f, "invalid memory access at 0x{addr:04X}")
            }
        }
    }
}

impl std::error::Error for SimulatorError {}
