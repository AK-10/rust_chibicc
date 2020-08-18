// EBNF
// program := stmt*
// stmt := expr ";" | "return" expr ";"
// expr := assign
// assign := equality ("=" assign)?  a=b=1のようなものを許す
// equality := relational ("==" relational | "!=" relational)*
// relational := add ("<" add | "<=" add | ">" add | ">=" add)*
// add := mul ("+" mul | "-" mul)*
// mul := unary ("*" unary | "/" unary)*
// unary := ("+" | "-")? primary
// primary := num | "(" expr ")"

#[derive(PartialEq, Clone, Debug)]
pub enum Node {
    Eq {
        lhs: Box<Node>,
        rhs: Box<Node>
    },
    Neq {
        lhs: Box<Node>,
        rhs: Box<Node>
    },
    Gt {
        lhs: Box<Node>,
        rhs: Box<Node>
    },
    Ge {
        lhs: Box<Node>,
        rhs: Box<Node>
    },
    Lt {
        lhs: Box<Node>,
        rhs: Box<Node>
    },
    Le {
        lhs: Box<Node>,
        rhs: Box<Node>
    },
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
    },
    Return {
        val: Box<Node>
    },
    ExprStmt {
        val: Box<Node>
    },
    Assign {
        var: Box<Node>, // Lvarしか入れたくない
        val: Box<Node> // Exprしか入れたくない
    },
    Var {
        name: String,
        offset: i64
    }
}
