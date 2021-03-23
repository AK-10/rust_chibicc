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

// TODO: new_type パターンに置き換えたい
// op, nameなどのアクセスがかなりめんどくさい
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Reserved(Reserved),
    Num(Num),
    Ident(Ident),
    Symbol(Symbol),
    Str(Str),
    Eof
}

impl Token {
    pub fn at_eof(self) -> bool {
        match self {
            Token::Eof => true,
            _ => false
        }
    }

    pub fn tk_str(&self) -> Rc<String> {
        match self {
            Token::Reserved(reserved) => reserved.tk_str,
            Token::Num(num) => num.tk_str,
            Token::Ident(ident) => ident.tk_str,
            Token::Symbol(sym) => sym.tk_str,
            Token::Str(str_content) => str_content.tk_str,
            Token::Eof => panic!("Eof does not have tk_str")
        }
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
