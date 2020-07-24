pub mod token;
pub mod lexer;

use std::str::{Chars, FromStr};
use std::iter::{Peekable};

type Result<T> = std::result::Result<T, <T as std::str::FromStr>::Err>;

pub fn strtol<T: FromStr>(chars: &mut Peekable<Chars>) -> Result<T> {
    let mut num = String::new();
    while let Some(ch) = chars.peek() {
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

    num.parse::<T>()
}
