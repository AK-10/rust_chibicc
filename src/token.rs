use crate::tokenizer::loc::Loc;

pub mod token_type {
    use std::rc::Rc;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Reserved {
        pub op: Rc<String>,
        pub tk_str: Rc<String>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Num {
        pub val: isize,
        pub tk_str: Rc<String>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Ident {
        pub name: Rc<String>,
        pub tk_str: Rc<String>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Symbol {
        pub sym: Rc<String>,
        pub tk_str: Rc<String>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Str {
        pub bytes: Vec<u8>,
        pub tk_str: Rc<String>
    }
}

use token_type::{ Reserved, Num, Ident, Symbol, Str };
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Reserved(Reserved),
    Num(Num),
    Ident(Ident),
    Symbol(Symbol),
    Str(Str),
    Eof
}

impl TokenType {
    pub fn at_eof(&self) -> bool {
        match self {
            TokenType::Eof => true,
            _ => false
        }
    }

    pub fn tk_str(&self) -> Rc<String> {
        match self {
            TokenType::Reserved(reserved) => Rc::clone(&reserved.tk_str),
            TokenType::Num(num) => Rc::clone(&num.tk_str),
            TokenType::Ident(ident) => Rc::clone(&ident.tk_str),
            TokenType::Symbol(sym) => Rc::clone(&sym.tk_str),
            TokenType::Str(str_content) => Rc::clone(&str_content.tk_str),
            TokenType::Eof => panic!("Eof does not have tk_str")
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub loc: Loc
}

impl Token {
    pub fn new(token_type: TokenType, loc: Loc) -> Self {
        Token { token_type, loc }
    }

    pub fn error_message<'a>(&self, msg: &'a str) -> String {
        format!("{} {}", self.loc, msg)
    }
}

#[derive(Debug, Clone)]
pub struct TokenIter<'a> {
    tokens: &'a Vec<Token>, // TODO: 追加とかないのでsliceのほうが良いと思う
    pos: usize
}

pub enum TokenIterErr {
    OutOfRangeErr(String)
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = &'a Token;

    fn next(&mut self) -> Option<Self::Item> {
        let tk = self.tokens.get(self.current_position());

        if let Some(_) = tk {
            self.pos += 1;
        }

        tk
    }
}

impl<'a> TokenIter<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0
        }
    }

    pub fn back_to(&mut self, n: usize) -> Result<(), TokenIterErr> {
        let max_size = self.tokens.len() - 1;
        if n > max_size {
            let msg = format!("n must be 0..{}", max_size);
            return Err(TokenIterErr::OutOfRangeErr(msg))
        }

        self.pos = n;

        Ok(())
    }

    pub fn current_position(&self) -> usize {
        self.pos
    }

    pub fn peek(&self) -> Option<&'a Token> {
        self.tokens.get(self.current_position())
    }
}
