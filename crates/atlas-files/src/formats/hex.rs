//! Intel HEX (IHEX) file format writer.
//!
//! Produces files conforming to the Intel HEX format (`:LLAAAATT[DD…]CC`).
//! Only record types 00 (Data) and 01 (EOF) are emitted since the Atlas
//! address space fits in 16 bits.

use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::Write;

/// Maximum data bytes per record line (standard is 16, some tools use 32).
const BYTES_PER_LINE: usize = 16;

/// Format a byte slice as Intel HEX starting at the given base address.
pub fn to_ihex(data: &[u8], base_address: u16) -> String {
    let mut out = String::new();

    for (chunk_idx, chunk) in data.chunks(BYTES_PER_LINE).enumerate() {
        let address = base_address.wrapping_add((chunk_idx * BYTES_PER_LINE) as u16);
        let byte_count = chunk.len() as u8;
        let record_type: u8 = 0x00; // Data record

        // Start checksum accumulator
        let mut sum: u8 = 0;
        sum = sum.wrapping_add(byte_count);
        sum = sum.wrapping_add((address >> 8) as u8);
        sum = sum.wrapping_add(address as u8);
        sum = sum.wrapping_add(record_type);

        write!(out, ":{:02X}{:04X}{:02X}", byte_count, address, record_type).unwrap();

        for &b in chunk {
            write!(out, "{:02X}", b).unwrap();
            sum = sum.wrapping_add(b);
        }

        let checksum = (!sum).wrapping_add(1); // two's complement
        writeln!(out, "{:02X}", checksum).unwrap();
    }

    // EOF record
    out.push_str(":00000001FF\n");
    out
}

/// Write a byte slice as an Intel HEX file.
pub fn write_hex_file(path: &str, data: &[u8], base_address: u16) -> std::io::Result<()> {
    let hex = to_ihex(data, base_address);
    let mut file = File::create(path)?;
    file.write_all(hex.as_bytes())?;
    Ok(())
}

/// Parse an Intel HEX string back into raw bytes.
/// Returns the data bytes in linear address order.
pub fn from_ihex(hex: &str) -> Result<Vec<u8>, std::io::Error> {
    use std::io::{Error, ErrorKind};

    let mut result: Vec<u8> = Vec::new();

    for line in hex.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if !line.starts_with(':') {
            return Err(Error::new(ErrorKind::InvalidData, "Line does not start with ':'"));
        }
        let hex_str = &line[1..];
        if hex_str.len() < 10 {
            return Err(Error::new(ErrorKind::InvalidData, "Line too short"));
        }

        let byte_count = u8::from_str_radix(&hex_str[0..2], 16)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid byte count"))?;
        let record_type = u8::from_str_radix(&hex_str[6..8], 16)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid record type"))?;

        match record_type {
            0x00 => {
                // Data record
                let address = u16::from_str_radix(&hex_str[2..6], 16)
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid address"))?;
                let addr = address as usize;

                // Extend result buffer if needed
                let needed = addr + byte_count as usize;
                if result.len() < needed {
                    result.resize(needed, 0);
                }

                for i in 0..byte_count as usize {
                    let offset = 8 + i * 2;
                    let b = u8::from_str_radix(&hex_str[offset..offset + 2], 16)
                        .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid data byte"))?;
                    result[addr + i] = b;
                }
            }
            0x01 => break, // EOF
            _ => {} // Skip other record types
        }
    }

    Ok(result)
}

/// Read an Intel HEX file and return the raw bytes.
pub fn read_hex_file(path: &str) -> std::io::Result<Vec<u8>> {
    let contents = std::fs::read_to_string(path)?;
    from_ihex(&contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_produces_only_eof() {
        let hex = to_ihex(&[], 0);
        assert_eq!(hex, ":00000001FF\n");
    }

    #[test]
    fn single_byte() {
        let hex = to_ihex(&[0x5A], 0x0000);
        let lines: Vec<&str> = hex.lines().collect();
        assert_eq!(lines.len(), 2);
        // :01 0000 00 5A xx
        assert!(lines[0].starts_with(":01000000"));
        assert_eq!(lines[1], ":00000001FF");
    }

    #[test]
    fn checksum_is_correct() {
        // :02 0000 00 1110 -> sum = 02+00+00+00+11+10 = 23
        // checksum = (~0x23 + 1) & 0xFF = 0xDD
        let hex = to_ihex(&[0x11, 0x10], 0x0000);
        let first_line = hex.lines().next().unwrap();
        assert_eq!(first_line, ":020000001110DD");
    }

    #[test]
    fn respects_base_address() {
        let hex = to_ihex(&[0xAB], 0x1000);
        let first_line = hex.lines().next().unwrap();
        assert!(first_line.starts_with(":011000"));
    }
}
