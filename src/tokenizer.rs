use crate::token::Token;
use std::str::FromStr;

// TODO: LexerErrorの定義
// TODO: Location structがほしい

const KEYWORDS: [&str; 8] = ["return", "if", "while", "else", "for", "int", "char", "sizeof"];

pub struct Tokenizer {
    user_input: Vec<char>,
    pos: usize
}

impl<'a> Tokenizer {
    pub fn new(user_input: &'_ str) -> Self {
        Self {
            user_input: user_input.chars().collect(),
            pos: 0
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::<Token>::new();

        while self.pos <= self.user_input.len() - 1 {
            let c = self.current().expect("pos is out of user_input range");
            match c {
                // line comment
                _ if self.multi_get(2)
                    .map(|line_comment| line_comment == ['/', '/'])
                    .unwrap_or(false) => {
                        while self.current().unwrap() != '\n' {
                            self.pos += 1;
                        }
                },
                // block comment
                _ if self.multi_get(2)
                    .map(|block_comment| block_comment == ['/', '*'])
                    .unwrap_or(false) => {
                        self.block_comment()?;
                }
                // "=" or "=="
                '=' => {
                    let token = self.tokenize_eq()?;
                    tokens.push(token);
                },
                // "!="
                '!' => {
                    let token = self.tokenize_not()?;
                    tokens.push(token);
                },
                // "<" or "<="
                '<' => {
                    let token = self.tokenize_lt()?;
                    tokens.push(token);
                },
                '>' => {
                    let token = self.tokenize_gt()?;
                    tokens.push(token);
                }
                // binary or unary oeprator
                '+' | '-' | '*' | '&' | '/' => {
                    self.pos += 1;
                    let token = Token::Reserved { op: c.to_string() };
                    tokens.push(token);
                },
                // symbol
                '(' | ')' | ';' | '{' | '}' | ',' | '[' | ']' => {
                    self.pos += 1;
                    let token = Token::Symbol(c.to_string());
                    tokens.push(token);
                },
                // string
                '"' => {
                    let contents = self.read_string_literal();
                    match contents {
                        Ok(c) => tokens.push(Token::Str(c)),
                        Err(e) => {
                            let msg = format!("error occured in tokenizing string: {}", e);
                            return Err(msg)
                        }
                    }
                },
                // num
                '0' ..= '9' => {
                    let num = self.strtol::<isize>()?;
                    let token = Token::Num {
                        val: num,
                        t_str: num.to_string()
                    };

                    tokens.push(token);
                },
                ws if ws.is_whitespace() => {
                    self.pos += 1;
                    continue
                },
                // ident or reserved
                'a' ..= 'z' | 'A' ..= 'Z' | '_' => {
                    let letter = self.get_letter();
                    if KEYWORDS.contains(&&*letter) {
                        tokens.push(Token::Reserved{ op: letter });
                    } else {
                        tokens.push(Token::Ident { name: letter })
                    }
                },
                unsupported => {
                    // unsuupported character
                    let msg = format!("unsupported character: {}", unsupported);
                    return Err(msg)
                }
            }
        }

        tokens.push(Token::Eof);

        Ok(tokens)
    }

    fn multi_get(&self, n: usize) -> Option<&[char]> {
        if self.pos + n >= self.user_input.len() { return None }

        Some(&self.user_input[self.pos .. (self.pos + n)])
    }

    fn current(&self) -> Option<char> {
        self.user_input.get(self.pos).map(|c| *c)
    }

    fn tokenize_eq(&mut self) -> Result<Token, String> {
        self.pos += 1;
        match self.current() {
            Some('=') => {
                self.pos += 1;
                Ok(Token::Reserved { op: "==".to_string() })
            },
            Some(_) => Ok(Token::Reserved { op: "=".to_string() }),
            _ => Err("token must exist after =".to_string())
        }
    }

    fn tokenize_not(&mut self) -> Result<Token, String> {
        self.pos += 1;
        match self.current() {
            Some('=') => {
                self.pos += 1;
                Ok(Token::Reserved { op: "!=".to_string() })
            },
            _ => Err("token must exist after !".to_string())
        }
    }

    fn tokenize_lt(&mut self) -> Result<Token, String> {
        self.pos += 1;
        match self.current() {
            Some('=') => {
                self.pos += 1;
                Ok(Token::Reserved { op: "<=".to_string() })
            },
            Some(_) => Ok(Token::Reserved { op: "<".to_string() }),
            _ => Err("token must exist after >".to_string())
        }
    }

    fn tokenize_gt(&mut self) -> Result<Token, String> {
        self.pos += 1;
        match self.current() {
            Some('=') => {
                self.pos += 1;
                Ok(Token::Reserved { op: ">=".to_string() })
            },
            Some(_) => Ok(Token::Reserved { op: ">".to_string() }),
            _ => Err("token must exist after >".to_string())
        }
    }

    fn read_string_literal(&mut self) -> Result<Vec<u8>, String> {
        // skip first '"'
        self.pos += 1;

        let mut str_content = Vec::<u8>::new();

        while let Some(c) = self.current() {
            match c {
                // insert null char at last
                '"' => {
                    self.pos += 1;
                    // push '\0'
                    str_content.push(0);
                    break
                },
                // escaped
                '\\' => {
                    self.pos += 1;
                    str_content.push(self.read_escaped_literal());
                },
                _ => {
                    self.pos += 1;
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

        self.pos += 1;

        escaped.expect("failed read_escaped_literal")
    }

    fn strtol<T: FromStr>(&mut self) -> Result<T, String> {
        let mut num_str = String::new();
        while let Some(c) = self.current() {
            match c {
                num if num.is_digit(10) => {
                    self.pos += 1;
                    num_str.push(num);
                },
                _ => break
            }
        }

        num_str.parse::<T>()
            .or(Err("failed pasring num".to_string()))
    }

    fn get_letter(&mut self) -> String {
        let mut letter = String::new();
        while let Some(c) = self.current() {
            match c {
                // '0' ..= '9'は呼び出し側で排除されているので，ここでは含めて良い(文字列の先頭以外は含めて良い)
                'a' ..= 'z' | 'A' ..= 'Z' | '0' ..= '9' | '_' => {
                    self.pos += 1;
                    letter.push(c);
                },
                _ => break

            }
        }

        letter
    }

    fn block_comment(&mut self) -> Result<(), String> {
        while let Some(false) = self.multi_get(2).map(|char_slice| char_slice == ['*', '/']) {
            self.pos += 1;
        }

        self.pos += 2;

        if self.pos >= self.user_input.len() - 1 {
            return Err("unclosed block comment".to_string())
        }

        Ok(())
    }
}

