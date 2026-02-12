
pub mod system;
pub mod cpu;
pub mod bus;
pub mod architectures;


pub trait Clockable {
    fn tick(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        
    }
}