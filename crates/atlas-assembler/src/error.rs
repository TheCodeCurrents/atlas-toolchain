use std::fmt::Display;
use std::io;

use crate::lexer::LexError;
use crate::parser::ParseError;
use atlas_isa::EncodingError;

#[derive(Debug)]
pub enum AssemblerError {
    // File I/O errors
    IoError {
        operation: String,
        source: io::Error,
    },
    // Parsing errors
    ParseError(ParseError),
    // Lexing errors
    LexError(LexError),
    // Encoding errors (unresolved labels, invalid instructions, etc.)
    EncodingError(EncodingError),
}

impl Display for AssemblerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssemblerError::IoError { operation, source } => {
                write!(f, "{}: {}", operation, source)
            }
            AssemblerError::ParseError(err) => {
                write!(f, "{}", err)
            }
            AssemblerError::LexError(err) => {
                write!(f, "{}", err)
            }
            AssemblerError::EncodingError(err) => {
                write!(f, "{}", err)
            }
        }
    }
}

impl std::error::Error for AssemblerError {}

impl From<ParseError> for AssemblerError {
    fn from(err: ParseError) -> Self {
        AssemblerError::ParseError(err)
    }
}

impl From<LexError> for AssemblerError {
    fn from(err: LexError) -> Self {
        AssemblerError::LexError(err)
    }
}

impl From<EncodingError> for AssemblerError {
    fn from(err: EncodingError) -> Self {
        AssemblerError::EncodingError(err)
    }
}
