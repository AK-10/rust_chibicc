use crate::token::Token;

use std::str::{Chars, FromStr};
use std::iter::{Peekable, Enumerate};

// TODO: LexerErrorの定義

// 本当はimpl Iter<Item=Token>を返したい
// pub fn tokenize(chars: &mut Peekable<Chars>) -> impl Iter<Item=Token>

const KEYWORDS: [&str; 8] = ["return", "if", "while", "else", "for", "int", "char", "sizeof"];

pub fn tokenize(line: String) -> Result<Vec<Token>, String> {
    let mut tokens: Vec<Token> = Vec::new();
    let chars_with_index = &mut line.chars().enumerate().peekable();

    while let Some((i, ch)) = chars_with_index.peek() {
        match ch {
            '=' => {
                chars_with_index.next();
                tokens.push(tokenize_eq(chars_with_index));
            },
            '!' => {
                let i = *i;
                chars_with_index.next();

                match chars_with_index.peek() {
                    Some((_, '=')) => {
                        chars_with_index.next();
                        let token = Token::Reserved { op: "!=".to_string() };
                        tokens.push(token);
                    }
                    Some((idx, _)) => {
                        let space = (0..*idx).fold(String::new(), |a, _| a + " " ) + "^";
                        eprintln!("{}", line);
                        eprintln!("{} can not parse", space);
                        return Err("neq tokenization failed error".to_string());
                    }
                    None => {
                        let space = (0..i).fold(String::new(), |a, _| a + " " ) + "^";
                        eprintln!("{}", line);
                        eprintln!("{} can not parse", space);
                        return Err("neq tokenization failed error".to_string());
                    }
                }
            },
            '<'=> {
                chars_with_index.next();
                tokens.push(tokenize_lt(chars_with_index));
            },
            '>' => {
                chars_with_index.next();
                tokens.push(tokenize_gt(chars_with_index));
            },
            '+' | '-' | '*' | '/' | '&' => {
                // Todo: Reserved -> Symbolに変更する
                let token = Token::Reserved { op: ch.to_string() };
                tokens.push(token);
                chars_with_index.next();
            },
            '(' | ')' | ';' | '{' | '}' | ',' | '[' | ']' => {
                let token = Token::Symbol(ch.to_string());
                tokens.push(token);
                chars_with_index.next();
            },
            '"' => {
                let contents = read_string_literal(chars_with_index);
                match contents {
                    Ok(c) => tokens.push(Token::Str(c)),
                    Err(e) => {
                        let msg  = format!("{:?}", e);
                        panic!("{:?}", msg);
                    }
                }
             }
            '0'..='9' => {
                // chars_with_index.peek()で可変な参照をしてるのでここでiの参照外しをする.
                // そうしないとstrtol::<usize>(chars_with_index)ができない?(あんまりわかってない)
                let idx = *i;
                let num_result = strtol::<isize>(chars_with_index);
                match num_result {
                    Ok(num) => {
                        let token = Token::Num{
                            val: num,
                            t_str: num.to_string(),
                        };

                        // strtolで既に数字の次まで進んでいるのでchars.next()はしない
                        tokens.push(token);
                    },
                    Err(_) => {
                        let space = (0..idx).fold(String::new(), |a, _| a + " " ) + "^";
                        eprintln!("{}", line);
                        eprintln!("{} not a number", space);
                        return Err("not num error".to_string());
                    }
                }
            }
            ws if ws.is_whitespace() => {
                chars_with_index.next();
                continue;
            },
            'a'..='z' | 'A'..='Z' | '_' => {
                let _i = *i;
                let letter = get_letter(chars_with_index);
                if KEYWORDS.contains(&&*letter) {
                    tokens.push(Token::Reserved { op: letter })
                } else {
                    let token = Token::Ident { name: letter };
                    tokens.push(token);
                    // chars_with_index.next();
                }
            }
            _ => {
                let space = (0..*i).fold(String::new(), |a, _| a + " " ) + "^";
                eprintln!("{}", line);
                eprintln!("{} tokenizeできません", space);
                chars_with_index.next();
                return Err("not assumption character error".to_string());
            }
        };
    };
    tokens.push(Token::Eof);

    Ok(tokens)
}

fn strtol<T: FromStr>(chars: &mut Peekable<Enumerate<Chars>>) -> Result<T, String> {
    let mut num = String::new();
    while let Some((_, ch)) = chars.peek() {
        match ch {
            '0'..='9' => {
                num.push(*ch);
                chars.next();
            },
            _ => {
                break;
            }
        }
    }

    num.parse::<T>().or(Err("parse failed".to_string()))
}

fn get_letter(chars: &mut Peekable<Enumerate<Chars>>) -> String {
    let mut letter = String::new();
    while let Some((_, ch)) = chars.peek() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                letter.push(*ch);
                chars.next();
            },
            _ => { break; }
        }
    };

    letter
}

fn read_string_literal(chars: &mut Peekable<Enumerate<Chars>>) -> Result<Vec<u8>, String> {
    // 最初の'"'を飛ばす
    chars.next();

    let mut str_content = Vec::<u8>::new();

    while let Some((_, c)) = chars.next() {
        match c {
            '"' => {
                str_content.push(0);
                break
            },
            '\\' => { str_content.push(read_escaped_literal(chars)); },
            _ => { str_content.push(c as u8); }
        }
    };

    Ok(str_content)
}

fn read_escaped_literal(chars: &mut Peekable<Enumerate<Chars>>) -> u8 {
    chars.next().map(|(_, ch)| {
        match ch {
            'a' => 7,
            'b' => 8,
            't' => 9,
            'n' => 10,
            'v' => 11,
            'f' => 12,
            'r' => 13,
            'e' => 27,
            '0' => 0,
            _ => ch as u8 // 「escapeできない文字だよ」でもいい説ある
        }
    }).expect("failed read_escaped_literal")
}

fn tokenize_eq(chars_with_index: &mut Peekable<Enumerate<Chars>>) -> Token {
    match chars_with_index.peek() {
        Some((_, '=')) => {
            chars_with_index.next();
            Token::Reserved { op: "==".to_string() }
        }
        _ => Token::Reserved { op: "=".to_string() }
    }
}

fn tokenize_lt(chars_with_index: &mut Peekable<Enumerate<Chars>>) -> Token {
    match chars_with_index.peek() {
        Some((_, '=')) => {
            chars_with_index.next();
            Token::Reserved { op: "<=".to_string() }
        }
        _ => Token::Reserved { op: "<".to_string() }
    }
}

fn tokenize_gt(chars_with_index: &mut Peekable<Enumerate<Chars>>) -> Token {
    // map がself(chars_with_index)へのimmutable borrowを持っているのでダメ
    // closure内部でchars_with_index.next()ができない
    // chars_with_index.peek().map(|(_, ch)| {
    //     match ch {
    //         '=' => {
    //             chars_with_index.next();
    //             Token::Reserved { op: ">=".to_string() }
    //         }
    //         _ => Token::Reserved { op: ">".to_string() },
    //     }
    // })
    match chars_with_index.peek() {
        Some((_, '=')) => {
            chars_with_index.next();
            Token::Reserved { op: ">=".to_string() }
        }
        _ => Token::Reserved { op: ">".to_string() }
    }
}

#[test]
fn tokenize_arithmetic_test() {
    let input = " 1 + 2 + 3 -20 ".to_string();
    let result = tokenize(input);
    let expected: Result<Vec<Token>, String> = Ok(vec![
        Token::Num { val: 1, t_str: "1".to_string() },
        Token::Reserved { op: "+".to_string() },
        Token::Num { val: 2, t_str: "2".to_string() },
        Token::Reserved { op: "+".to_string() },
        Token::Num { val: 3, t_str: "3".to_string() },
        Token::Reserved { op: "-".to_string() },
        Token::Num { val: 20, t_str: "20".to_string() },
        Token::Eof
    ]);

    assert_eq!(result, expected);
}

#[test]
fn tokenize_gt_test() {
    let input = "1 > 2".to_string();
    let result = tokenize(input);
    let expected: Result<Vec<Token>, String> = Ok(vec![
        Token::Num { val: 1, t_str: "1".to_string() },
        Token::Reserved { op: ">".to_string() },
        Token::Num { val: 2, t_str: "2".to_string() },
        Token::Eof
    ]);

    assert_eq!(result, expected);
}

#[test]
fn tokenize_ge_test() {
    let input = "1 >= 2".to_string();
    let result = tokenize(input);
    let expected: Result<Vec<Token>, String> = Ok(vec![
        Token::Num { val: 1, t_str: "1".to_string() },
        Token::Reserved { op: ">=".to_string() },
        Token::Num { val: 2, t_str: "2".to_string() },
        Token::Eof
    ]);

    assert_eq!(result, expected);
}

#[test]
fn tokenize_lt_test() {
    let input = "1 < 2".to_string();
    let result = tokenize(input);
    let expected: Result<Vec<Token>, String> = Ok(vec![
        Token::Num { val: 1, t_str: "1".to_string() },
        Token::Reserved { op: "<".to_string() },
        Token::Num { val: 2, t_str: "2".to_string() },
        Token::Eof
    ]);

    assert_eq!(result, expected);
}

#[test]
fn tokenize_le_test() {
    let input = "1 <= 2".to_string();
    let result = tokenize(input);
    let expected: Result<Vec<Token>, String> = Ok(vec![
        Token::Num { val: 1, t_str: "1".to_string() },
        Token::Reserved { op: "<=".to_string() },
        Token::Num { val: 2, t_str: "2".to_string() },
        Token::Eof
    ]);

    assert_eq!(result, expected);
}

#[test]
fn tokenize_eq_test() {
    let input = "1 == 2".to_string();
    let result = tokenize(input);
    let expected: Result<Vec<Token>, String> = Ok(vec![
        Token::Num { val: 1, t_str: "1".to_string() },
        Token::Reserved { op: "==".to_string() },
        Token::Num { val: 2, t_str: "2".to_string() },
        Token::Eof
    ]);

    assert_eq!(result, expected);
}

#[test]
fn tokenize_neq_test() {
    let input = "1 != 2".to_string();
    let result = tokenize(input);
    let expected: Result<Vec<Token>, String> = Ok(vec![
        Token::Num { val: 1, t_str: "1".to_string() },
        Token::Reserved { op: "!=".to_string() },
        Token::Num { val: 2, t_str: "2".to_string() },
        Token::Eof
    ]);

    assert_eq!(result, expected);
}

#[test]
fn tokenize_assign_test() {
    let input = "a = 2;".to_string();
    let result = tokenize(input);
    let expected: Result<Vec<Token>, String> = Ok(vec![
        Token::Ident { name: "a".to_string() },
        Token::Reserved { op: "=".to_string() },
        Token::Num { val: 2, t_str: "2".to_string() },
        Token::Reserved { op: ";".to_string() },
        Token::Eof
    ]);

    assert_eq!(result, expected);
}
