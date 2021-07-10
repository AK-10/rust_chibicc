pub mod loc;

use crate::token::{ Token, TokenType };
use crate::tokenizer::loc::Loc;
use crate::token::token_type::*;

use std::rc::Rc;

// TODO: LexerErrorの定義
const KEYWORDS: [&str; 17] = [
    "return",
    "if",
    "while",
    "else",
    "for",
    "int",
    "short",
    "long",
    "char",
    "void",
    "_Bool",
    "sizeof",
    "struct",
    "typedef",
    "enum",
    "static",
    "break"
];

// multi-letter punctuator
const MULTI_LETTER_PUNCTUACTORS: [&str; 13] = [
    "==",
    "!=",
    "<=",
    ">=",
    "->",
    "++",
    "--",
    "+=",
    "-=",
    "*=",
    "/=",
    "&&",
    "||"
];

pub struct Tokenizer {
    user_input: String,
    current_col_index: usize,
    current_row_index: usize,
    pos: usize
}

impl<'a> Tokenizer {
    pub fn new(user_input: String) -> Self {
        Self {
            user_input, current_col_index: 0, current_row_index: 0,
            pos: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::<Token>::new();

        while self.pos <= self.user_input.len() - 1 {
            // line comment
            if self.multi_get(2)
                .map(|line_comment| line_comment == "//")
                .unwrap_or(false) {
                    while self.current().unwrap() != '\n' {
                        self.increment_pos(1);
                    }
                    self.current_col_index = 0;
                    self.current_row_index += 1;

                    continue
            }
            // block comment
            if self.multi_get(2)
                .map(|block_comment| block_comment == "/*")
                .unwrap_or(false) {
                    self.block_comment()?;

                continue
            }

            if let Some(punct) = self.starts_with_multi_letter_punct() {
                self.increment_pos(punct.len());

                let op = Rc::new(punct);
                let token_type = TokenType::Reserved(Reserved {
                    op: Rc::clone(&op),
                    tk_str: op
                });

                tokens.push(self.new_token(token_type));

                continue
            }

            let c = self.current().expect("pos is out of user_input range");
            match c {
                '=' | '!' | '<' | '>' | '+' | '-' | '*' | '&' | '/' | '~' | '|' | '^' => {
                    self.increment_pos(1);

                    let op = c.to_string();
                     let rc_op = Rc::new(op);
                    let reserved = Reserved {
                        op: Rc::clone(&rc_op),
                        tk_str: rc_op
                    };
                    let token_type = TokenType::Reserved(reserved);
                    tokens.push(self.new_token(token_type));
                },
                // symbol
                '(' | ')' | ';' | '{' | '}' | '.' | ',' | '[' | ']' => {
                    self.increment_pos(1);

                    let sym = c.to_string();
                    let rc_sym = Rc::new(sym);
                    let symbol = Symbol {
                        sym: Rc::clone(&rc_sym),
                        tk_str: rc_sym
                    };
                    let token_type = TokenType::Symbol(symbol);
                    tokens.push(self.new_token(token_type));
                },
                // string literal
                '"' => {
                    let contents = self.read_string_literal();
                    match contents {
                        Ok(c) => {
                            let bytes = c.clone();
                            let contents_string = String::from_utf8(c);
                            match contents_string {
                                Ok(cs) => {
                                    let str_type = Str {
                                        bytes,
                                        tk_str: Rc::new(cs)
                                    };

                                    tokens.push(self.new_token(TokenType::Str(str_type)));
                                },
                                Err(e) => return Err(e.to_string())
                            }
                        },
                        Err(e) => {
                            let msg = format!("error occured in tokenizing string: {}", e);
                            return Err(msg)
                        }
                    }
                },
                // character literal
                '\'' => {
                    let c = self.read_char_literal()?;
                    let token_type = TokenType::Num(
                        Num {
                            val: c as isize,
                            tk_str: Rc::new(format!("'{}'", c as char))
                        }
                    );

                    tokens.push(self.new_token(token_type));
                }
                // num
                '0' ..= '9' => {
                    let token_type = self.read_int_literal()?;

                    tokens.push(self.new_token(token_type));
                },
                // 改行文字はis_whitespaceに含まれるため，それより前に書く
                '\n' => {
                    self.increment_pos(1);
                    self.current_col_index = 0;
                    self.current_row_index += 1;

                }
                ws if ws.is_whitespace() => {
                    self.increment_pos(1);
                    continue
                },
                // ident or reserved
                'a' ..= 'z' | 'A' ..= 'Z' | '_' => {
                    let letter = self.get_letter();
                    let rc_letter = Rc::new(letter);
                    if KEYWORDS.contains(&rc_letter.as_ref().as_str()) {
                        let reserved_type = Reserved {
                            op: Rc::clone(&rc_letter),
                            tk_str: rc_letter
                        };

                        tokens.push(self.new_token(TokenType::Reserved(reserved_type)));
                    } else {
                        let ident_type = Ident {
                            name: Rc::clone(&rc_letter),
                            tk_str: rc_letter
                        };

                        tokens.push(self.new_token(TokenType::Ident(ident_type)));
                    }
                },
                unsupported => {
                    // unsuupported character
                    let msg = format!("unsupported character: {}", unsupported);
                    return Err(msg)
                }
            }
        }

        tokens.push(self.new_token(TokenType::Eof));

        Ok(tokens)
    }

    fn multi_get(&self, n: usize) -> Option<&str> {
        if self.pos + n >= self.user_input.len() { return None }

        Some(&self.user_input[self.pos .. (self.pos + n)])
    }

    fn current(&self) -> Option<char> {
        self.user_input.chars().nth(self.pos)
    }

    fn read_string_literal(&mut self) -> Result<Vec<u8>, String> {
        // skip first '"'
        self.increment_pos(1);

        let mut str_content = Vec::<u8>::new();

        while let Some(c) = self.current() {
            match c {
                // insert null char at last
                '"' => {
                    self.increment_pos(1);
                    // push '\0'
                    str_content.push(0);
                    break
                },
                // escaped
                '\\' => {
                    self.increment_pos(1);
                    str_content.push(self.read_escaped_literal());
                },
                _ => {
                    self.increment_pos(1);
                    str_content.push(c as u8);
                }

            }
        }

        Ok(str_content)
    }

    fn read_escaped_literal(&mut self) -> u8 {
        let escaped = self.current().map(|c| {
            match c {
                'a' => 7,
                'b' => 8,
                't' => 9,
                'n' => 10,
                'v' => 11,
                'f' => 12,
                'r' => 13,
                'e' => 27,
                '0' => 0,
                _ => c as u8 // 「escapeできない文字だよ」でもいい説ある
            }
        });

        self.increment_pos(1);

        escaped.expect("failed read_escaped_literal")
    }

    fn read_char_literal(&mut self) -> Result<u8, String>{
         // skip first '\''
        self.increment_pos(1);

        let ch = match self.current() {
            Some('\0') | None => return Err("unclosed char literal".to_string()),
            Some('\\') => {
                self.increment_pos(1);

                self.read_escaped_literal()
            },
            Some(c) => {
                self.increment_pos(1);

                c as u8
            }
        };

        if let Some('\'') = self.current() {
            self.increment_pos(1);
        } else {
            return Err("char literal too long".to_string())
        }

        Ok(ch)
    }

    fn strtol(&mut self, base: u32) -> Result<isize, String> {
        let start = self.pos;
        while let Some(true) = self.current().map(|digit| digit.is_digit(base)) {
            self.increment_pos(1);
        }

        isize::from_str_radix(&self.user_input[start .. self.pos], base)
            .or(Err("failed parsing num".to_string()))
    }

    fn get_letter(&mut self) -> String {
        let mut letter = String::new();
        while let Some(c) = self.current() {
            match c {
                // '0' ..= '9'は呼び出し側で排除されているので，ここでは含めて良い(文字列の先頭以外は含めて良い)
                'a' ..= 'z' | 'A' ..= 'Z' | '0' ..= '9' | '_' => {
                    self.increment_pos(1);

                    letter.push(c);
                },
                _ => break

            }
        }

        letter
    }

    fn block_comment(&mut self) -> Result<(), String> {
        while let Some(false) = self.multi_get(2).map(|char_slice| char_slice == "*/") {
            self.increment_pos(1);
        }

        self.increment_pos(2);

        if self.pos >= self.user_input.len() - 1 {
            return Err("unclosed block comment".to_string())
        }

        Ok(())
    }

    fn starts_with_multi_letter_punct(&self) -> Option<String> {
        MULTI_LETTER_PUNCTUACTORS.iter().find(|punct| {
            **punct == self.multi_get(punct.len()).unwrap_or("")
            })
            .map(|punct| punct.to_string())
    }

    fn strncasecmp(&self, n: usize, string: &str) -> bool {
        self.multi_get(n)
            .map(|chs| chs.to_lowercase() == string.to_lowercase())
            .unwrap_or(false)
    }

    fn is_ascii_alphanumeric(&self, pos: usize) -> bool {
        self.user_input.chars()
            .nth(self.pos + pos)
            .map(|c| c.is_ascii_alphanumeric())
            .unwrap_or(false)
    }

    fn get_int_base_num(&mut self) -> usize {
        if self.strncasecmp(2, "0x") && self.is_ascii_alphanumeric(2) {
            16
        } else if self.strncasecmp(2, "0b") && self.is_ascii_alphanumeric(2) {
            2
        } else if self.current().map_or(false, |c| c == '0') {
            8
        } else {
            10
        }
    }

    fn read_int_literal(&mut self) -> Result<TokenType, String> {
        let start = self.pos;
        let base = self.get_int_base_num();
        match base {
            2 | 16 => self.increment_pos(2),
            _ => {}
        };

        let val = self.strtol(base as u32)?;

        if self.current().map_or(false, |c| c.is_ascii_alphanumeric()) {
            return Err("invalid digit".to_string())
        } else {
            Ok(TokenType::Num(
                Num {
                    val,
                    tk_str: Rc::new(String::from(&self.user_input[start .. self.pos]))
                }
            ))
        }
    }

    pub fn col_number(&self) -> usize {
        self.current_col_index + 1
    }

    pub fn row_number(&self) -> usize {
        self.current_row_index + 1
    }

    fn new_token(&self, token_type: TokenType) -> Token {
        Token::new(
            token_type,
            Loc::new
            (self.row_number(), self.col_number())
        )
    }

    fn increment_pos(&mut self, count: usize) {
        self.pos += count;
        self.current_col_index += count;
    }
}
