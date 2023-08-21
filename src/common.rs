use crate::error::LinkerError;
use object::{Object, ObjectSection, ObjectSymbol};

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub file: String,
    pub address: u64,
    pub kind: object::SymbolKind,
    pub defined: bool,
    pub global: bool,
}

#[derive(Debug)]
pub struct Relocation {
    pub addend: i64,
    pub offset: u64,
    pub kind: object::RelocationKind,
    pub encoding: object::RelocationEncoding,
}

#[derive(Debug)]
pub struct Section {
    pub name: String,
}

#[derive(Debug, Default)]
pub struct ObjectFile {
    pub symbols: Vec<Symbol>,
    pub relocations: Vec<Relocation>,
    pub sections: Vec<Section>,
}

pub fn load_object_file(path: &String) -> Result<ObjectFile, LinkerError> {
    let file = match std::fs::File::open(path) {
        Err(e) => return Err(LinkerError::IOError(path.to_string(), e)),
        Ok(x) => x,
    };

    let mmap = unsafe { memmap2::Mmap::map(&file).unwrap() };

    let obj = match object::File::parse(&*mmap) {
        Ok(x) => x,
        Err(e) => return Err(LinkerError::ParseError(path.to_string(), e)),
    };

    if obj.kind() != object::ObjectKind::Relocatable {
        return Err(LinkerError::InvalidFileType(path.to_string()));
    }

    let mut ret = ObjectFile::default();

    for sym in obj.symbols() {
        ret.symbols.push(Symbol {
            name: sym.name().unwrap().to_string(),
            address: sym.address(),
            kind: sym.kind(),
            defined: !sym.is_undefined(),
            global: sym.is_global(),
            file: path.clone(),
        })
    }

    for section in obj.sections() {
        ret.sections.push(Section {
            name: section.name().unwrap().to_string(),
        });

        for reloc in section.relocations() {
            ret.relocations.push(Relocation {
                addend: reloc.1.addend(),
                kind: reloc.1.kind(),
                offset: reloc.0,
                encoding: reloc.1.encoding(),
            });
        }
    }

    Ok(ret)
}
