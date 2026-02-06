use atlas_isa::{Mnemonic, RegisterIdentifier};

#[derive(Debug, Clone, Copy)]
pub struct Immediate {
    pub value: i32,
    pub signed: bool,  // true if prefixed with +/-
}

#[derive(Debug)]
pub enum Token {
    Mnemonic(Mnemonic),
    Directive(Directive),
    Register(RegisterIdentifier),
    Immediate(Immediate),
    LabelDef(String),
    LabelRef(String),

    Comma,
    AtSign,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,

    NewLine,
    EoF,   
}


#[derive(Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
}

#[derive(Debug)]
pub struct SpannedToken { pub token: Token, pub span: Span }


#[derive(Debug)]
pub enum Directive {
    Import,     // import label from another file
    Export,     // export label for use in another file
}

impl Directive {
    pub fn from_str(s: &str) -> Option<Directive> {
        match s {
            "import" => Some(Directive::Import),
            "export" => Some(Directive::Export),
            _ => None,
        }
    }
}
