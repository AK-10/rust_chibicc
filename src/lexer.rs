use crate::token::Token;
use crate::strtol;

use std::str::{Chars};
use std::iter::{Peekable};

// 本当はimpl Iter<Item=Token>を返したい
// pub fn tokenize(chars: &mut Peekable<Chars>) -> impl Iter<Item=Token>
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
                // strtolで既に数字の次まで進んでいるのでchars.next()はしない
                let num = strtol::<usize>(chars).expect("数字ではありません");
                let token = Token::Num{
                    val: num,
                    t_str: num.to_string(),
                };

                tokens.push(token);
            }
            _ => {
                // 本当はエラーにするべき
                chars.next();
                continue;
            }
        };
    };
    tokens.push(Token::Eof);

    tokens
}

#[test]
fn tokenize_test() {
    let input = &mut " 1 + 2 + 3 -20 ".chars().peekable();
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