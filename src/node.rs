use crate::program::Var;

use crate::_type::Type;
use crate::_type::Type::{ Int, Ptr };

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
        cond: Box<ExprWrapper>,
        then: Box<Stmt>,
        els: Option<Box<Stmt>>,
    },
    While {
        cond: Box<Expr>,
        then: Box<Stmt>
    },
    For {
        init: Box<Option<ExprWrapper>>,
        cond: Box<Option<ExprWrapper>>,
        inc: Box<Option<ExprWrapper>>,
        then: Box<Stmt>,
    },
    Block {
        stmts: Vec<Stmt>,
    }
}

#[derive(PartialEq, Debug)]
pub struct ExprWrapper {
    ty: Type,
    expr: Box<Expr>
}

#[derive(PartialEq, Debug)]
pub enum Expr {
    Eq {
        lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    Neq {
        lhs: ExprWrapper,
        rhs: ExprWrapper
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
        lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    Sub {
        lhs: ExprWrapper,
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
    Var {
        var: Var
    },
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
    }
}

impl Expr {
    pub fn detect_type(&self) -> Type {
        match self {
            Expr::Eq { .. } => Int,
            Expr::Neq { .. } => Int,
            Expr::Gt { .. } => Int,
            Expr::Ge { .. } => Int,
            Expr::Lt { .. } => Int,
            Expr::Le { .. } => Int,
            Expr::Add { .. } => Int,
            Expr::Sub { .. } => Int,
            Expr::Mul { .. } => Int,
            Expr::Div { .. } => Int,
            Expr::Num { .. } => Int,
            Expr::Var { .. } => Int,
            Expr::PtrDiff { .. } => Int,
            Expr::FnCall { .. } => Int,
            Expr::PtrAdd { lhs, rhs } => {
                Expr::type_of_ptr_operation(lhs)
            },
            Expr::PtrSub { lhs, rhs } => {
                Expr::type_of_ptr_operation(lhs)
            },
            Expr::Assign { val, .. } => {
                Expr::type_of_ptr_operation(val)
            },
            Expr::Addr { operand } => {
                Ptr { base: Box::new(operand.ty) }
            },
            Expr::Deref { operand } => {
                let ty = operand.expr.detect_type();
                match ty {
                    Ptr { base } => {
                        *base.as_ref()
                    },
                    Int => Int
                }
            }
        }
    }

    fn type_of_ptr_operation(expr_wrapper: &ExprWrapper) -> Type {
        expr_wrapper.ty
    }
}
