use atlas_isa::{AluOp, BranchCond, BranchOperand, ImmOp, Instruction, MemOp, PortOp, ResolvedInstruction, StackOp, XTypeOp, instruction::InstructionFormat, operands::{MOffset, RegisterPairIdentifier, XOperand}};
use crate::lexer::{Directive, LexError, Lexer, SpannedToken, Token};

use crate::{parser::error::ParseError, parser::symbols::SymbolTable};


pub struct Parser<'a> {
    lexer: Lexer<'a>,
    pos: u32,
    symbols: SymbolTable,
    last_line: usize,
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<ResolvedInstruction, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        // get next token from lexer
        let spanned = match self.lexer.next() {
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
                // handle directives (e.g. .import label)
                if let Err(err) = self.handle_directive(name) {
                    return Some(Err(err));
                }

                // call again so it returns the next instruction
                self.next()
            }
            Token::LabelDef(name) => {
                // handle label definitions here
                self.symbols.insert(name, crate::parser::symbols::Symbol::Label(self.pos));

                // call again so it returns the next instruction
                self.next()
            }
            Token::Mnemonic(mnemonic) => {
                let result = self.process_instruction(mnemonic, spanned.span.line);
                if result.is_ok() {
                    self.pos += 2;
                }
                Some(result)
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

    fn handle_directive(&mut self, directive: Directive) -> Result<(), ParseError> {
        match directive {
            Directive::Import => {
                let next: SpannedToken = self.next_token()?;
                if let Token::LabelRef(name) = next.token {
                    self.symbols.insert(name, crate::parser::symbols::Symbol::External);
                } else {
                    return Err(ParseError::UnexpectedToken {
                        line: next.span.line,
                        expected: "label after .import",
                        found: Self::token_description(&next.token),
                    });
                }
                self.skip_to_line_end()
            }
            Directive::Export => {
                let next: SpannedToken = self.next_token()?;
                if let Token::LabelRef(name) = next.token {
                    self.symbols.export(name);
                } else {
                    return Err(ParseError::UnexpectedToken {
                        line: next.span.line,
                        expected: "label after .export",
                        found: Self::token_description(&next.token),
                    });
                }
                self.skip_to_line_end()
            }
        }
    }

    fn next_token(&mut self) -> Result<SpannedToken, ParseError> {
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

    fn expect_immediate(&mut self) -> Result<i32, ParseError> {
        let token = self.next_token()?;
        match token.token {
            Token::Immediate(imm) => Ok(imm.value),
            other => Err(ParseError::UnexpectedToken {
                line: token.span.line,
                expected: "immediate",
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

    fn process_instruction(&mut self, instruction: Instruction, line: usize) -> Result<ResolvedInstruction, ParseError> {
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
                
                Ok(ResolvedInstruction::A {
                    op,
                    dest: rd,
                    source: rs,
                    line,
                    source_file: None,
                })
            },
            InstructionFormat::I => {
                // I-type: rd, immediate
                let rd = self.expect_register()?;
                self.expect_comma()?;
                let imm = self.expect_immediate()?;
                self.expect_newline()?;

                let op = ImmOp::from_instruction(instruction)
                    .ok_or(ParseError::InvalidParameters {
                        line,
                        details: format!("Instruction '{}' is not a valid immediate op", instruction.mnemonic()),
                    })?;

                Ok(ResolvedInstruction::I {
                    op,
                    dest: rd,
                    immediate: imm as u8,
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
                        if imm_val < -8 || imm_val > 5 {
                            return Err(ParseError::ImmediateOutOfRange {
                                line: offset_token.span.line,
                                value: imm_val,
                                min: -8,
                                max: 5,
                            });
                        }
                        // Map special offsets to registers
                        match imm_val {
                            -6 => MOffset::SR(10), // TR
                            -7 => MOffset::SR(12), // SP
                            -8 => MOffset::SR(14), // PC
                            val => MOffset::Offset8(val as u8),
                        }
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

                Ok(ResolvedInstruction::M {
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
                            Ok(ResolvedInstruction::BI {
                                absolute: false,
                                cond,
                                operand: BranchOperand::Immediate(imm.value as u8),
                                line,
                                source_file: None,
                            })
                        } else {
                            // Absolute immediate branch
                            self.expect_newline()?;
                            Ok(ResolvedInstruction::BI {
                                absolute: true,
                                cond,
                                operand: BranchOperand::Immediate(imm.value as u8),
                                line,
                                source_file: None,
                            })
                        }
                    },
                    Token::LabelRef(label_name) => {
                        // Branch to label - store label reference (do NOT resolve yet)
                        self.expect_newline()?;
                        
                        // Branches to labels are always absolute (will be resolved by linker)
                        Ok(ResolvedInstruction::BI {
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
                        
                        Ok(ResolvedInstruction::BR {
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
                // S-type: register only
                let reg = self.expect_register()?;
                self.expect_newline()?;

                let op = StackOp::from_instruction(instruction)
                    .ok_or(ParseError::InvalidParameters {
                        line,
                        details: format!("Instruction '{}' is not a valid stack op", instruction.mnemonic()),
                    })?;

                Ok(ResolvedInstruction::S {
                    op,
                    register: reg,
                    line,
                    source_file: None,
                })
            },
            InstructionFormat::P => {
                // P-type: register, offset
                let reg = self.expect_register()?;
                self.expect_comma()?;
                let offset = self.expect_immediate()?;
                self.expect_newline()?;

                let op = PortOp::from_instruction(instruction)
                    .ok_or(ParseError::InvalidParameters {
                        line,
                        details: format!("Instruction '{}' is not a valid port op", instruction.mnemonic()),
                    })?;

                Ok(ResolvedInstruction::P {
                    op,
                    register: reg,
                    offset: offset as u8,
                    line,
                    source_file: None,
                })
            },
            InstructionFormat::X => {
                // X-type: may have no operands or various operand formats
                let next_tok = self.next_token()?;
                
                let operand = match next_tok.token {
                    Token::NewLine => {
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
                            Token::NewLine => {
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

                Ok(ResolvedInstruction::X {
                    op,
                    operand,
                    line,
                    source_file: None,
                })
            },
            InstructionFormat::Virtual => {
                // Virtual instructions like NOP
                self.expect_newline()?;
                
                // NOP is typically treated as MOV r0, r0
                Ok(ResolvedInstruction::A {
                    op: AluOp::MOV,
                    dest: 0,
                    source: 0,
                    line,
                    source_file: None,
                })
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
