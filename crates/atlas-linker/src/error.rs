use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkerErrorKind {
    Io,
    ObjectFile,
    UnresolvedLabel,
    DuplicateSymbol,
    Encoding,
}

impl Display for LinkerErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            LinkerErrorKind::Io => "IO",
            LinkerErrorKind::ObjectFile => "ObjectFile",
            LinkerErrorKind::UnresolvedLabel => "UnresolvedLabel",
            LinkerErrorKind::DuplicateSymbol => "DuplicateSymbol",
            LinkerErrorKind::Encoding => "Encoding",
        };
        write!(f, "{}", label)
    }
}

#[derive(Debug, Clone)]
pub struct LinkerError {
    pub kind: LinkerErrorKind,
    pub message: String,
    pub line: usize,
    pub source_file: Option<String>,
}

impl LinkerError {
    pub fn new(kind: LinkerErrorKind, message: String, line: usize, source_file: Option<String>) -> Self {
        Self {
            kind,
            message,
            line,
            source_file,
        }
    }
}

impl Display for LinkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source = self.source_file.as_deref().unwrap_or("<unknown>");
        write!(
            f,
            "Linker error ({}) at {}:{}: {}",
            self.kind, source, self.line, self.message
        )
    }
}

impl std::error::Error for LinkerError {}
