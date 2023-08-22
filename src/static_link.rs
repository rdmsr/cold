use crate::common;
use crate::error::LinkerError;
use dashmap::DashMap;
use object::write;
use object::write::elf;
use rayon::prelude::*;
use std::fs::File;
use std::io::BufWriter;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub fn statically_link_files(input_files: Vec<String>, output: String) -> Result<(), LinkerError> {
    let out_file = BufWriter::new(File::create(output).unwrap());
    let mut writer = write::StreamingBuffer::new(out_file);
    let mut _elf_writer = elf::Writer::new(object::Endianness::Little, true, &mut writer);

    let symbols_map: DashMap<String, common::Symbol> = DashMap::new();
    let undefined_symbols_num = Arc::new(AtomicUsize::new(0));

    input_files.par_iter().try_for_each(|p| {
        let file = common::load_object_file(p)?;

        file.symbols.par_iter().for_each(|sym| {
            if !sym.defined {
                undefined_symbols_num.fetch_add(1, Ordering::SeqCst);
            } else if undefined_symbols_num.load(Ordering::SeqCst) != 0 {
                undefined_symbols_num.fetch_add(1, Ordering::SeqCst);
            }

            if sym.global {
                symbols_map.insert(sym.name.clone(), sym.clone());
            }
        });

        Ok(())
    })?;

    if undefined_symbols_num.load(Ordering::SeqCst) > 0 {
        for sym in symbols_map {
            if !sym.1.defined {
                return Err(LinkerError::UndefinedSymbol(sym.1.file, sym.0));
            }
        }
    }

    Ok(())
}
