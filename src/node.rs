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
// unary := "+"? primary
//        | "-"? primary
//        | "*"? unary
//        | "&"? unary
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
        var: Box<Expr>, // x = 10, *y = 100とかあるので今のところexprにするしかない
        val: Box<Expr>
    },
    FnCall {
        fn_name: String,
        args: Vec<Expr>
    },
    Addr {
        operand: Box<Expr>
    },
    Deref {
        operand: Box<Expr>
    }
}
