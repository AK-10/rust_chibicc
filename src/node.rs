use crate::program::Var;
use crate::_type::Type;
use crate::_type::Type::{ Int, Ptr };

use std::rc::Rc;
use std::cell::RefCell;


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

#[derive(PartialEq, Debug, Clone)]
pub enum Stmt {
    Return {
        val: ExprWrapper
    },
    ExprStmt {
        val: ExprWrapper
    },
    If {
        cond: ExprWrapper,
        then: Box<Stmt>,
        els: Option<Box<Stmt>>,
    },
    While {
        cond: ExprWrapper,
        then: Box<Stmt>
    },
    For {
        init: Option<ExprWrapper>,
        cond: Option<ExprWrapper>,
        inc: Option<ExprWrapper>,
        then: Box<Stmt>,
    },
    Block {
        stmts: Vec<Stmt>,
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct ExprWrapper {
    pub ty: Rc<Type>,
    pub expr: Box<Expr>
}

impl ExprWrapper {
    pub fn new(expr: Expr) -> Self {
        Self {
            ty: expr.detect_type(),
            expr: Box::new(expr)
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
    Eq {
        lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    Neq {
        lhs: ExprWrapper, rhs: ExprWrapper
    },
    Gt {
        lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    Ge {
        lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    Lt {
        lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    Le {
        lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    Add {
        lhs: ExprWrapper, rhs: ExprWrapper }, Sub { lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    Mul {
        lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    Div {
        lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    Num {
        val: isize
    },
    Var(Rc<RefCell<Var>>),
    Assign {
        var: ExprWrapper, // x = 10, *y = 100とかあるので今のところexprにするしかない
        val: ExprWrapper
    },
    FnCall {
        fn_name: String,
        args: Vec<ExprWrapper>
    },
    Addr {
        operand: ExprWrapper
    },
    Deref {
        operand: ExprWrapper
    },
    PtrAdd { // ptr + num or num + ptr
        lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    PtrSub { // ptr - num or num - ptr
        lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    PtrDiff { // ptr - ptr
        lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    Null,
    StmtExpr(Vec<Stmt>) // GNU C extension Null
}

impl Expr {
    pub fn detect_type(&self) -> Rc<Type> {
        match self {
            Expr::Eq { .. } => Rc::new(Int),
            Expr::Neq { .. } => Rc::new(Int),
            Expr::Gt { .. } => Rc::new(Int),
            Expr::Ge { .. } => Rc::new(Int),
            Expr::Lt { .. } => Rc::new(Int),
            Expr::Le { .. } => Rc::new(Int),
            Expr::Add { .. } => Rc::new(Int),
            Expr::Sub { .. } => Rc::new(Int),
            Expr::Mul { .. } => Rc::new(Int),
            Expr::Div { .. } => Rc::new(Int),
            Expr::Num { .. } => Rc::new(Int),
            Expr::PtrDiff { .. } => Rc::new(Int),
            Expr::FnCall { .. } => Rc::new(Int),
            Expr::PtrAdd { lhs, rhs: _ } => { Rc::clone(&lhs.ty)
            },
            Expr::PtrSub { lhs, rhs: _ } => {
                Rc::clone(&lhs.ty)
            },
            Expr::Assign { val, .. } => {
                Rc::clone(&val.ty)
            },
            Expr::Addr { operand } => {
                let ty = operand.ty.as_ref();
                match ty {
                    Type::Array { base, .. } => Rc::clone(base),
                    _ => Rc::new(Ptr { base: Rc::clone(&operand.ty) })
                }
            },
            Expr::Deref { operand } => {
                 match operand.ty.as_ref() {
                    Ptr { base } => {
                        Rc::clone(base) },
                    Type::Array { base, .. } => {
                        Rc::clone(base)
                    }
                    _ => panic!("can not deref value")
                }
            },
            Expr::Var(var) => {
                Rc::clone(&var.borrow().ty)
            },
            Expr::Null => Rc::new(Int),
            Expr::StmtExpr(stmts) => { // stmt.lastはexpr_stmtのはず
                match stmts.last() {
                    Some(Stmt::ExprStmt { val }) => Rc::clone(&val.ty),
                    _ => unreachable!("stmts.last can only be expr_stmt")
                }
            }
        }
    }

    pub fn to_expr_wrapper(&self) -> ExprWrapper {
        ExprWrapper::new(self.clone())
    }
}
