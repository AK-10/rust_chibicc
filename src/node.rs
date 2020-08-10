// 四則演算のEBNF
// expr := mul ("+" mul | "-" mul)*
// mul := primary ("*" primary | "/" primary)*
// unary := ("+" | "-")? primary
// primary := num | "(" expr ")"

#[derive(PartialEq, Clone, Debug)]
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
