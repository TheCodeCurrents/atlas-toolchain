use std::fs::File;
use std::io::{self, Write};

use crate::formats::FileFormat;

// constants
const MAGIC: &[u8; 4] = b"ATOB";

pub struct Section {
    pub name: String,
    pub start: u32,
    pub size: u32,
    pub data: Vec<u8>,
}

pub struct Symbol {
    pub name: String,
    pub addr: Option<u32>,
    pub section: String,
}

pub struct Relocation {
    pub offset: u32,
    pub symbol: String,
    pub addend: i32,
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

            sections.push(Section { name, start, size, data });
        }

        // read symbols
        let mut symbols = Vec::with_capacity(symbol_count as usize);
        for _ in 0..symbol_count {
            file.read_exact(&mut count_bytes)?;
            let name_len = u32::from_le_bytes(count_bytes) as usize;
            let mut name_bytes = vec![0u8; name_len];
            file.read_exact(&mut name_bytes)?;
            let name = String::from_utf8(name_bytes).map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8 in symbol name"))?;

            let mut addr_flag = [0u8; 1];
            file.read_exact(&mut addr_flag)?;
            let addr = if addr_flag[0] == 1 {
                let mut addr_bytes = [0u8; 4];
                file.read_exact(&mut addr_bytes)?;
                Some(u32::from_le_bytes(addr_bytes))
            } else {
                None
            };

            file.read_exact(&mut count_bytes)?;
            let section_len = u32::from_le_bytes(count_bytes) as usize;
            let mut section_bytes = vec![0u8; section_len];
            file.read_exact(&mut section_bytes)?;
            let section = String::from_utf8(section_bytes).map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8 in symbol section"))?;

            symbols.push(Symbol { name, addr, section });
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

            relocations.push(Relocation { offset, symbol, addend });
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
            file.write_all(&section.size.to_le_bytes())?;
            file.write_all(&section.data)?;
        }

        // write symbols
        for symbol in &self.symbols {
            let name_bytes = symbol.name.as_bytes();
            let name_len = name_bytes.len() as u32;
            file.write_all(&name_len.to_le_bytes())?;
            file.write_all(name_bytes)?;
            match symbol.addr {
                Some(addr) => {
                    file.write_all(&1u8.to_le_bytes())?; // has address
                    file.write_all(&addr.to_le_bytes())?;
                }
                None => {
                    file.write_all(&0u8.to_le_bytes())?; // no address
                }
            }
            let section_bytes = symbol.section.as_bytes();
            let section_len = section_bytes.len() as u32;
            file.write_all(&section_len.to_le_bytes())?;
            file.write_all(section_bytes)?;
        }

        // write relocations
        for reloc in &self.relocations {
            file.write_all(&reloc.offset.to_le_bytes())?;
            let symbol_bytes = reloc.symbol.as_bytes();
            let symbol_len = symbol_bytes.len() as u32;
            file.write_all(&symbol_len.to_le_bytes())?;
            file.write_all(symbol_bytes)?;
            file.write_all(&reloc.addend.to_le_bytes())?;
        }

        Ok(())
    }

    fn format(&self) -> super::FileType {
        super::FileType::Obj
    }
}