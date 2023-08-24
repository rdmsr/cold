use crate::error::LinkerError;
use dashmap::DashMap;
use memmap2::Mmap;
use object::{
    Object, ObjectKind, ObjectSection, ObjectSymbol, RelocationEncoding, RelocationKind, SymbolKind,
};
use std::fs;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub file: String,
    pub address: u64,
    pub kind: SymbolKind,
    pub defined: bool,
    pub global: bool,
    pub strong: bool,
    pub section_index: Option<usize>,
    pub object_id: usize,
}

impl Symbol {
    fn new(path: &str, symbol: &object::Symbol, object_id: usize) -> Self {
        Symbol {
            name: symbol.name().unwrap().to_owned(),
            address: symbol.address(),
            kind: symbol.kind(),
            defined: !symbol.is_undefined(),
            global: symbol.is_global(),
            file: path.to_owned(),
            strong: !symbol.is_weak(),
            section_index: symbol.section_index().map(|i| i.0),
            object_id,
        }
    }

    pub fn is_unique_in(&self, context: &Context) -> bool {
        if let Some(existing_sym) = context.global_symbols.get(&self.name) {
            !(existing_sym.strong && existing_sym.defined)
        } else {
            true
        }
    }
}

#[derive(Debug)]
pub struct Relocation {
    pub addend: i64,
    pub offset: u64,
    pub kind: RelocationKind,
    pub encoding: RelocationEncoding,
    pub object_id: usize,
}

impl Relocation {
    fn new(offset: u64, relocation: &object::Relocation, object_id: usize) -> Self {
        Relocation {
            addend: relocation.addend(),
            kind: relocation.kind(),
            offset,
            encoding: relocation.encoding(),
            object_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Section {
    pub data: Vec<u8>,
    pub name: String,
    pub index: usize,
    pub address: u64,
    pub object_id: usize,
}

impl Section {
    fn new(section: &object::Section, object_id: usize) -> Self {
        Section {
            // FIXME: this is slow.
            data: section.data().unwrap().to_vec(),
            name: section.name().unwrap().to_owned(),
            index: section.index().0,
            address: section.address(),
            object_id,
        }
    }
}

#[derive(Debug, Default)]
pub struct ObjectFile {
    pub symbols: Vec<Symbol>,
    pub relocations: Vec<Relocation>,
    pub sections: Vec<Section>,
}

impl ObjectFile {
    pub fn new(path: &str, context: &mut Context) -> Result<Self, LinkerError> {
        let file = fs::File::open(path).map_err(|e| LinkerError::IOError(path.to_owned(), e))?;

        let mmap = unsafe { memmap2::Mmap::map(&file).unwrap() };
        context.mmaps.push(mmap);

        let file = object::File::parse(context.mmaps.last().unwrap().as_ref())
            .map_err(|e| LinkerError::ParseError(path.to_owned(), e))?;

        if file.kind() != ObjectKind::Relocatable {
            return Err(LinkerError::InvalidFileType(path.to_owned()));
        }

        let mut object_file = ObjectFile::default();
        let object_id = context.object_files.len();

        for symbol in file.symbols() {
            object_file
                .symbols
                .push(Symbol::new(path, &symbol, object_id))
        }

        for section in file.sections() {
            object_file
                .sections
                .insert(section.index().0, Section::new(&section, object_id));

            for (offset, relocation) in section.relocations() {
                object_file
                    .relocations
                    .push(Relocation::new(offset, &relocation, object_id));
            }
        }

        Ok(object_file)
    }
}

#[derive(Debug, Default)]
pub struct Context {
    pub mmaps: Vec<Mmap>,
    pub object_files: Vec<ObjectFile>,
    pub global_symbols: DashMap<String, Symbol>,
}
