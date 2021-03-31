use crate::program::Var;
use crate::_type::{ Type, Member };
use crate::_type::Type::{ Int, Ptr };

use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;

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
        init: Box<Option<Stmt>>, // only ExprStmt
        cond: Option<ExprWrapper>,
        inc: Box<Option<Stmt>>, // only ExprStmt
        then: Box<Stmt>,
    },
    Block {
        stmts: Vec<Stmt>,
    },
    PureExpr(ExprWrapper) // StmtExprの返り値のために作った
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
        fn_name: Rc<String>,
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
    StmtExpr(Vec<Stmt>), // GNU C extension Null
    Member(ExprWrapper, Member) // struct member
}

impl Expr {
    pub fn detect_type(&self) -> Rc<Type> {
        match self {
            Expr::Eq { .. }
            | Expr::Neq { .. }
            | Expr::Gt { .. }
            | Expr::Ge { .. }
            | Expr::Lt { .. }
            | Expr::Le { .. }
            | Expr::Add { .. }
            | Expr::Sub { .. }
            | Expr::Mul { .. }
            | Expr::Div { .. }
            | Expr::Num { .. }
            | Expr::PtrDiff { .. }
            | Expr::FnCall { .. } => Rc::new(Int),
            Expr::PtrAdd { lhs, rhs: _ }
            | Expr::PtrSub { lhs, rhs: _ } => Rc::clone(&lhs.ty),
            Expr::Assign { var, .. } => {
                Rc::clone(&var.ty)
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
                   Type::Ptr { base }
                   | Type::Array { base, .. } => Rc::clone(base),
                   _ => Rc::clone(&operand.ty)
                }
            },
            Expr::Var(var) => {
                Rc::clone(&var.borrow().ty)
            },
            Expr::Null => Rc::new(Int),
            Expr::StmtExpr(stmts) => { // stmt.lastはPureExprのはず
                match stmts.last() {
                    Some(Stmt::PureExpr(expr)) => Rc::clone(&expr.ty),
                    _ => unreachable!("stmts.last can only be expr_stmt")
                }
            },
            Expr::Member(_, member) => {
                Rc::clone(&member.ty)
            }
        }
    }

    pub fn to_expr_wrapper(&self) -> ExprWrapper {
        // TODO: cloneやめたい
        // ExprWrapper.exprを&'a Exprにする?
        // もしくはRc
        ExprWrapper::new(self.clone())
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Expr::Eq { .. } => write!(f, "Eq"),
            Expr::Neq { .. } => write!(f, "Neq"),
            Expr::Gt { .. } => write!(f, "Gt"),
            Expr::Ge { .. } => write!(f, "Ge"),
            Expr::Lt { .. } => write!(f, "Lt"),
            Expr::Le { .. } => write!(f, "Le"),
            Expr::Add { .. } => write!(f, "Add"),
            Expr::Sub { .. } => write!(f, "Sub"),
            Expr::Mul { .. } => write!(f, "Mul"),
            Expr::Div { .. } => write!(f, "Div"),
            Expr::Num { .. } => write!(f, "Num"),
            Expr::Var(_) => write!(f, "Var"),
            Expr::Assign { .. } => write!(f, "Assign"),
            Expr::FnCall { .. } => write!(f, "FnCall"),
            Expr::Addr { .. } => write!(f, "Addr"),
            Expr::Deref { .. } => write!(f, "Deref"),
            Expr::PtrAdd { .. } => write!(f, "PtrAdd"),
            Expr::PtrSub { .. } => write!(f, "PtrSub"),
            Expr::PtrDiff { .. } => write!(f, "PtrDiff"),
            Expr::StmtExpr(_) => write!(f, "StmtExpr"),
            Expr::Null => write!(f, "Null"),
            Expr::Member(_, _) =>  write!(f, "Member")
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Stmt::Return { .. } => write!(f, "Return"),
            Stmt::ExprStmt { .. } => write!(f, "ExprStmt"),
            Stmt::If { .. } => write!(f, "If"),
            Stmt::While { .. } => write!(f, "While"),
            Stmt::For { .. } => write!(f, "For"),
            Stmt::Block { .. } => write!(f, "Block"),
            Stmt::PureExpr { .. } => write!(f, "PureExpr"),
        }
    }
}
