use atlas_isa::{Instruction};

use crate::lexer::{LexError, Token, token::{Directive, Span, SpannedToken}};

#[derive(Debug)]
pub struct Lexer<'a> {
    src: &'a str,
    pos: usize,
    line: usize,
    eof_reached: bool,
    last_was_newline: bool,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<SpannedToken, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        // check if we already emitted EOF
        if self.eof_reached {
            return None;
        }

        // skip whitespace and comments (but not newlines)
        self.skip();

        // look for end of file
        if self.pos >= self.src.len() {
            self.eof_reached = true;
            return Some(Ok(SpannedToken {
                token: crate::lexer::Token::EoF,
                span: Span {
                    start: self.pos,
                    end: self.pos,
                    line: self.line
                }
            }))
        }

        // check for newline token (but skip if last token was also a newline)
        if let Some('\n') = self.peek() {
            if self.last_was_newline {
                // skip this newline and any consecutive ones
                while let Some('\n') = self.peek() {
                    self.advance(1);
                    self.line += 1;
                }
                // continue to next token
                self.skip();
                return self.next();
            } else {
                let start = self.pos;
                let line = self.line;
                self.advance(1);
                self.line += 1;
                self.last_was_newline = true;
                
                return Some(Ok(SpannedToken {
                    token: Token::NewLine,
                    span: Span { start, end: self.pos, line },
                }));
            }
        }

        // start tokenizing
        let start = self.pos;

        // check for single-char tokens first (before get_word to avoid empty strings)
        if let Some(c) = self.peek() {
            if let Some(token) = Self::process_single_char_token(c) {
                self.advance(c.len_utf8());
                self.last_was_newline = false;
                return Some(Ok(SpannedToken {
                    token,
                    span: Span {
                        start,
                        end: self.pos,
                        line: self.line,
                    },
                }));
            }
        }

        // get next word
        let word = self.get_word();

        // check for directives
        if let Some(rest) = word.strip_prefix('.') {
            if let Some(directive) = Directive::from_str(rest) {
                self.last_was_newline = false;
                return Some(Ok(SpannedToken {
                    token: Token::Directive(directive),
                    span: Span {
                        start,
                        end: self.pos,
                        line: self.line,
                    },
                }));
            } else {
                // invalid directive
                return Some(Err(LexError::InvalidDirective(word.to_string(), start, self.line)));
            }
        }

        // check for registers
        if let Some(reg_num) = word.strip_prefix('r') {
            if let Ok(n) = reg_num.parse::<u8>() {
                if n <= 15 {
                    self.last_was_newline = false;
                    return Some(Ok(SpannedToken {
                        token: Token::Register(n),
                        span: Span { start, end: self.pos, line: self.line },
                    }));
                }
            }
        }

        if let Some(reg) = match word {
            "tr" => Some(10),
            "sp" => Some(12),
            "pc" => Some(14),
            _ => None,
        } {
            self.last_was_newline = false;
            return Some(Ok(SpannedToken {
                token: Token::Register(reg),
                span: Span { start, end: self.pos, line: self.line },
            }));
        }

        // check for numbers
        if let Some(result) = Self::check_for_number(word) {
            self.last_was_newline = false;
            // Check if the number has an explicit +/- prefix
            let is_signed = word.starts_with('+') || word.starts_with('-');
            return Some(result.map(|value| SpannedToken {
                token: Token::Immediate(crate::lexer::token::Immediate {
                    value,
                    signed: is_signed,
                }),
                span: Span {
                    start,
                    end: self.pos,
                    line: self.line
                }
            }).map_err(|(error_msg, _)| {
                LexError::InvalidNumber(error_msg, start, self.line)
            }));
        }

        // check for label definitions
        if let Some(label) = word.strip_suffix(':') {
            if label.is_empty() {
                // invalid label (no label name)
                return Some(Err(LexError::InvalidCharacter(':', start, self.line)));
            }

            self.last_was_newline = false;
            return Some(Ok(SpannedToken {
                token: Token::LabelDef(label.to_string()),
                span: Span {
                    start,
                    end: self.pos,
                    line: self.line,
                },
            }));
        }

        // check for mnemonics and label references
        if let Some(instruction) = Instruction::from_str(word) {
            self.last_was_newline = false;
            return Some(Ok(SpannedToken {
                token: Token::Mnemonic(instruction),
                span: Span {
                    start,
                    end: self.pos,
                    line: self.line
                }
            }));
        } else {
            self.last_was_newline = false;
            return Some(Ok(SpannedToken {
                token: Token::LabelRef(String::from(word)),
                span: Span {
                    start,
                    end: self.pos,
                    line: self.line
                }
            }));
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src, pos: 0, line: 1, eof_reached: false, last_was_newline: false
        }
    }

    pub fn tokenize(src: &'a str) -> Result<Vec<SpannedToken>, LexError> {
        let mut lexer = Lexer::new(src);
        let mut tokens = Vec::new();

        while let Some(result) = lexer.next() {
            match result {
                Ok(token) => tokens.push(token),
                Err(e) => return Err(e),
            }
        }

        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn advance(&mut self, n: usize) {
        self.pos += n;
    }

    /// skip whitespace and comments (but not newlines)
    fn skip(&mut self) {
        loop {
            while let Some(c) = self.peek() {
                if c == ' ' || c == '\t' {
                    self.advance(c.len_utf8());
                } else if c == ';' {
                    // skip comment until newline
                    while let Some(c2) = self.peek() {
                        if c2 == '\n' {
                            break;
                        }
                        self.advance(c2.len_utf8());
                    }
                    break; // exit inner loop to check for newline
                } else {
                    break; // reached a real token (including newlines)
                }
            }
            
            // if we're at a newline, don't skip it yet
            if let Some('\n') = self.peek() {
                break;
            } else {
                break;
            }
        }
    }


    /// advance until next whitespace or punctuation and return the word
    fn get_word(&mut self) -> &'a str {
        let start = self.pos;

        // check for EOF
        if self.peek().is_none() {
            return &self.src[start..self.pos];
        }

        while let Some(c) = self.peek() {
            if Self::is_whitespace(c) || Self::is_punctuation(c) {
                break;
            }
            self.advance(c.len_utf8());
        }

        &self.src[start..self.pos]
    }

    fn is_whitespace(c: char) -> bool {
        c == ' ' || c == '\t' || c == '\n'
    }

    fn is_punctuation(c: char) -> bool {
        matches!(c, ',' | '@' | '\n' | '(' | ')' | '[' | ']')
    }

    fn process_single_char_token(char: char) -> Option<Token> {
        match char {
            ',' => Some(Token::Comma),
            '@' => Some(Token::AtSign),
            '(' => Some(Token::OpenParen),
            ')' => Some(Token::CloseParen),
            '[' => Some(Token::OpenBracket),
            ']' => Some(Token::CloseBracket),
            _ => None,
        }
    }

    /// Checks if given word is a number, returns None if not a number,
    /// Some(Ok(value)) if valid, or Some(Err((error_msg, _))) if invalid number format
    fn check_for_number(word: &str) -> Option<Result<i32, (String, ())>> {
        // check for prefixes
        if let Some(number) = word.strip_prefix("0x") {
            // parse hex number
            return Some(i32::from_str_radix(number, 16)
                .map_err(|_| (word.to_string(), ())));
        } else if let Some(number) = word.strip_prefix("0b") {
            // parse bin number
            return Some(i32::from_str_radix(number, 2)
                .map_err(|_| (word.to_string(), ())));
        } else if let Some(number) = word.strip_prefix("0o") {
            // parse octal number
            return Some(i32::from_str_radix(number, 8)
                .map_err(|_| (word.to_string(), ())));
        }

        // try to parse as decimal
        match word.parse::<i32>() {
            Ok(value) => Some(Ok(value)),
            Err(_) => None  // Not a number, let other checks handle it
        }
    }
}
