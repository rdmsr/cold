use crate::common;
use crate::common::Context;
use crate::error::LinkerError;
use rayon::prelude::*;
use std::sync::Arc;

pub fn statically_link_files(input_files: Vec<String>, _output: String) -> Result<(), LinkerError> {
    let context = Arc::new(std::sync::RwLock::new(Context::default()));

    input_files.par_iter().try_for_each(|p| {
        let file = common::load_object_file(p, &mut context.write().unwrap())?;

        file.symbols.par_iter().for_each(|sym| {
            if sym.global {
                let ctx = &context.write().unwrap();
                let cloned_sym = sym.clone();

                if sym.defined {
                    if let Some(existing_sym) = ctx.symbols.get(&sym.name) {
                        // If there is already a defined symbol with the same name that is strong
                        if existing_sym.strong && existing_sym.defined {
                            log::error!("Multiple definitions of symbol `{}`", existing_sym.name);
                        }
                    }

                    ctx.symbols.insert(cloned_sym.name.clone(), cloned_sym);
                }
            }
        });

        for section in file.sections {
            context.write().unwrap().sections.push(section);
        }

        Ok(())
    })?;

    for sym in &context.read().unwrap().symbols {
        if !sym.defined {
            log::error!("Undefined symbol `{}`", sym.name);
        }

        log::info!("{:?}", *sym);
    }

    Ok(())
}
