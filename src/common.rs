use crate::error::LinkerError;
use dashmap::DashMap;
use object::{Object, ObjectSection, ObjectSymbol, SectionIndex};

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub file: String,
    pub address: u64,
    pub kind: object::SymbolKind,
    pub defined: bool,
    pub global: bool,
    pub strong: bool,
    pub sec_index: Option<SectionIndex>,
    pub object_id: usize,
}

#[derive(Debug)]
pub struct Relocation {
    pub addend: i64,
    pub offset: u64,
    pub kind: object::RelocationKind,
    pub encoding: object::RelocationEncoding,
    pub object_id: usize,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub data: Vec<u8>,
    pub name: String,
    pub index: SectionIndex,
    pub address: u64,
    pub object_id: usize,
}

#[derive(Debug, Default)]
pub struct ObjectFile {
    pub symbols: Vec<Symbol>,
    pub relocations: Vec<Relocation>,
    pub sections: Vec<Section>,
}

#[derive(Debug, Default)]
pub struct Context {
    pub mmaps: Vec<memmap2::Mmap>,
    pub files: Vec<ObjectFile>,
    pub global_symbols: DashMap<String, Symbol>,
}

pub fn load_object_file(path: &str, context: &mut Context) -> Result<(), LinkerError> {
    let file = std::fs::File::open(path).map_err(|e| LinkerError::IOError(path.to_string(), e))?;

    let mmap = unsafe { memmap2::Mmap::map(&file).unwrap() };

    context.mmaps.push(mmap);

    let obj = object::File::parse(context.mmaps.last().unwrap().as_ref())
        .map_err(|e| LinkerError::ParseError(path.to_string(), e))?;

    if obj.kind() != object::ObjectKind::Relocatable {
        return Err(LinkerError::InvalidFileType(path.to_string()));
    }

    let mut ret = ObjectFile::default();
    let object_id = context.files.len();

    for sym in obj.symbols() {
        ret.symbols.push(Symbol {
            name: sym.name().unwrap().to_string(),
            address: sym.address(),
            kind: sym.kind(),
            defined: !sym.is_undefined(),
            global: sym.is_global(),
            file: path.to_string(),
            strong: !sym.is_weak(),
            sec_index: sym.section_index(),
            object_id,
        })
    }

    for section in obj.sections() {
        ret.sections.insert(
            section.index().0,
            Section {
                // FIXME: this is slow
                data: section.data().unwrap().to_vec(),
                name: section.name().unwrap().to_string(),
                index: section.index(),
                address: section.address(),
                object_id,
            },
        );

        for reloc in section.relocations() {
            ret.relocations.push(Relocation {
                addend: reloc.1.addend(),
                kind: reloc.1.kind(),
                offset: reloc.0,
                encoding: reloc.1.encoding(),
                object_id,
            });
        }
    }

    context.files.push(ret);
    Ok(())
}
