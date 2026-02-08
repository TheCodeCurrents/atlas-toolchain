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
    Global,     // declare a global symbol: .global foo
    Import,     // declare an imported (external) symbol: .import foo

    Imm,        // assign an immediate value to the preceding label: label: .imm 42

    Text,
    Data,
    Bss,
    Section,

    Byte,
    Word,
    Ascii,
}

impl Directive {
    pub fn from_str(s: &str) -> Option<Directive> {
        match s {
            "global" | "export" => Some(Directive::Global),
            "import" => Some(Directive::Import),
            "imm" => Some(Directive::Imm),
            "text" => Some(Directive::Text),
            "data" => Some(Directive::Data),
            "bss" => Some(Directive::Bss),
            "section" => Some(Directive::Section),
            "byte" => Some(Directive::Byte),
            "word" => Some(Directive::Word),
            "ascii" => Some(Directive::Ascii),
            _ => None,
        }
    }
}
