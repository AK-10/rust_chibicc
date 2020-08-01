// EBNF
// expr := mul ("+" mul | "-" mul)*
// mul := primary ("+" primary | "-" primary)*
// primary := num | "(" expr ")"

#[derive(PartialEq, Clone)]
pub enum Node {
    Add {
        lhs: Box<Node>,
        rhs: Box<Node>
    },
    Sub {
        lhs: Box<Node>,
        rhs: Box<Node>
    },
    Mul {
        lhs: Box<Node>,
        rhs: Box<Node>
    },
    Div {
        lhs: Box<Node>,
        rhs: Box<Node>
    },
    Num {
        val: isize
    }
}
