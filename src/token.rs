pub mod token_type {
    use std::rc::Rc;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Reserved {
        op: Rc<String>,
        tk_str: Rc<String>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Num {
        val: isize,
        tk_str: Rc<String>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Ident {
        name: Rc<String>,
        tk_str: Rc<String>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Symbol {
        sym: Rc<String>,
        tk_str: Rc<String>
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Str {
        bytes: Vec<u8>,
        tk_str: Rc<String>
    }
}

use token_type::{ Reserved, Num, Ident, Symbol, Str };

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
