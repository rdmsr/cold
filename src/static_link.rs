use crate::common::Context;
use crate::common::{self, ObjectFile};
use crate::error::LinkerError;

const EXECUTABLE_BASE: u64 = 0x400000;
const EXECUTABLE_SECTIONS: &[&str] = &[".text", ".data"];

pub fn statically_link_files(input_files: Vec<String>, _output: String) -> Result<(), LinkerError> {
    let mut context = Context::default();
    let out_sections: dashmap::DashMap<String, common::Section> = Default::default();

    for path in &input_files {
        let object = ObjectFile::new(path, &mut context)?;
        context.object_files.push(object);

        let object = context.object_files.last().unwrap();

        for symbol in &object.symbols {
            if symbol.global && symbol.defined {
                if !symbol.is_unique_in(&context) {
                    return Err(LinkerError::MultipleDefinitions(symbol.name.to_owned()));
                }

                let symbol = symbol.clone();
                context.global_symbols.insert(symbol.name.clone(), symbol);
            }
        }
    }

    let mut curr_address = EXECUTABLE_BASE;

    for &section_name in EXECUTABLE_SECTIONS {
        for file in &mut context.object_files {
            if let Some(mut section) = file
                .sections
                .iter()
                .find(|s| s.name == section_name)
                .cloned()
            {
                file.sections[section.index].address = curr_address;

                if let Some(mut out_section) = out_sections.get_mut(section_name) {
                    out_section.data.extend(&section.data);
                    curr_address += section.data.len() as u64;
                } else {
                    section.address = curr_address;

                    out_sections.insert(section_name.to_owned(), section.clone());
                    curr_address += section.data.len() as u64;

                    log::info!("{:x?}", section);
                }
            }
        }
    }

    for (name, symbol) in context.global_symbols {
        if !symbol.defined {
            log::error!("Undefined symbol `{}`", name);
        }

        let object_file = &context.object_files[symbol.object_id];
        let section = &object_file.sections[symbol.section_index.unwrap()];

        log::info!("Symbol {} at {:x}", name, symbol.address + section.address,);
    }

    Ok(())
}
