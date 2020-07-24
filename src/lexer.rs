use crate::token::Token;
use crate::strtol;

use std::str::{Chars};
use std::iter::{Peekable};


pub fn tokenize(chars: &mut Peekable<Chars>) -> Option<Token> {
    let mut cur_token: Option<Token> = None;

    while let Some(ch) = chars.peek() {
        println!("{:?}", cur_token);
        if ch.is_ascii_whitespace() {
            chars.next();
            continue;
        }

        match ch {
            '+' | '-' => {
                let token = Token::Reserved {
                    op: *ch,
                    t_str: ch.to_string(),
                    next: Box::new(None)
                };

                set_next(&mut cur_token, &token);
                cur_token = Some(token);

                chars.next();
            },
            '0'..='9' => {
                let num = strtol::<usize>(chars).expect("数字ではありません");
                let token = Token::Num{
                    val: num,
                    t_str: num.to_string(),
                    next: Box::new(None),
                };

                set_next(&mut cur_token, &token);
                cur_token = Some(token);

                chars.next();
            }
            _ => {
                continue;
            }
        };
    }

    cur_token
}

// cur_tokenは参照先への書き込みをしたいので型が&mut Option<Token>
// mut cur_token: Option<Token>だと参照先が変更されない(おそらくcopy渡しである)
fn set_next(cur_token: &mut Option<Token>, next: &Token) {
    if let Some(tk) = cur_token {
        *cur_token = match tk {
            Token::Reserved { op, t_str, .. } => Some(
                Token::Reserved {
                    op: *op,
                    t_str: t_str.to_string(),
                    next: Box::new(Some(next.clone()))
                }
            ),
            Token::Num { val, t_str, .. } => Some(
                Token::Num {
                    val: *val,
                    t_str: t_str.to_string(),
                    next: Box::new(Some(next.clone()))
                }
            ),
            Token::Eof => Some(Token::Eof)
        };
    } else {
        *cur_token = Some(next.clone());
    }
}

#[test]
fn tokenize_test() {
    let input = &mut "1 + 2 + 3 -2".chars().peekable();
    let expected = tokenize(input);

    println!("{:?}", expected);
}