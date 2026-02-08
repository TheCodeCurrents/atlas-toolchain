pub mod formats;

pub use formats::obj::{ObjectFile, Symbol};

pub use formats::FileFormat;

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs;

	#[test]
	fn test_objectfile_roundtrip() {
		let tmpfile = "test_objfile.atob";
		let section = formats::obj::Section {
			name: "text".to_string(),
			start: 0x1000,
			size: 4,
			data: vec![1, 2, 3, 4],
		};
		let symbol = Symbol {
			name: "main".to_string(),
			addr: Some(0x1000),
			section: "text".to_string(),
		};
		let reloc = formats::obj::Relocation {
			offset: 2,
			symbol: "main".to_string(),
			addend: -1,
		};
		let obj = ObjectFile {
			sections: vec![section],
			symbols: vec![symbol],
			relocations: vec![reloc],
			version: 42,
		};

		obj.to_file(tmpfile).expect("write failed");
		let obj2 = ObjectFile::from_file(tmpfile).expect("read failed");
		fs::remove_file(tmpfile).ok();

		assert_eq!(obj.version, obj2.version);
		assert_eq!(obj.sections.len(), obj2.sections.len());
		assert_eq!(obj.symbols.len(), obj2.symbols.len());
		assert_eq!(obj.relocations.len(), obj2.relocations.len());
		assert_eq!(obj.sections[0].name, obj2.sections[0].name);
		assert_eq!(obj.sections[0].data, obj2.sections[0].data);
		assert_eq!(obj.symbols[0].name, obj2.symbols[0].name);
		assert_eq!(obj.symbols[0].addr, obj2.symbols[0].addr);
		assert_eq!(obj.relocations[0].symbol, obj2.relocations[0].symbol);
		assert_eq!(obj.relocations[0].addend, obj2.relocations[0].addend);
	}
}

