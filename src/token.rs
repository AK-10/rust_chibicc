#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Reserved {
        op: String,
    },
    Num {
        val: isize,
        t_str: String,
    },
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
