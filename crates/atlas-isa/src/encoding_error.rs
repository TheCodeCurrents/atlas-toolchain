use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct EncodingError {
    pub line: usize,
    pub message: String,
}

impl Display for EncodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encoding error at line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for EncodingError {}
