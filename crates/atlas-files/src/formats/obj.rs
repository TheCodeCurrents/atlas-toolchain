use std::fs::File;
use std::io::Write;

use crate::formats::FileFormat;

// constants
const MAGIC: &[u8; 4] = b"ATOB";

pub struct Section {
    pub name: String,
    pub start: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub enum SymbolBinding {
    Local = 0,
    Global = 1,
}

pub struct Symbol {
    pub name: String,
    pub value: u32,                  // offset in section
    pub section: Option<String>,     // None = undefined (import)
    pub binding: SymbolBinding,
}


pub struct Relocation {
    pub offset: u32,
    pub symbol: String,
    pub addend: i32,
    pub section: String,
}

pub struct ObjectFile {
    pub sections: Vec<Section>,
    pub symbols: Vec<Symbol>,
    pub relocations: Vec<Relocation>,

    pub version: u32,
}

impl FileFormat for ObjectFile {
    fn from_file(path: &str) -> std::io::Result<Self> where Self: Sized {
        use std::io::{Read, Error, ErrorKind};
        let mut file = File::open(path)?;
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid magic number"));
        }

        let mut version_bytes = [0u8; 4];
        file.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);

        let mut count_bytes = [0u8; 4];
        file.read_exact(&mut count_bytes)?;
        let section_count = u32::from_le_bytes(count_bytes);
        file.read_exact(&mut count_bytes)?;
        let symbol_count = u32::from_le_bytes(count_bytes);
        file.read_exact(&mut count_bytes)?;
        let relocation_count = u32::from_le_bytes(count_bytes);

        // read sections
        let mut sections = Vec::with_capacity(section_count as usize);
        for _ in 0..section_count {
            file.read_exact(&mut count_bytes)?;
            let name_len = u32::from_le_bytes(count_bytes) as usize;
            let mut name_bytes = vec![0u8; name_len];
            file.read_exact(&mut name_bytes)?;
            let name = String::from_utf8(name_bytes).map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8 in section name"))?;

            let mut start_bytes = [0u8; 4];
            file.read_exact(&mut start_bytes)?;
            let start = u32::from_le_bytes(start_bytes);

            let mut size_bytes = [0u8; 4];
            file.read_exact(&mut size_bytes)?;
            let size = u32::from_le_bytes(size_bytes);

            let mut data = vec![0u8; size as usize];
            file.read_exact(&mut data)?;

            sections.push(Section { name, start, data });
        }

        // read symbols
        let mut symbols = Vec::with_capacity(symbol_count as usize);
        for _ in 0..symbol_count {
            file.read_exact(&mut count_bytes)?;
            let name_len = u32::from_le_bytes(count_bytes) as usize;
            let mut name_bytes = vec![0u8; name_len];
            file.read_exact(&mut name_bytes)?;
            let name = String::from_utf8(name_bytes).map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8 in symbol name"))?;

            let mut value_bytes = [0u8; 4];
            file.read_exact(&mut value_bytes)?;
            let value = u32::from_le_bytes(value_bytes);

            let mut section_flag = [0u8; 1];
            file.read_exact(&mut section_flag)?;
            let section = if section_flag[0] == 1 {
                file.read_exact(&mut count_bytes)?;
                let section_len = u32::from_le_bytes(count_bytes) as usize;
                let mut section_bytes = vec![0u8; section_len];
                file.read_exact(&mut section_bytes)?;
                Some(String::from_utf8(section_bytes).map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8 in symbol section"))?)
            } else {
                None
            };

            let mut binding_byte = [0u8; 1];
            file.read_exact(&mut binding_byte)?;
            let binding = match binding_byte[0] {
                0 => SymbolBinding::Local,
                1 => SymbolBinding::Global,
                _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid symbol binding")),
            };

            symbols.push(Symbol { name, value, section, binding });
        }

        // read relocations
        let mut relocations = Vec::with_capacity(relocation_count as usize);
        for _ in 0..relocation_count {
            let mut offset_bytes = [0u8; 4];
            file.read_exact(&mut offset_bytes)?;
            let offset = u32::from_le_bytes(offset_bytes);

            file.read_exact(&mut count_bytes)?;
            let symbol_len = u32::from_le_bytes(count_bytes) as usize;
            let mut symbol_bytes = vec![0u8; symbol_len];
            file.read_exact(&mut symbol_bytes)?;
            let symbol = String::from_utf8(symbol_bytes).map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8 in relocation symbol"))?;

            let mut addend_bytes = [0u8; 4];
            file.read_exact(&mut addend_bytes)?;
            let addend = i32::from_le_bytes(addend_bytes);

            file.read_exact(&mut count_bytes)?;
            let section_len = u32::from_le_bytes(count_bytes) as usize;
            let mut section_bytes = vec![0u8; section_len];
            file.read_exact(&mut section_bytes)?;
            let section = String::from_utf8(section_bytes).map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8 in relocation section"))?;

            relocations.push(Relocation { offset, symbol, addend, section });
        }

        Ok(Self {
            sections,
            symbols,
            relocations,
            version,
        })
    }

    fn to_file(&self, path: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;

        // write identifier and version
        file.write_all(MAGIC)?;
        file.write_all(&(self.version.to_le_bytes()))?;

        // write counts
        let section_count = self.sections.len() as u32;
        let symbol_count = self.symbols.len() as u32;
        let relocation_count = self.relocations.len() as u32;
        file.write_all(&section_count.to_le_bytes())?;
        file.write_all(&symbol_count.to_le_bytes())?;
        file.write_all(&relocation_count.to_le_bytes())?;

        // write sections
        for section in &self.sections {
            let name_bytes = section.name.as_bytes();
            let name_len = name_bytes.len() as u32;
            file.write_all(&name_len.to_le_bytes())?;
            file.write_all(name_bytes)?;
            file.write_all(&section.start.to_le_bytes())?;
            file.write_all(&(section.data.len() as u32).to_le_bytes())?;
            file.write_all(&section.data)?;
        }

        // write symbols
        for symbol in &self.symbols {
            let name_bytes = symbol.name.as_bytes();
            let name_len = name_bytes.len() as u32;
            file.write_all(&name_len.to_le_bytes())?;
            file.write_all(name_bytes)?;

            file.write_all(&symbol.value.to_le_bytes())?;

            match &symbol.section {
                Some(section_name) => {
                    file.write_all(&1u8.to_le_bytes())?;
                    let section_bytes = section_name.as_bytes();
                    let section_len = section_bytes.len() as u32;
                    file.write_all(&section_len.to_le_bytes())?;
                    file.write_all(section_bytes)?;
                }
                None => {
                    file.write_all(&0u8.to_le_bytes())?;
                }
            }

            file.write_all(&(symbol.binding as u8).to_le_bytes())?;
        }

        // write relocations
        for reloc in &self.relocations {
            file.write_all(&reloc.offset.to_le_bytes())?;
            let symbol_bytes = reloc.symbol.as_bytes();
            let symbol_len = symbol_bytes.len() as u32;
            file.write_all(&symbol_len.to_le_bytes())?;
            file.write_all(symbol_bytes)?;
            file.write_all(&reloc.addend.to_le_bytes())?;
            let section_bytes = reloc.section.as_bytes();
            let section_len = section_bytes.len() as u32;
            file.write_all(&section_len.to_le_bytes())?;
            file.write_all(section_bytes)?;
        }

        Ok(())
    }

    fn format(&self) -> super::FileType {
        super::FileType::Obj
    }
}