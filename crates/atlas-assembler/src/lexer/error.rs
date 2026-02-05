use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum LexError {
    InvalidCharacter(char, usize, usize),
    InvalidNumber(String, usize, usize),
    InvalidDirective(String, usize, usize),
    UnexpectedEof,
}

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::InvalidCharacter(c, line, pos) => {
                write!(f, "Invalid character '{}' at line {}, position {}", c, line, pos)
            }
            LexError::InvalidNumber(num, line, pos) => {
                write!(f, "Invalid number '{}' at line {}, position {}", num, line, pos)
            }
            LexError::InvalidDirective(dir, line, pos) => {
                write!(f, "Invalid directive '{}' at line {}, position {}", dir, line, pos)
            }
            LexError::UnexpectedEof => {
                write!(f, "Unexpected end of file")
            }
        }
    }
}

impl std::error::Error for LexError {}
