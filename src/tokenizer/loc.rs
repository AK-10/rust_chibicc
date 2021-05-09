use std::fmt;
use std::fmt::Display;

// token location
#[derive(Debug, Clone, PartialEq)]
pub struct Loc {
    pub row: usize,
    pub col: usize
}

impl Loc {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

impl Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}:{}:", self.row, self.col)
    }
}
