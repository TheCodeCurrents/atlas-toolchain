use atlas_isa::{AluOp, BranchCond, BranchOperand, ImmOp, Mnemonic, MemOp, Operand, PeekPokeOp, ParsedInstruction, StackOp, XTypeOp, instruction::InstructionFormat, operands::{MOffset, RegisterPairIdentifier, XOperand}};
use crate::lexer::{Directive, LexError, Lexer, SpannedToken, Token};

use crate::{parser::error::ParseError, parser::symbols::{ParsedItem, SymbolTable}};


pub struct Parser<'a> {
    lexer: Lexer<'a>,
    pos: u32,
    symbols: SymbolTable,
    last_line: usize,
    /// Single-token lookahead buffer used when peeking after a label definition.
    pending: Option<SpannedToken>,
    /// The current section (defaults to ".text").
    current_section: String,
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<ParsedItem, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        // get next token, draining the lookahead buffer first
        let spanned = match self.pending.take().map(Ok).or_else(|| self.lexer.next()) {
            Some(Ok(token)) => token,
            Some(Err(err)) => return Some(Err(self.lex_error(err))),
            None => return None,
        };
        self.last_line = spanned.span.line;

        // check for all valid token types
        match spanned.token {
            Token::EoF => None,
            Token::NewLine => {
                // skip blank lines
                self.next()
            }
            Token::Directive(name) => {
                // handle directives
                match self.handle_directive(name) {
                    Ok(Some(item)) => return Some(Ok(item)),
                    Ok(None) => return self.next(),
                    Err(err) => return Some(Err(err)),
                }
            }
            Token::LabelDef(name) => {
                // Peek at the next token to see if a directive follows.
                let next = match self.lexer.next() {
                    Some(Ok(tok)) => Some(tok),
                    Some(Err(err)) => return Some(Err(self.lex_error(err))),
                    None => None,
                };

                match next {
                    Some(SpannedToken { token: Token::Directive(Directive::Imm), .. }) => {
                        // label: .imm <value>
                        let val_tok = match self.next_token() {
                            Ok(t) => t,
                            Err(e) => return Some(Err(e)),
                        };
                        let value = match val_tok.token {
                            Token::Immediate(imm) => imm.value as u16,
                            other => {
                                return Some(Err(ParseError::UnexpectedToken {
                                    line: val_tok.span.line,
                                    expected: "immediate value after .imm",
                                    found: Self::token_description(&other),
                                }));
                            }
                        };
                        self.symbols.insert(name, crate::parser::symbols::Symbol::Constant(value));
                    }
                    Some(tok) => {
                        // No directive – this is a normal positional label.
                        let section = self.current_section.clone();
                        self.symbols.insert(name, crate::parser::symbols::Symbol::Label { offset: self.pos, section });
                        // Put the token back so it gets processed normally.
                        self.pending = Some(tok);
                    }
                    None => {
                        // Label at end-of-file.
                        let section = self.current_section.clone();
                        self.symbols.insert(name, crate::parser::symbols::Symbol::Label { offset: self.pos, section });
                    }
                }

                // call again so it returns the next instruction
                self.next()
            }
            Token::Mnemonic(mnemonic) => {
                let result = self.process_instruction(mnemonic, spanned.span.line);
                match result {
                    Ok(instr) => {
                        self.pos += 2;
                        Some(Ok(ParsedItem::Instruction(instr)))
                    }
                    Err(e) => Some(Err(e)),
                }
            }
            other => {
                // expected Directive, LabelDef, or Mnemonic
                Some(Err(ParseError::UnexpectedToken {
                    line: spanned.span.line,
                    expected: "directive, label definition, or mnemonic",
                    found: Self::token_description(&other),
                }))
            }
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            lexer: Lexer::new(src),
            pos: 0,
            symbols: SymbolTable::new(),
            last_line: 1,
            pending: None,
            current_section: ".text".to_string(),
        }
    }

    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    fn skip_to_line_end(&mut self) -> Result<(), ParseError> {
        loop {
            match self.lexer.next() {
                Some(Ok(token)) => {
                    self.last_line = token.span.line;
                    match token.token {
                        Token::NewLine | Token::EoF => return Ok(()),
                        _ => continue,
                    }
                }
                Some(Err(err)) => return Err(self.lex_error(err)),
                None => return Ok(()),
            }
        }
    }

    fn handle_directive(&mut self, directive: Directive) -> Result<Option<ParsedItem>, ParseError> {
        match directive {
            Directive::Global => {
                let next: SpannedToken = self.next_token()?;
                if let Token::LabelRef(name) = next.token {
                    self.symbols.export(name);
                } else {
                    return Err(ParseError::UnexpectedToken {
                        line: next.span.line,
                        expected: "label after .global",
                        found: Self::token_description(&next.token),
                    });
                }
                self.skip_to_line_end()?;
                Ok(None)
            }
            Directive::Import => {
                let next: SpannedToken = self.next_token()?;
                if let Token::LabelRef(name) = next.token {
                    self.symbols.import(name);
                } else {
                    return Err(ParseError::UnexpectedToken {
                        line: next.span.line,
                        expected: "label after .import",
                        found: Self::token_description(&next.token),
                    });
                }
                self.skip_to_line_end()?;
                Ok(None)
            }
            Directive::Imm => {
                // .imm without a preceding label is invalid
                Err(ParseError::UnexpectedToken {
                    line: self.last_line,
                    expected: "label definition before .imm",
                    found: ".imm directive".to_string(),
                })
            }
            Directive::Text => {
                self.current_section = ".text".to_string();
                self.pos = 0;
                self.skip_to_line_end()?;
                Ok(Some(ParsedItem::SectionChange(".text".to_string())))
            }
            Directive::Data => {
                self.current_section = ".data".to_string();
                self.pos = 0;
                self.skip_to_line_end()?;
                Ok(Some(ParsedItem::SectionChange(".data".to_string())))
            }
            Directive::Bss => {
                self.current_section = ".bss".to_string();
                self.pos = 0;
                self.skip_to_line_end()?;
                Ok(Some(ParsedItem::SectionChange(".bss".to_string())))
            }
            Directive::Section => {
                // .section <name>
                let next: SpannedToken = self.next_token()?;
                let name = match next.token {
                    Token::LabelRef(name) => name,
                    other => {
                        return Err(ParseError::UnexpectedToken {
                            line: next.span.line,
                            expected: "section name after .section",
                            found: Self::token_description(&other),
                        });
                    }
                };
                let section_name = if name.starts_with('.') { name } else { format!(".{}", name) };
                self.current_section = section_name.clone();
                self.pos = 0;
                self.skip_to_line_end()?;
                Ok(Some(ParsedItem::SectionChange(section_name)))
            }
            Directive::Byte => {
                let data = self.collect_byte_list()?;
                self.pos += data.len() as u32;
                Ok(Some(ParsedItem::Data(data)))
            }
            Directive::Word => {
                let data = self.collect_word_list()?;
                self.pos += data.len() as u32;
                Ok(Some(ParsedItem::Data(data)))
            }
            Directive::Ascii => {
                let data = self.collect_ascii_string()?;
                self.pos += data.len() as u32;
                Ok(Some(ParsedItem::Data(data)))
            }
        }
    }

    /// Collect a comma-separated list of byte values: `.byte 0x41, 0x42, 0x43`
    fn collect_byte_list(&mut self) -> Result<Vec<u8>, ParseError> {
        let mut bytes = Vec::new();
        loop {
            let tok = self.next_token()?;
            match tok.token {
                Token::Immediate(imm) => {
                    if imm.value < -128 || imm.value > 255 {
                        return Err(ParseError::ImmediateOutOfRange {
                            line: tok.span.line,
                            value: imm.value,
                            min: -128,
                            max: 255,
                        });
                    }
                    bytes.push(imm.value as u8);
                }
                Token::NewLine | Token::EoF => break,
                other => {
                    return Err(ParseError::UnexpectedToken {
                        line: tok.span.line,
                        expected: "byte value",
                        found: Self::token_description(&other),
                    });
                }
            }
            // check for comma or end of line
            let next = self.next_token()?;
            match next.token {
                Token::Comma => continue,
                Token::NewLine | Token::EoF => break,
                other => {
                    return Err(ParseError::UnexpectedToken {
                        line: next.span.line,
                        expected: "',' or end of line",
                        found: Self::token_description(&other),
                    });
                }
            }
        }
        Ok(bytes)
    }

    /// Collect a comma-separated list of 16-bit word values: `.word 0x1234, 0x5678`
    fn collect_word_list(&mut self) -> Result<Vec<u8>, ParseError> {
        let mut bytes = Vec::new();
        loop {
            let tok = self.next_token()?;
            match tok.token {
                Token::Immediate(imm) => {
                    if imm.value < -32768 || imm.value > 65535 {
                        return Err(ParseError::ImmediateOutOfRange {
                            line: tok.span.line,
                            value: imm.value,
                            min: -32768,
                            max: 65535,
                        });
                    }
                    let word = imm.value as u16;
                    // little-endian
                    bytes.push(word as u8);
                    bytes.push((word >> 8) as u8);
                }
                Token::NewLine | Token::EoF => break,
                other => {
                    return Err(ParseError::UnexpectedToken {
                        line: tok.span.line,
                        expected: "word value",
                        found: Self::token_description(&other),
                    });
                }
            }
            // check for comma or end of line
            let next = self.next_token()?;
            match next.token {
                Token::Comma => continue,
                Token::NewLine | Token::EoF => break,
                other => {
                    return Err(ParseError::UnexpectedToken {
                        line: next.span.line,
                        expected: "',' or end of line",
                        found: Self::token_description(&other),
                    });
                }
            }
        }
        Ok(bytes)
    }

    /// Collect an ASCII string literal. Since the lexer doesn't have string tokens yet,
    /// this reads bytes as a comma-separated list: `.ascii 0x48, 0x65, 0x6C`
    fn collect_ascii_string(&mut self) -> Result<Vec<u8>, ParseError> {
        // For now, treat the same as .byte
        self.collect_byte_list()
    }

    fn next_token(&mut self) -> Result<SpannedToken, ParseError> {
        if let Some(tok) = self.pending.take() {
            self.last_line = tok.span.line;
            return Ok(tok);
        }
        match self.lexer.next() {
            Some(Ok(token)) => {
                self.last_line = token.span.line;
                Ok(token)
            }
            Some(Err(err)) => Err(self.lex_error(err)),
            None => Err(ParseError::UnexpectedToken {
                line: self.last_line,
                expected: "token",
                found: "end of file".to_string(),
            }),
        }
    }

    fn expect_register(&mut self) -> Result<atlas_isa::RegisterIdentifier, ParseError> {
        let token = self.next_token()?;
        match token.token {
            Token::Register(reg) => Ok(reg),
            Token::Immediate(imm) if !imm.signed && (0..=15).contains(&imm.value) => {
                Ok(imm.value as u8)
            }
            other => Err(ParseError::UnexpectedToken {
                line: token.span.line,
                expected: "register",
                found: Self::token_description(&other),
            }),
        }
    }

    /// Expect either an immediate value or a label reference, returning an `Operand`.
    fn expect_immediate_or_label(&mut self) -> Result<Operand, ParseError> {
        let token = self.next_token()?;
        match token.token {
            Token::Immediate(imm) => Ok(Operand::Immediate(imm.value as u16)),
            Token::LabelRef(name) => Ok(Operand::Label(name)),
            other => Err(ParseError::UnexpectedToken {
                line: token.span.line,
                expected: "immediate or label",
                found: Self::token_description(&other),
            }),
        }
    }

    fn expect_comma(&mut self) -> Result<(), ParseError> {
        let token = self.next_token()?;
        match token.token {
            Token::Comma => Ok(()),
            other => Err(ParseError::UnexpectedToken {
                line: token.span.line,
                expected: "','",
                found: Self::token_description(&other),
            }),
        }
    }

    fn expect_newline(&mut self) -> Result<(), ParseError> {
        let token = self.next_token()?;
        match token.token {
            Token::NewLine | Token::EoF => Ok(()),
            other => Err(ParseError::UnexpectedToken {
                line: token.span.line,
                expected: "end of line",
                found: Self::token_description(&other),
            }),
        }
    }

    fn process_instruction(&mut self, instruction: Mnemonic, line: usize) -> Result<ParsedInstruction, ParseError> {
        match instruction.get_type() {
            InstructionFormat::A => {
                // A-type: rd, rs
                let rd = self.expect_register()?;
                self.expect_comma()?;
                let rs = self.expect_register()?;
                self.expect_newline()?;

                let op = AluOp::from_instruction(instruction)
                    .ok_or(ParseError::InvalidParameters {
                        line,
                        details: format!("Instruction '{}' is not a valid ALU op", instruction.mnemonic()),
                    })?;

                // r0 is hardwired to zero; CMP and TST only set flags so they're fine
                if rd == 0 && !matches!(op, AluOp::CMP | AluOp::TST) {
                    return Err(ParseError::WriteToR0 {
                        line,
                        instruction: instruction.mnemonic().to_string(),
                    });
                }
                
                Ok(ParsedInstruction::A {
                    op,
                    dest: rd,
                    source: rs,
                    line,
                    source_file: None,
                })
            },
            InstructionFormat::I => {
                // I-type: rd, immediate or label
                let rd = self.expect_register()?;
                self.expect_comma()?;
                let imm = self.expect_immediate_or_label()?;
                self.expect_newline()?;

                let op = ImmOp::from_instruction(instruction)
                    .ok_or(ParseError::InvalidParameters {
                        line,
                        details: format!("Instruction '{}' is not a valid immediate op", instruction.mnemonic()),
                    })?;

                if rd == 0 {
                    return Err(ParseError::WriteToR0 {
                        line,
                        instruction: instruction.mnemonic().to_string(),
                    });
                }

                Ok(ParsedInstruction::I {
                    op,
                    dest: rd,
                    immediate: imm,
                    line,
                    source_file: None,
                })
            },
            InstructionFormat::M => {
                // M-type: rd, [base + offset]
                let rd = self.expect_register()?;
                self.expect_comma()?;

                // Expect opening bracket
                let bracket_tok = self.next_token()?;
                match bracket_tok.token {
                    Token::OpenBracket => {},
                    other => {
                        return Err(ParseError::UnexpectedToken {
                            line: bracket_tok.span.line,
                            expected: "'['",
                            found: Self::token_description(&other),
                        });
                    }
                }

                // Get base register
                let base = self.expect_register()?;

                // Expect ',' or '+' or '-'
                let op_token = self.next_token()?;
                let op = match op_token.token {
                    Token::Comma => "+",
                    Token::LabelRef(ref name) if name == "+" => "+",
                    Token::LabelRef(ref name) if name == "-" => "-",
                    other => {
                        return Err(ParseError::UnexpectedToken {
                            line: op_token.span.line,
                            expected: "',' or '+' or '-'",
                            found: Self::token_description(&other),
                        });
                    }
                };

                // Get offset: register or immediate
                let offset_token = self.next_token()?;
                let offset = match offset_token.token {
                    Token::Register(reg) => {
                        if op == "-" {
                            return Err(ParseError::InvalidParameters {
                                line: offset_token.span.line,
                                details: "negative register offsets are not supported".to_string(),
                            });
                        }
                        MOffset::SR(reg)
                    }
                    Token::Immediate(imm) => {
                        let mut imm_val = imm.value;
                        if op == "-" && imm_val > 0 {
                            imm_val = -imm_val;
                        }
                        if imm_val < -5 || imm_val > 7 {
                            return Err(ParseError::ImmediateOutOfRange {
                                line: offset_token.span.line,
                                value: imm_val,
                                min: -5,
                                max: 7,
                            });
                        }
                        // Map special offsets to SPR codes via negative sentinels
                        // (values -6, -7, -8 are outside the valid range and
                        //  correspond to the reserved SPR codes in the 4-bit field)
                        MOffset::Offset8(imm_val as u8)
                    }
                    other => {
                        return Err(ParseError::UnexpectedToken {
                            line: offset_token.span.line,
                            expected: "offset immediate or register",
                            found: Self::token_description(&other),
                        });
                    }
                };

                // Expect closing bracket
                let close_tok = self.next_token()?;
                match close_tok.token {
                    Token::CloseBracket => {},
                    other => {
                        return Err(ParseError::UnexpectedToken {
                            line: close_tok.span.line,
                            expected: "']'",
                            found: Self::token_description(&other),
                        });
                    }
                }

                self.expect_newline()?;

                let op = MemOp::from_instruction(instruction)
                    .ok_or(ParseError::InvalidParameters {
                        line,
                        details: format!("Instruction '{}' is not a valid memory op", instruction.mnemonic()),
                    })?;

                if rd == 0 && op == MemOp::LD {
                    return Err(ParseError::WriteToR0 {
                        line,
                        instruction: instruction.mnemonic().to_string(),
                    });
                }

                Ok(ParsedInstruction::M {
                    op,
                    dest: rd,
                    base,
                    offset,
                    line,
                    source_file: None,
                })
            },
            InstructionFormat::B => {
                // B-type: condition and either immediate address, label, or register pair
                let cond = BranchCond::from_instruction(instruction)
                    .ok_or(ParseError::InvalidParameters {
                        line,
                        details: format!("Instruction '{}' is not a valid branch op", instruction.mnemonic()),
                    })?;

                let next_tok = self.next_token()?;
                
                match next_tok.token {
                    Token::Immediate(imm) => {
                        // Check if signed (+/-) to determine if relative or absolute
                        if imm.signed {
                            // Relative offset branch
                            if imm.value < -128 || imm.value > 127 {
                                return Err(ParseError::ImmediateOutOfRange {
                                    line: next_tok.span.line,
                                    value: imm.value,
                                    min: -128,
                                    max: 127,
                                });
                            }
                            self.expect_newline()?;
                            Ok(ParsedInstruction::BI {
                                absolute: false,
                                cond,
                                operand: BranchOperand::Immediate(imm.value as u16),
                                line,
                                source_file: None,
                            })
                        } else {
                            // Absolute immediate branch
                            self.expect_newline()?;
                            Ok(ParsedInstruction::BI {
                                absolute: true,
                                cond,
                                operand: BranchOperand::Immediate(imm.value as u16),
                                line,
                                source_file: None,
                            })
                        }
                    },
                    Token::LabelRef(label_name) => {
                        // Branch to label - store label reference (do NOT resolve yet)
                        self.expect_newline()?;
                        
                        // Branches to labels are always absolute (will be resolved by linker)
                        Ok(ParsedInstruction::BI {
                            absolute: true,
                            cond,
                            operand: BranchOperand::Label(label_name),
                            line,
                            source_file: None,
                        })
                    },
                    Token::Register(reg1) => {
                        // Register pair branch (reg1 and next register)
                        self.expect_comma()?;
                        let reg2 = self.expect_register()?;
                        self.expect_newline()?;
                        
                        Ok(ParsedInstruction::BR {
                            absolute: true,
                            cond,
                            source: RegisterPairIdentifier { high: reg1, low: reg2 },
                            line,
                            source_file: None,
                        })
                    },
                    other => {
                        Err(ParseError::UnexpectedToken {
                            line: next_tok.span.line,
                            expected: "immediate, label, or register",
                            found: Self::token_description(&other),
                        })
                    }
                }
            },
            InstructionFormat::S => {
                // S-type: push/pop take a register; subsp/addsp take register OR immediate
                let next_tok = self.next_token()?;

                match instruction {
                    Mnemonic::PUSH => {
                        let reg = match next_tok.token {
                            Token::Register(r) => r,
                            other => return Err(ParseError::UnexpectedToken {
                                line: self.last_line,
                                expected: "register",
                                found: Self::token_description(&other),
                            }),
                        };
                        self.expect_newline()?;
                        Ok(ParsedInstruction::S {
                            op: StackOp::PUSH,
                            operand: reg,
                            line,
                            source_file: None,
                        })
                    }
                    Mnemonic::POP => {
                        let reg = match next_tok.token {
                            Token::Register(r) => r,
                            other => return Err(ParseError::UnexpectedToken {
                                line: self.last_line,
                                expected: "register",
                                found: Self::token_description(&other),
                            }),
                        };
                        if reg == 0 {
                            return Err(ParseError::WriteToR0 {
                                line,
                                instruction: instruction.mnemonic().to_string(),
                            });
                        }
                        self.expect_newline()?;
                        Ok(ParsedInstruction::S {
                            op: StackOp::POP,
                            operand: reg,
                            line,
                            source_file: None,
                        })
                    }
                    Mnemonic::SUBSP => {
                        match next_tok.token {
                            Token::Register(reg) => {
                                self.expect_newline()?;
                                Ok(ParsedInstruction::S {
                                    op: StackOp::SUBSP_REG,
                                    operand: reg,
                                    line,
                                    source_file: None,
                                })
                            }
                            Token::Immediate(imm) => {
                                if imm.value < 0 || imm.value > 255 {
                                    return Err(ParseError::ImmediateOutOfRange {
                                        line: self.last_line,
                                        value: imm.value,
                                        min: 0,
                                        max: 255,
                                    });
                                }
                                self.expect_newline()?;
                                Ok(ParsedInstruction::S {
                                    op: StackOp::SUBSP_IMM,
                                    operand: imm.value as u8,
                                    line,
                                    source_file: None,
                                })
                            }
                            other => Err(ParseError::UnexpectedToken {
                                line: self.last_line,
                                expected: "register or immediate",
                                found: Self::token_description(&other),
                            }),
                        }
                    }
                    Mnemonic::ADDSP => {
                        match next_tok.token {
                            Token::Register(reg) => {
                                self.expect_newline()?;
                                Ok(ParsedInstruction::S {
                                    op: StackOp::ADDSP_REG,
                                    operand: reg,
                                    line,
                                    source_file: None,
                                })
                            }
                            Token::Immediate(imm) => {
                                if imm.value < 0 || imm.value > 255 {
                                    return Err(ParseError::ImmediateOutOfRange {
                                        line: self.last_line,
                                        value: imm.value,
                                        min: 0,
                                        max: 255,
                                    });
                                }
                                self.expect_newline()?;
                                Ok(ParsedInstruction::S {
                                    op: StackOp::ADDSP_IMM,
                                    operand: imm.value as u8,
                                    line,
                                    source_file: None,
                                })
                            }
                            other => Err(ParseError::UnexpectedToken {
                                line: self.last_line,
                                expected: "register or immediate",
                                found: Self::token_description(&other),
                            }),
                        }
                    }
                    _ => Err(ParseError::InvalidParameters {
                        line,
                        details: format!("Instruction '{}' is not a valid stack op", instruction.mnemonic()),
                    }),
                }
            },
            InstructionFormat::P => {
                // P-type: register, offset (immediate or label)
                let reg = self.expect_register()?;
                self.expect_comma()?;
                let offset = self.expect_immediate_or_label()?;
                self.expect_newline()?;

                let op = PeekPokeOp::from_instruction(instruction)
                    .ok_or(ParseError::InvalidParameters {
                        line,
                        details: format!("Instruction '{}' is not a valid peek/poke op", instruction.mnemonic()),
                    })?;

                if reg == 0 && op == PeekPokeOp::PEEK {
                    return Err(ParseError::WriteToR0 {
                        line,
                        instruction: instruction.mnemonic().to_string(),
                    });
                }

                Ok(ParsedInstruction::P {
                    op,
                    register: reg,
                    offset,
                    line,
                    source_file: None,
                })
            },
            InstructionFormat::X => {
                // X-type: may have no operands or various operand formats
                let next_tok = self.next_token()?;
                
                let operand = match next_tok.token {
                    Token::NewLine | Token::EoF => {
                        // No operands
                        XOperand::None
                    },
                    Token::Immediate(imm) => {
                        self.expect_newline()?;
                        XOperand::Immediate(imm.value as u8)
                    },
                    Token::Register(reg) => {
                        // Check if followed by comma for register pair
                        let check_next = self.next_token()?;
                        match check_next.token {
                            Token::Comma => {
                                let reg2 = self.expect_register()?;
                                self.expect_newline()?;
                                XOperand::Registers(reg, reg2)
                            },
                            Token::NewLine | Token::EoF => {
                                XOperand::Register(reg)
                            },
                            other => {
                                return Err(ParseError::UnexpectedToken {
                                    line: check_next.span.line,
                                    expected: "',' or end of line",
                                    found: Self::token_description(&other),
                                });
                            }
                        }
                    },
                    other => {
                        return Err(ParseError::UnexpectedToken {
                            line: next_tok.span.line,
                            expected: "immediate, register, or end of line",
                            found: Self::token_description(&other),
                        });
                    }
                };

                let op = XTypeOp::from_instruction(instruction)
                    .ok_or(ParseError::InvalidParameters {
                        line,
                        details: format!("Instruction '{}' is not a valid extended op", instruction.mnemonic()),
                    })?;

                Ok(ParsedInstruction::X {
                    op,
                    operand,
                    line,
                    source_file: None,
                })
            },
            InstructionFormat::Virtual => {
                match instruction {
                    Mnemonic::NOP => {
                        // NOP = add r0, r0  (encodes as 0x0000)
                        self.expect_newline()?;
                        Ok(ParsedInstruction::A {
                            op: AluOp::ADD,
                            dest: 0,
                            source: 0,
                            line,
                            source_file: None,
                        })
                    }
                    Mnemonic::INC => {
                        // INC rd = addi rd, 1
                        let rd = self.expect_register()?;
                        self.expect_newline()?;
                        if rd == 0 {
                            return Err(ParseError::WriteToR0 {
                                line,
                                instruction: instruction.mnemonic().to_string(),
                            });
                        }
                        Ok(ParsedInstruction::I {
                            op: ImmOp::ADDI,
                            dest: rd,
                            immediate: Operand::Immediate(1),
                            line,
                            source_file: None,
                        })
                    }
                    Mnemonic::DEC => {
                        // DEC rd = subi rd, 1
                        let rd = self.expect_register()?;
                        self.expect_newline()?;
                        if rd == 0 {
                            return Err(ParseError::WriteToR0 {
                                line,
                                instruction: instruction.mnemonic().to_string(),
                            });
                        }
                        Ok(ParsedInstruction::I {
                            op: ImmOp::SUBI,
                            dest: rd,
                            immediate: Operand::Immediate(1),
                            line,
                            source_file: None,
                        })
                    }
                    _ => {
                        Err(ParseError::InvalidParameters {
                            line,
                            details: format!("Unknown virtual instruction '{}'", instruction.mnemonic()),
                        })
                    }
                }
            }
        }
    }

    fn token_description(token: &Token) -> String {
        match token {
            Token::Mnemonic(inst) => format!("mnemonic '{}'", inst.mnemonic()),
            Token::Directive(dir) => format!("directive '{:?}'", dir),
            Token::Register(reg) => format!("register r{}", reg),
            Token::Immediate(imm) => {
                if imm.signed {
                    format!("relative offset {}", imm.value)
                } else {
                    format!("immediate {}", imm.value)
                }
            },
            Token::LabelDef(name) => format!("label definition '{}'", name),
            Token::LabelRef(name) => format!("label reference '{}'", name),
            Token::Comma => ",".to_string(),
            Token::AtSign => "'@'".to_string(),
            Token::OpenParen => "'('".to_string(),
            Token::CloseParen => "')'".to_string(),
            Token::OpenBracket => "'['".to_string(),
            Token::CloseBracket => "']'".to_string(),
            Token::NewLine => "newline".to_string(),
            Token::EoF => "end of file".to_string(),
        }
    }

    fn lex_error(&self, err: LexError) -> ParseError {
        let line = match err {
            LexError::InvalidCharacter(_, line, _)
            | LexError::InvalidNumber(_, line, _)
            | LexError::InvalidDirective(_, line, _) => line,
            LexError::UnexpectedEof => self.last_line,
        };

        ParseError::LexError {
            line,
            details: err.to_string(),
        }
    }
}
