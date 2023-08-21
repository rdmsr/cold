use object::read::Error as ParseError;
use std::io::Error;

#[derive(Debug)]
pub enum LinkerError {
    InvalidFileType(String),
    ParseError(String, ParseError),
    IOError(String, Error),
    UndefinedSymbol(String, String),
}

pub fn report_error_impl(msg: &str) {
    println!("\x1b[1;31merror:\x1b[0m {}", msg)
}

macro_rules! report_error {
    ($($t:tt)*) => {{
        report_error_impl(&format!($($t)*));
    }};
}

pub fn handle_error(e: LinkerError) {
    match e {
        LinkerError::IOError(f, e) => report_error!("IO error: {}: {}", f, e.to_string()),
        LinkerError::InvalidFileType(f) => {
            report_error!("{} cannot be linked (is the file relocatable?)", f)
        }

        LinkerError::ParseError(f, e) => {
            report_error!("Object file parsing error: {}: {}", f, e.to_string())
        }

        LinkerError::UndefinedSymbol(f, name) => {
            report_error!("Undefined symbol `{}` referenced in {}", name, f);
        }
    }
}
