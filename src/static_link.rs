use crate::common;
use crate::error::LinkerError;
use dashmap::DashMap;
use object::write;
use object::write::elf;
use rayon::prelude::*;
use std::fs::File;
use std::io::BufWriter;
use std::sync::Arc;
use std::sync::Mutex;

pub fn statically_link_files(input_files: Vec<String>, output: String) -> Result<(), LinkerError> {
    let out_file = BufWriter::new(File::create(output).unwrap());
    let mut writer = write::StreamingBuffer::new(out_file);
    let mut _elf_writer = elf::Writer::new(object::Endianness::Little, true, &mut writer);

    let symbols_map: DashMap<String, common::Symbol> = DashMap::new();
    let undefined_symbols_num = Arc::new(Mutex::new(0));

    input_files.par_iter().try_for_each(|p| {
        let file = common::load_object_file(p)?;

        for sym in file.symbols {
            if !sym.defined {
                *undefined_symbols_num.lock().unwrap() += 1;
            } else if *undefined_symbols_num.lock().unwrap() != 0 {
                *undefined_symbols_num.lock().unwrap() -= 1;
            }

            if sym.global {
                symbols_map.insert(sym.clone().name, sym);
            }
        }

        Ok(())
    })?;

    if *undefined_symbols_num.lock().unwrap() > 0 {
        for sym in symbols_map {
            if !sym.1.defined {
                return Err(LinkerError::UndefinedSymbol(sym.1.file, sym.0));
            }
        }
    }

    Ok(())
}
