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
                _ if self.multi_get(2) == ['/', '/'] => {
                    while self.current().unwrap() != '\n' {
                        self.pos += 1;
                    }
                },
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
                'a' ..= 'z' | 'A' ..= 'Z' => {
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

    fn multi_get(&self, n: usize) -> &[char] {
        &self.user_input[self.pos ..= (self.pos + n)]
    }

    fn current(&self) -> Option<char> {
        self.user_input.get(self.pos).map(|c| *c)
    }

    fn tokenize_eq(&mut self) -> Result<Token, String> {
        self.pos += 1;
        match self.user_input.get(self.pos) {
            Some('=') => {
                self.pos += 1;
                Ok(Token::Reserved { op: "==".to_string() })
            },
            Some(_) => Ok(Token::Reserved { op: "=".to_string() }),
            _ => Err("token must exist after =".to_string())
        }
    }

    fn tokenize_not(&mut self) -> Result<Token, String> {
        match self.user_input.get(self.pos) {
            Some('=') => {
                self.pos += 2;
                Ok(Token::Reserved { op: "!=".to_string() })
            },
            _ => Err("token must exist after !".to_string())
        }
    }

    fn tokenize_lt(&mut self) -> Result<Token, String> {
        self.pos += 1;
        match self.user_input.get(self.pos) {
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
        match self.user_input.get(self.pos) {
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
                'a' ..= 'z' | 'A' ..= 'Z' | '0' ..= '9' | '|' => {
                    self.pos += 1;
                    letter.push(c);
                },
                _ => break

            }
        }

        letter
    }
}

// pub fn tokenize(line: String) -> Result<Vec<Token>, String> {
//     let mut tokens: Vec<Token> = Vec::new();
//     let chars_with_index = &mut line.chars().enumerate().peekable();
// 
//     while let Some((i, ch)) = chars_with_index.peek() {
//         match ch {
//             '=' => {
//                 chars_with_index.next();
//                 tokens.push(tokenize_eq(chars_with_index));
//             },
//             '!' => {
//                 let i = *i;
//                 chars_with_index.next();
// 
//                 match chars_with_index.peek() {
//                     Some((_, '=')) => {
//                         chars_with_index.next();
//                         let token = Token::Reserved { op: "!=".to_string() };
//                         tokens.push(token);
//                     }
//                     Some((idx, _)) => {
//                         let space = (0..*idx).fold(String::new(), |a, _| a + " " ) + "^";
//                         eprintln!("{}", line);
//                         eprintln!("{} can not parse", space);
//                         return Err("neq tokenization failed error".to_string());
//                     }
//                     None => {
//                         let space = (0..i).fold(String::new(), |a, _| a + " " ) + "^";
//                         eprintln!("{}", line);
//                         eprintln!("{} can not parse", space);
//                         return Err("neq tokenization failed error".to_string());
//                     }
//                 }
//             },
//             '<'=> {
//                 chars_with_index.next();
//                 tokens.push(tokenize_lt(chars_with_index));
//             },
//             '>' => {
//                 chars_with_index.next();
//                 tokens.push(tokenize_gt(chars_with_index));
//             },
//             '+' | '-' | '*' | '&' => {
//                 // TODO: Reserved -> Symbolに変更する
//                 let token = Token::Reserved { op: ch.to_string() };
//                 tokens.push(token);
//                 chars_with_index.next();
//             },
//            '/' => {
//                 let ch = chars_with_index.next().unwrap().0;
//                 if let Some((_, '/')) = chars_with_index.peek() {
//                     // skip line comments
//                     loop {
//                         if let Some((_, '\n')) = chars_with_index.next() {
//                             break
//                         }
//                     };
//                 } else {
//                     let token = Token::Reserved { op: ch.to_string() };
//                     tokens.push(token);
//                 }
//                 println!("{:#?}", chars_with_index);
//                 println!("{:?}", chars_with_index.peek());
//             }
//             '(' | ')' | ';' | '{' | '}' | ',' | '[' | ']' => {
//                 let token = Token::Symbol(ch.to_string());
//                 tokens.push(token);
//                 chars_with_index.next();
//             },
//             '"' => {
//                 let contents = read_string_literal(chars_with_index);
//                 match contents {
//                     Ok(c) => tokens.push(Token::Str(c)),
//                     Err(e) => {
//                         let msg  = format!("{:?}", e);
//                         panic!("{:?}", msg);
//                     }
//                 }
//              }
//             '0'..='9' => {
//                 // chars_with_index.peek()で可変な参照をしてるのでここでiの参照外しをする.
//                 // そうしないとstrtol::<usize>(chars_with_index)ができない?(あんまりわかってない)
//                 let idx = *i;
//                 let num_result = strtol::<isize>(chars_with_index);
//                 match num_result {
//                     Ok(num) => {
//                         let token = Token::Num{
//                             val: num,
//                             t_str: num.to_string(),
//                         };
// 
//                         // strtolで既に数字の次まで進んでいるのでchars.next()はしない
//                         tokens.push(token);
//                     },
//                     Err(_) => {
//                         let space = (0..idx).fold(String::new(), |a, _| a + " " ) + "^";
//                         eprintln!("{}", line);
//                         eprintln!("{} not a number", space);
//                         return Err("not num error".to_string());
//                     }
//                 }
//             }
//             ws if ws.is_whitespace() => {
//                 chars_with_index.next();
//                 continue;
//             },
//             'a'..='z' | 'A'..='Z' | '_' => {
//                 let _i = *i;
//                 let letter = get_letter(chars_with_index);
//                 if KEYWORDS.contains(&&*letter) {
//                     tokens.push(Token::Reserved { op: letter })
//                 } else {
//                     let token = Token::Ident { name: letter };
//                     tokens.push(token);
//                     // chars_with_index.next();
//                 }
//             }
//             _ => {
//                 let space = (0..*i).fold(String::new(), |a, _| a + " " ) + "^";
//                 eprintln!("{}", line);
//                 eprintln!("{} tokenizeできません", space);
//                 chars_with_index.next();
//                 return Err("not assumption character error".to_string());
//             }
//         };
//     };
//     tokens.push(Token::Eof);
// 
//     Ok(tokens)
// }
// 
// fn strtol<T: FromStr>(chars: &mut Peekable<Enumerate<Chars>>) -> Result<T, String> {
//     let mut num = String::new();
//     while let Some((_, ch)) = chars.peek() {
//         match ch {
//             '0'..='9' => {
//                 num.push(*ch);
//                 chars.next();
//             },
//             _ => {
//                 break;
//             }
//         }
//     }
// 
//     num.parse::<T>().or(Err("parse failed".to_string()))
// }
// 
// fn get_letter(chars: &mut Peekable<Enumerate<Chars>>) -> String {
//     let mut letter = String::new();
//     while let Some((_, ch)) = chars.peek() {
//         match ch {
//             'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
//                 letter.push(*ch);
//                 chars.next();
//             },
//             _ => { break; }
//         }
//     };
// 
//     letter
// }
// 
// fn read_string_literal(chars: &mut Peekable<Enumerate<Chars>>) -> Result<Vec<u8>, String> {
//     // 最初の'"'を飛ばす
//     chars.next();
// 
//     let mut str_content = Vec::<u8>::new();
// 
//     while let Some((_, c)) = chars.next() {
//         match c {
//             '"' => {
//                 // null文字の挿入
//                 str_content.push(0);
//                 break
//             },
//             '\\' => { str_content.push(read_escaped_literal(chars)); },
//             _ => { str_content.push(c as u8); }
//         }
//     };
// 
//     Ok(str_content)
// }
// 
//
// fn tokenize_eq(chars_with_index: &mut Peekable<Enumerate<Chars>>) -> Token {
//     match chars_with_index.peek() {
//         Some((_, '=')) => {
//             chars_with_index.next();
//             Token::Reserved { op: "==".to_string() }
//         }
//         _ => Token::Reserved { op: "=".to_string() }
//     }
// }
// 
// fn tokenize_lt(chars_with_index: &mut Peekable<Enumerate<Chars>>) -> Token {
//     match chars_with_index.peek() {
//         Some((_, '=')) => {
//             chars_with_index.next();
//             Token::Reserved { op: "<=".to_string() }
//         }
//         _ => Token::Reserved { op: "<".to_string() }
//     }
// }
// 
// fn tokenize_gt(chars_with_index: &mut Peekable<Enumerate<Chars>>) -> Token {
//     // map がself(chars_with_index)へのimmutable borrowを持っているのでダメ
//     // closure内部でchars_with_index.next()ができない
//     // chars_with_index.peek().map(|(_, ch)| {
//     //     match ch {
//     //         '=' => {
//     //             chars_with_index.next();
//     //             Token::Reserved { op: ">=".to_string() }
//     //         }
//     //         _ => Token::Reserved { op: ">".to_string() },
//     //     }
//     // })
//     match chars_with_index.peek() {
//         Some((_, '=')) => {
//             chars_with_index.next();
//             Token::Reserved { op: ">=".to_string() }
//         }
//         _ => Token::Reserved { op: ">".to_string() }
//     }
// }
