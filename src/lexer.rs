use crate::token::Token;

use std::str::{Chars, FromStr};
use std::iter::{Peekable, Enumerate};

// TODO: LexerErrorの定義

// 本当はimpl Iter<Item=Token>を返したい
// pub fn tokenize(chars: &mut Peekable<Chars>) -> impl Iter<Item=Token>
pub fn tokenize(line: String) -> Result<Vec<Token>, String> {
    let mut tokens: Vec<Token> = Vec::new();
    let chars_with_index = &mut line.chars().enumerate().peekable();

    while let Some((i, ch)) = chars_with_index.peek() {
        if ch.is_ascii_whitespace() {
            chars_with_index.next();
            continue;
        }

        match ch {
            '+' | '-' => {
                let token = Token::Reserved {
                    op: *ch,
                    t_str: ch.to_string(),
                };

                tokens.push(token);
                chars_with_index.next();
            },
            '0'..='9' => {
                // chars_with_index.peek()で可変な参照をしてるのでここでiの参照外しをする.
                // そうしないとstrtol::<usize>(chars_with_index)ができない(あんまりわかってない)
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
                        eprintln!("{} 数ではありません", space);
                        return Err("not num error".to_string());
                    }
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

#[test]
fn tokenize_test() {
    let input = " 1_ + 2 + 3 -20 ".to_string();
    let result = tokenize(input);
    let expected = vec![
        Token::Num { val: 1, t_str: "1".to_string() },
        Token::Reserved { op: '+', t_str: "+".to_string() },
        Token::Num { val: 2, t_str: "2".to_string() },
        Token::Reserved { op: '+', t_str: "+".to_string() },
        Token::Num { val: 3, t_str: "3".to_string() },
        Token::Reserved { op: '-', t_str: "-".to_string() },
        Token::Num { val: 20, t_str: "20".to_string() },
        Token::Eof
    ];

    assert_eq!(result, expected);
}