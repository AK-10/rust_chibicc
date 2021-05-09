use crate::token::{ Token, TokenType };
use crate::token::token_type::*;
use self::loc::Loc;

use std::str::FromStr;
use std::rc::Rc;

pub mod loc;

// TODO: TokenzieErrorの定義
// TODO: Location structがほしい

const KEYWORDS: [&str; 14] = [
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
    "typedef"
];

pub struct Tokenizer<'a> {
    user_input: &'a [String],
    current_col_index: usize,
    current_row_index: usize,
    pos: usize
}

impl<'a> Tokenizer<'a> {
    pub fn new(user_input: &'a [String]) -> Self {
        // TODO: should fix, maybe.
        Self {
            user_input,
            current_row_index: 0, // user_input index
            current_col_index: 0, // current_str index
            pos: 0
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::<Token>::new();

        while self.current_col_index < self.user_input.len() {
            let loc = self.new_loc();
            let c = self.current().expect("current_col is out of user_input range");
            match c {
                // line comment
                _ if self.multi_get(2)
                    .map(|line_comment| line_comment == "//")
                    .unwrap_or(false) => {
                        while self.current().unwrap() != '\n' {
                            self.step_forward(1);
                        }
                },
                // block comment
                _ if self.multi_get(2)
                    .map(|block_comment| block_comment == "/*")
                    .unwrap_or(false) => {
                        self.block_comment()?;
                },
                // arrow operator
                _ if self.multi_get(2)
                    .map(|block_comment| block_comment == "->")
                    .unwrap_or(false) => {
                        let token_type = self.arrow()?;
                        tokens.push(Token::new(token_type, loc));
                },
                // "=" or "=="
                '=' => {
                    let token_type = self.tokenize_eq()?;
                    tokens.push(Token::new(token_type, loc));
                },
                // "!="
                '!' => {
                    let token_type = self.tokenize_not()?;
                    tokens.push(Token::new(token_type, loc));
                },
                // "<" or "<="
                '<' => {
                    let token_type = self.tokenize_lt()?;
                    tokens.push(Token::new(token_type, loc));
                },
                '>' => {
                    let token_type = self.tokenize_gt()?;
                    tokens.push(Token::new(token_type, loc));
                }
                // binary or unary oeprator
                '+' | '-' | '*' | '&' | '/' => {
                    self.step_forward(1);

                    let op = c.to_string();
                    let rc_op = Rc::new(op);
                    let reserved = Reserved {
                        op: Rc::clone(&rc_op),
                        tk_str: rc_op
                    };
                    let token_type = TokenType::Reserved(reserved);
                    tokens.push(Token::new(token_type, loc));
                },
                // symbol
                '(' | ')' | ';' | '{' | '}' | '.' | ',' | '[' | ']' => {
                    self.step_forward(1);

                    let sym = c.to_string();
                    let rc_sym = Rc::new(sym);
                    let symbol = Symbol {
                        sym: Rc::clone(&rc_sym),
                        tk_str: rc_sym
                    };
                    let token_type = TokenType::Symbol(symbol);
                    tokens.push(Token::new(token_type, loc));
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

                                    let token_type = TokenType::Str(str_type);
                                    tokens.push(Token::new(token_type, loc));
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

                    tokens.push(Token::new(token_type, loc));
                }
                // num
                '0' ..= '9' => {
                    let (num, tk_str) = self.strtol::<isize>()?;
                    let num_type = Num {
                        val: num,
                        tk_str: Rc::new(tk_str)
                    };
                    let token_type = TokenType::Num(num_type);

                    tokens.push(Token::new(token_type, loc));
                },
                ws if ws.is_whitespace() => {
                    self.step_forward(1);
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

                        let token_type = TokenType::Reserved(reserved_type);
                        tokens.push(Token::new(token_type, loc));
                    } else {
                        let ident_type = Ident {
                            name: Rc::clone(&rc_letter),
                            tk_str: rc_letter
                        };

                        let token_type = TokenType::Ident(ident_type);
                        tokens.push(Token::new(token_type, loc));
                    }
                },
                unsupported => {
                    // unsuupported character
                    let msg = format!("unsupported character: {}", unsupported);
                    return Err(msg)
                }
            }
        }

        tokens.push(Token {
            token_type: TokenType::Eof,
            loc: Loc {
                row: 999999,
                col: 999999
            }
        });

        Ok(tokens)
    }

    fn multi_get(&self, n: usize) -> Option<&'a str> {
        if self.current_col_index + n >= self.user_input.len() { return None }

        Some(&self.current_row()[self.current_col_index .. (self.current_col_index + n)])
    }

    fn current(&self) -> Option<char> {
        self.current_row()
            .chars()
            .nth(self.current_col_index)
    }

    fn current_row(&self) -> &'a String {
        // 面倒なのでexpectしている
        // ちゃんとしたほうが良いかも
        let x = self.user_input.get(self.current_row_index);
        x.expect(format!("current_row_index: {} is out of range", self.current_row_index).as_str())
    }

    fn tokenize_eq(&mut self) -> Result<TokenType, String> {
        self.step_forward(1);

        let op = match self.current() {
            Some('=') => {
                self.step_forward(1);
                "==".to_string()
            },
            Some(_) => "=".to_string(),
            _ => return Err("token must exist after =".to_string())
        };

        let rc_op = Rc::new(op);
        let reserved_type = Reserved {
            op: Rc::clone(&rc_op),
            tk_str: rc_op
        };

        Ok(TokenType::Reserved(reserved_type))
    }

    fn tokenize_not(&mut self) -> Result<TokenType, String> {
        self.step_forward(1);
        match self.current() {
            Some('=') => {
                self.step_forward(1);
                let op = "!=".to_string();
                let rc_op = Rc::new(op);

                let reserved_type = Reserved {
                    op: Rc::clone(&rc_op),
                    tk_str: rc_op
                };

                Ok(TokenType::Reserved(reserved_type))
            },
            _ => Err("token must exist after !".to_string())
        }
    }

    fn tokenize_lt(&mut self) -> Result<TokenType, String> {
        self.step_forward(1);
        let op = match self.current() {
            Some('=') => {
                self.step_forward(1);
                "<=".to_string()
            },
            Some(_) => "<".to_string(),
            _ => return Err("token must exist after <".to_string())
        };

        let rc_op = Rc::new(op);
        let reserved_type = Reserved {
            op: Rc::clone(&rc_op),
            tk_str: rc_op
        };
        Ok(TokenType::Reserved(reserved_type))
    }

    fn tokenize_gt(&mut self) -> Result<TokenType, String> {
        self.step_forward(1);
        let op = match self.current() {
            Some('=') => {
                self.step_forward(1);
                ">=".to_string()
            },
            Some(_) => ">".to_string(),
            _ => return Err("token must exist after >".to_string())
        };

        let rc_op = Rc::new(op);
        let reserved_type = Reserved {
            op: Rc::clone(&rc_op),
            tk_str: rc_op
        };
        Ok(TokenType::Reserved(reserved_type))
    }

    fn read_string_literal(&mut self) -> Result<Vec<u8>, String> {
        // skip first '"'
        self.step_forward(1);

        let mut str_content = Vec::<u8>::new();

        while let Some(c) = self.current() {
            match c {
                // insert null char at last
                '"' => {
                    self.step_forward(1);
                    // push '\0'
                    str_content.push(0);
                    break
                },
                // escaped
                '\\' => {
                    self.step_forward(1);
                    str_content.push(self.read_escaped_literal());
                },
                _ => {
                    self.step_forward(1);
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

        self.step_forward(1);

        escaped.expect("failed read_escaped_literal")
    }

    fn read_char_literal(&mut self) -> Result<u8, String>{
         // skip first '\''
        self.step_forward(1);

        let ch = match self.current() {
            Some('\0') | None => return Err("unclosed char literal".to_string()),
            Some('\\') => {
                self.step_forward(1);

                self.read_escaped_literal()
            },
            Some(c) => {
                self.step_forward(1);

                c as u8
            }
        };

        if let Some('\'') = self.current() {
            self.step_forward(1);
        } else {
            return Err("char literal too long".to_string())
        }

        Ok(ch)
    }

    fn strtol<T: FromStr>(&mut self) -> Result<(T, String), String> {
        let mut num_str = String::new();
        while let Some(c) = self.current() {
            match c {
                num if num.is_digit(10) => {
                    self.step_forward(1);
                    num_str.push(num);
                },
                _ => break
            }
        }

        num_str.parse::<T>()
            .map(|i| (i, num_str)) // Result<T, String> -> Result<(T, String), String>
            .or(Err("failed pasring num".to_string()))
    }

    fn get_letter(&mut self) -> String {
        let mut letter = String::new();
        while let Some(c) = self.current() {
            match c {
                // '0' ..= '9'は呼び出し側で排除されているので，ここでは含めて良い(文字列の先頭以外は含めて良い)
                'a' ..= 'z' | 'A' ..= 'Z' | '0' ..= '9' | '_' => {
                    self.step_forward(1);
                    letter.push(c);
                },
                _ => break

            }
        }

        letter
    }

    fn block_comment(&mut self) -> Result<(), String> {
        while let Some(false) = self.multi_get(2).map(|char_slice| char_slice == "*/") {
            self.step_forward(1);
        }

        self.step_forward(2);

        if self.pos >= self.user_input.len() - 1 {
            return Err("unclosed block comment".to_string())
        }

        Ok(())
    }

    fn arrow(&mut self) -> Result<TokenType, String> {
        let arrow_str = Rc::new("->".to_string());
        self.step_forward(2);

        Ok(TokenType::Reserved(
            Reserved {
                op: Rc::clone(&arrow_str),
                tk_str: arrow_str
            })
        )
    }

    fn step_forward(&mut self, n: usize) {
        // straggle user_input
        // reset current_col, current_row
        if self.current_col_index + n >= self.current_row().len() {
            self.current_col_index = 0;
            self.current_row_index += 1;
            if self.current_row().as_str() == "" {
                self.current_row_index += 1;
            }
        } else {
            self.current_col_index += n;
        }
        self.pos += n;
    }

    fn row_number(&self) -> usize {
        self.current_row_index + 1
    }

    fn col_number(&self) -> usize {
        self.current_col_index + 1
    }

    fn new_loc(&self) -> Loc {
        Loc::new(self.row_number(), self.col_number())
    }
}
