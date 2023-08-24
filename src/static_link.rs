use crate::common;
use crate::common::Context;
use crate::error::LinkerError;

const EXE_BASE: u64 = 0x400000;

const SECTIONS: &[&str] = &[".text", ".data"];

pub fn statically_link_files(input_files: Vec<String>, _output: String) -> Result<(), LinkerError> {
    let mut context = Context::default();
    let out_sections: dashmap::DashMap<String, common::Section> = Default::default();

    // First,
    for p in &input_files {
        common::load_object_file(p, &mut context)?;

        let file = context.files.last().unwrap();

        for sym in &file.symbols {
            if sym.global {
                let cloned_sym = sym.clone();

                if sym.defined {
                    if let Some(existing_sym) = context.global_symbols.get(&sym.name) {
                        // If there is already a defined symbol with the same name that is strong
                        if existing_sym.strong && existing_sym.defined {
                            log::error!("Multiple definitions of symbol `{}`", existing_sym.name);
                        }
                    }

                    context
                        .global_symbols
                        .insert(cloned_sym.name.clone(), cloned_sym);
                }
            }
        }
    }

    let mut curr_address = EXE_BASE;

    for section_name in SECTIONS {
        for file in &mut context.files {
            if let Some(section) = file
                .sections
                .clone()
                .iter_mut()
                .find(|section| section.name == *section_name)
            {
                file.sections[section.index.0].address = curr_address;

                if let Some(mut out_section) = out_sections.get_mut(&section_name.to_string()) {
                    out_section.data.extend(&section.data);
                    curr_address += section.data.len() as u64;
                } else {
                    let mut sec_copy = section.clone();
                    sec_copy.address = curr_address;

                    out_sections.insert(section_name.to_string(), sec_copy);
                    curr_address += section.data.len() as u64;
                }
            }
        }
    }

    for sec in out_sections {
        log::info!("{:x?}", sec);
    }

    for sym in context.global_symbols {
        if !sym.1.defined {
            log::error!("Undefined symbol `{}`", sym.1.name);
        }

        //log::info!("{:x?}", sym.1);

        log::info!(
            "symbol {} at {:x}",
            sym.1.name,
            sym.1.address
                + context.files[sym.1.object_id].sections[sym.1.sec_index.unwrap().0].address,
        );
    }

    Ok(())
}
