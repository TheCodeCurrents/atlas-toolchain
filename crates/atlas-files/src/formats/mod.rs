pub mod bin;
pub mod elf;
pub mod hex;
pub mod obj;

pub enum FileType {
    Bin,
    Elf,
    Hex,
    Obj,
}

pub trait FileFormat {
    /// Return a canonical format enum or name
    fn format(&self) -> FileType;

    /// Serialize to a file (or bytes)
    fn to_file(&self, path: &str) -> std::io::Result<()>;

    /// Deserialize from a file (or bytes)
    fn from_file(path: &str) -> std::io::Result<Self> where Self: Sized;
}