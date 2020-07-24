#[derive(Debug, Clone)]
pub enum Token {
    Reserved {
        op: char,
        t_str: String,
    },
    Num{
        val: usize,
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
