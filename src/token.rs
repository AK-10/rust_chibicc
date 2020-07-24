#[derive(Debug, Clone)]
pub enum Token {
    Reserved {
        op: char,
        t_str: String,
        next: Box<Option<Token>>
    },
    Num{
        val: usize,
        t_str: String,
        next: Box<Option<Token>>
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
