use crate::program::Var;
use crate::_type::{ Type, Member };
use crate::_type::Type::{ Int, Ptr };

use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;

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
    PureExpr(ExprWrapper), // StmtExprの返り値のために作った
    Break,
    Continue
}

#[derive(PartialEq, Debug, Clone)]
pub struct ExprWrapper {
    pub ty: Box<Type>,
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
    BitAnd {
        lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    BitOr {
        lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    BitXor {
        lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    Num {
        val: isize
    },
    Cast(Box<Type>, ExprWrapper),
    Var(Rc<RefCell<Var>>),
    Assign {
        var: ExprWrapper, // x = 10, *y = 100とかあるので今のところexprにするしかない
        val: ExprWrapper
    },
    PreInc(ExprWrapper),
    PreDec(ExprWrapper),
    PostInc(ExprWrapper),
    PostDec(ExprWrapper),
    AddEq {
        var: ExprWrapper,
        val: ExprWrapper
    },
    PtrAddEq {
        var: ExprWrapper,
        val: ExprWrapper
    },
    SubEq {
        var: ExprWrapper,
        val: ExprWrapper
    },
    PtrSubEq {
        var: ExprWrapper,
        val: ExprWrapper
    },
    MulEq {
        var: ExprWrapper,
        val: ExprWrapper
    },
    DivEq {
        var: ExprWrapper,
        val: ExprWrapper
    },
    Comma {
        lhs: Stmt,
        rhs: ExprWrapper
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
    Not(ExprWrapper),
    BitNot(ExprWrapper),
    LogAnd {
        lhs: ExprWrapper,
        rhs: ExprWrapper
    },
    LogOr {
        lhs: ExprWrapper,
        rhs: ExprWrapper
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
    pub fn detect_type(&self) -> Box<Type> {
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
            | Expr::FnCall { .. }
            | Expr::Not(_)
            | Expr::BitAnd { .. }
            | Expr::BitOr { .. }
            | Expr::BitXor { .. }
            | Expr::LogAnd { .. }
            | Expr::LogOr { .. } => Box::new(Type::Long),
            Expr::Cast(ty, ..) => Box::clone(ty),
            Expr::PtrAdd { lhs, rhs: _ }
            | Expr::PtrSub { lhs, rhs: _ } => Box::clone(&lhs.ty),
            Expr::Assign { var, .. } => {
                Box::clone(&var.ty)
            },
            Expr::PreInc(expr_wrapper) => Box::clone(&expr_wrapper.ty),
            Expr::PreDec(expr_wrapper) => Box::clone(&expr_wrapper.ty),
            Expr::PostInc(expr_wrapper) => Box::clone(&expr_wrapper.ty),
            Expr::PostDec(expr_wrapper) => Box::clone(&expr_wrapper.ty),
            Expr::AddEq { var, .. } => {
                Box::clone(&var.ty)
            },
            Expr::PtrAddEq { var, .. } => {
                Box::clone(&var.ty)
            },
            Expr::SubEq { var, .. } => {
                Box::clone(&var.ty)
            }
            Expr::PtrSubEq { var, .. } => {
                Box::clone(&var.ty)
            },
            Expr::MulEq { var, .. } => {
                Box::clone(&var.ty)
            },
            Expr::DivEq { var, .. } => {
                Box::clone(&var.ty)
            },
            Expr::Comma { rhs, .. } => {
                Box::clone(&rhs.ty)
            },
            Expr::Addr { operand } => {
                let ty = operand.ty.as_ref();
                match ty {
                    Type::Array { base, .. } => Box::clone(base),
                    _ => Box::new(Ptr { base: Box::clone(&operand.ty) })
                }
            },
            Expr::Deref { operand } => {
                match operand.ty.as_ref() {
                   Type::Ptr { base }
                   | Type::Array { base, .. } => Box::clone(base),
                   Type::Void => panic!("derefierencing a void pointer"),
                   _ => Box::clone(&operand.ty)
                }
            },
            Expr::BitNot(target) => Box::clone(&target.ty),
            Expr::Var(var) => {
                Box::clone(&var.borrow().ty)
            },
            Expr::Null => Box::new(Int),
            Expr::StmtExpr(stmts) => { // stmt.lastはPureExprのはず
                match stmts.last() {
                    Some(Stmt::PureExpr(expr)) => Box::clone(&expr.ty),
                    _ => unreachable!("stmts.last can only be expr_stmt")
                }
            },
            Expr::Member(_, member) => {
                Box::clone(&member.ty)
            }
        }
    }

    pub fn to_expr_wrapper(self) -> ExprWrapper {
        ExprWrapper::new(self)
    }

    pub fn is_lvalue(&self) -> bool {
        match self {
            Expr::Deref { .. }
            | Expr::Var(_)
            | Expr::Member(_, _) => true,
            _ => false
        }
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
            Expr::BitAnd { .. } => write!(f, "BitAnd"),
            Expr::BitOr { .. } => write!(f, "BitOr"),
            Expr::BitXor { .. } => write!(f, "BitXor"),
            Expr::Num { .. } => write!(f, "Num"),
            Expr::Cast { .. } => write!(f, "Cast"),
            Expr::Var(_) => write!(f, "Var"),
            Expr::Assign { .. } => write!(f, "Assign"),
            Expr::PreInc(_) => write!(f, "PreInc"),
            Expr::PreDec(_) => write!(f, "PreDec"),
            Expr::PostInc(_) => write!(f, "PostInc"),
            Expr::PostDec(_) => write!(f, "PostDec"),
            Expr::AddEq { .. } => write!(f, "PostDec"),
            Expr::PtrAddEq { .. } => write!(f, "PostDec"),
            Expr::SubEq { .. } => write!(f, "PostDec"),
            Expr::PtrSubEq { .. } => write!(f, "PostDec"),
            Expr::MulEq { .. } => write!(f, "PostDec"),
            Expr::DivEq { .. } => write!(f, "PostDec"),
            Expr::Comma { .. } => write!(f, "Comma"),
            Expr::FnCall { .. } => write!(f, "FnCall"),
            Expr::Addr { .. } => write!(f, "Addr"),
            Expr::Deref { .. } => write!(f, "Deref"),
            Expr::Not { .. } => write!(f, "Not"),
            Expr::BitNot { .. } => write!(f, "BitNot"),
            Expr::LogAnd { .. } => write!(f, "LogAnd"),
            Expr::LogOr { .. } => write!(f, "LogOr"),
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
            Stmt::Break { .. } => write!(f, "Break"),
            Stmt::Continue { .. } => write!(f, "Continue"),
        }
    }
}
