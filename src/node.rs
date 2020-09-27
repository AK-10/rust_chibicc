use crate::program::Var;

// EBNF
// program := stmt*
// stmt := expr ";"
//       | "return" expr ";"
//       | "{" stmt* "}"
//       | "if" "(" expr ")" stmt ("else" stmt)? /* ( expr ) is primary. */
//       | "while" "(" expr ")" stmt
//       | "for" "(" expr? ";" expr? ";" expr? ")" stmt
//       | 
// expr := assign
// assign := equality ("=" assign)?
// equality := relational ("==" relational | "!=" relational)*
// relational := add ("<" add | "<=" add | ">" add | ">=" add)*
// add := mul ("+" mul | "-" mul)*
// mul := unary ("*" unary | "/" unary)*
// unary := ("+" | "-")? primary
// primary := num
//          | ident args? // 単なる変数か，関数呼び出し
//          | "(" expr ")"
// args := "(" ")"

#[derive(PartialEq, Debug)]
pub enum Stmt {
    Return {
        val: Expr
    },
    ExprStmt {
        val: Expr
    },
    If {
        cond: Box<Expr>,
        then: Box<Stmt>,
        els: Option<Box<Stmt>>,
    },
    While {
        cond: Box<Expr>,
        then: Box<Stmt>
    },
    For {
        init: Box<Option<Expr>>,
        cond: Box<Option<Expr>>,
        inc: Box<Option<Expr>>,
        then: Box<Stmt>,
    },
    Block {
        stmts: Vec<Stmt>,
    }
}

#[derive(PartialEq, Debug)]
pub enum Expr {
    Eq {
        lhs: Box<Expr>,
        rhs: Box<Expr>
    },
    Neq {
        lhs: Box<Expr>,
        rhs: Box<Expr>
    },
    Gt {
        lhs: Box<Expr>,
        rhs: Box<Expr>
    },
    Ge {
        lhs: Box<Expr>,
        rhs: Box<Expr>
    },
    Lt {
        lhs: Box<Expr>,
        rhs: Box<Expr>
    },
    Le {
        lhs: Box<Expr>,
        rhs: Box<Expr>
    },
    Add {
        lhs: Box<Expr>,
        rhs: Box<Expr>
    },
    Sub {
        lhs: Box<Expr>,
        rhs: Box<Expr>
    },
    Mul {
        lhs: Box<Expr>,
        rhs: Box<Expr>
    },
    Div {
        lhs: Box<Expr>,
        rhs: Box<Expr>
    },
    Num {
        val: isize
    },
    Var {
        var: Var
    },
    Assign {
        var: Var,
        val: Box<Expr>
    },
    FnCall {
        fn_name: String,
    }
}

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
        offset: usize // offset from RBP(ベースポインタ)
    },
    If {
        cond: Box<Node>,
        then: Box<Node>,
        els: Option<Box<Node>>,
    },
    While {
        cond: Box<Node>,
        then: Box<Node>
    },
    For {
        init: Box<Option<Node>>,
        cond: Box<Option<Node>>,
        inc: Box<Option<Node>>,
        then: Box<Node>,
    },
    Block {
        stmts: Vec<Node>,
    }
}
