use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum ParseError {
    InvalidParameters { line: usize, details: String },
    UnknownSymbol { line: usize, name: String },
    UnexpectedToken { line: usize, expected: &'static str, found: String },
    ImmediateOutOfRange { line: usize, value: i32, min: i32, max: i32 },
    LexError { line: usize, details: String },
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidParameters { line, details } => {
                write!(f, "Invalid parameters at line {}: {}", line, details)
            }
            ParseError::UnknownSymbol { line, name } => {
                write!(f, "Unknown symbol '{}' at line {}", name, line)
            }
            ParseError::UnexpectedToken { line, expected, found } => {
                write!(f, "Unexpected token at line {}: expected {}, found {}", line, expected, found)
            }
            ParseError::ImmediateOutOfRange { line, value, min, max } => {
                write!(f, "Immediate value {} out of range at line {} (expected {}..={})", value, line, min, max)
            }
            ParseError::LexError { line, details } => {
                write!(f, "Lex error at line {}: {}", line, details)
            }
        }
    }
}

impl std::error::Error for ParseError {}
