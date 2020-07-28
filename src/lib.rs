pub mod token;
pub mod lexer;

use std::str::{Chars, FromStr};
use std::iter::{Peekable};

type Result<T> = std::result::Result<T, <T as std::str::FromStr>::Err>;


