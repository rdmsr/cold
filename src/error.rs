use object::read;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum LinkerError {
    InvalidFileType(String),
    ParseError(String, read::Error),
    IOError(String, io::Error),
    UndefinedSymbol(String, String),
}

impl fmt::Display for LinkerError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkerError::InvalidFileType(f) => {
                write!(fmt, "{} cannot be linked (is the file relocatable?)", f)
            }
            LinkerError::ParseError(f, e) => {
                write!(fmt, "Object file parsing error: {}: {}", f, e)
            }
            LinkerError::IOError(f, e) => write!(fmt, "IO error: {f}: {e}"),
            LinkerError::UndefinedSymbol(f, name) => {
                write!(fmt, "Undefined symbol `{}` referenced in {}", name, f)
            }
        }
    }
}

impl LinkerError {
    pub fn report(&self) {
        eprintln!("\x1b[1;31merror:\x1b[0m {}", self)
    }
}
