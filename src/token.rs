// Todo: Symbolを追加する('(', ')', '{', '}'など)
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
