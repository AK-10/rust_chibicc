use crate::token::Token;
use crate::strtol;

use std::str::{Chars};
use std::iter::{Peekable};

pub fn tokenize(chars: &mut Peekable<Chars>) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    while let Some(ch) = chars.peek() {
        if ch.is_ascii_whitespace() {
            chars.next();
            continue;
        }

        match ch {
            '+' | '-' => {
                let token = Token::Reserved {
                    op: *ch,
                    t_str: ch.to_string(),
                };

                tokens.push(token);
                chars.next();
            },
            '0'..='9' => {
                let num = strtol::<usize>(chars).expect("数字ではありません");
                let token = Token::Num{
                    val: num,
                    t_str: num.to_string(),
                };

                tokens.push(token);
                chars.next();
            }
            _ => {
                continue;
            }
        };
    }

    tokens
}

#[test]
fn tokenize_test() {
    let input = &mut "1 + 2 + 3 -2".chars().peekable();
    let result = tokenize(input);

    println!("{:?}", result);
}