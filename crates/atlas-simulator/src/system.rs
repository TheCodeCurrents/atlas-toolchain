use crate::cpu::CPU;


pub type Addr = u64;
pub type Data = u64;

pub struct System {
    pub name: String,
    pub cpu: Box<dyn CPU>,
}