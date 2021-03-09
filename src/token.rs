#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Reserved {
        op: String, // lenはop.len()で代用
    },
    Num {
        val: isize,
        t_str: String,
    },
    Ident {
        name: String,
    },
    Symbol(String),
    Str(Vec<u8>),
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
        let tk = self.tokens.get(self.pos);

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
}
