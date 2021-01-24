use crate::parser::Parser;
use crate::node::{ Stmt, ExprWrapper, Expr };
use crate::token::Token;
use crate::program::{ Var, Offset };
use crate::_type::Type;

use std::rc::Rc;
use std::cell::RefCell;


impl<'a> Parser<'a> {
    pub(in super) fn find_lvar(&self, name: &String) -> Option<Rc<RefCell<Var>>> {
        self.locals.iter()
            .find(|var| { var.borrow().name == *name })
            .map(|var| Rc::clone(var)) // &Rc<RefCell<Var>> -> Rc<RefCell<Var>>にする
    }

    pub(in super) fn if_stmt(&mut self) -> Result<Stmt, String> {
        self.peekable.next();

        // primaryだと()なしでも動くようになるが, Cコンパイラではなくなる
        let cond = self.primary()?;
        let then = self.stmt()?;
        let els = match self.peekable.peek() {
            Some(Token::Reserved { op }) if *op == "else" => {
                self.peekable.next();

                Some(self.stmt()?)
            },
            _ => None
        };

        Ok(Stmt::If {
            cond: ExprWrapper::new(cond),
            then: Box::new(then),
            els: els.map(|x| Box::new(x)),
        })
    }

    pub(in super) fn while_stmt(&mut self) -> Result<Stmt, String> {
        self.peekable.next();

        let cond = self.primary()?;
        let then = self.stmt()?;

        Ok(Stmt::While {
            cond: ExprWrapper::new(cond),
            then: Box::new(then)
        })
    }

    pub(in super) fn for_stmt(&mut self) -> Result<Stmt, String> {
        self.peekable.next();

        self.expect_next_symbol("(".to_string())?;

        // 初期化，条件，処理後はない場合がある
        let init = self.assign().ok();
        self.expect_next_symbol(";".to_string())?;

        let cond = self.expr().ok();
        self.expect_next_symbol(";".to_string())?;

        let inc = self.assign().ok();
        self.expect_next_symbol(")".to_string())?;

        let then = self.stmt()?;

        Ok(Stmt::For {
            init: init.map(|x| ExprWrapper::new(x)),
            cond: cond.map(|x| ExprWrapper::new(x)),
            inc: inc.map(|x| ExprWrapper::new(x)),
            then: Box::new(then)
        })
    }

    // variable declaration
    // declaration := basetype ident ("[" num "]")* ("=" expr) ";"
    pub(in super) fn declaration(&mut self) -> Result<Stmt, String> {
        let ty = self.base_type()?;

        match self.peekable.peek() {
            Some(Token::Ident { name }) => {
                self.peekable.next();
                let ty = self.read_type_suffix(ty)?;

                let expr =
                    if let Err(_) = self.expect_next_reserved("=".to_string()) {
                        // int a; みたいな場合はローカル変数への追加だけ行う. (push rax, 3 みたいなのはしない)
                        let var = self.new_var(name, Rc::clone(&ty));
                        self.locals.push(var);
                        self.expect_next_symbol(";".to_string())?;

                        Expr::Null
                    } else {
                        let lhs = self.new_var(name, Rc::clone(&ty));
                        self.locals.push(Rc::clone(&lhs));

                        let rhs = self.expr()?;
                        self.expect_next_symbol(";".to_string())?;

                        Expr::Assign { var: Expr::Var(Rc::clone(&lhs)).to_expr_wrapper(), val: rhs.to_expr_wrapper() }
                    };

                Ok(Stmt::ExprStmt { val: ExprWrapper { ty: Rc::clone(&ty), expr: Box::new(expr) } })
            }
            _ => {
                return Err("expect ident, but not found".to_string())
            }
        }
    }

    pub(in super) fn expr_stmt(&mut self) -> Result<Stmt, String> {
        Ok(Stmt::ExprStmt { val: ExprWrapper::new(self.expr()?) })
    }

    pub(in super) fn expect_next_symbol(&mut self, word: String) -> Result<(), String> {
        let tk = self.peekable.peek();

        match tk {
            Some(Token::Symbol(op)) if *op == word => {
                self.peekable.next();
                Ok(())
            },
            _ => {
                let msg = format!("expect {}, but different found", word);
                Err(msg)
            }
        }
    }

    pub(in super) fn expect_next_reserved(&mut self, word: String) -> Result<(), String> {
        let tk = self.peekable.peek();

        match tk {
            Some(Token::Reserved { op }) if *op == word => {
                self.peekable.next();
                Ok(())
            },
            _ => {
                let msg = format!("expect {}, but different found", word);
                Err(msg)
            }
        }
    }

    // 関数呼び出しにおける引数をparseする
    pub(in super) fn parse_args(&mut self) -> Result<Vec<ExprWrapper>, String> {
        // 最初の一個だけ読んでおく
        let mut args = vec![ExprWrapper::new(self.expr()?)];
        while let Ok(_) = self.expect_next_symbol(",".to_string()) {
            args.push(ExprWrapper::new(self.expr()?));
        }

        Ok(args)
    }

    // 関数宣言における引数をparseする
    // params := ident ("," ident)*
    pub(in super) fn parse_func_params(&mut self) -> Result<Vec<Rc<RefCell<Var>>>, String> {
        self.expect_next_symbol("(".to_string())?;

        let mut params = Vec::<Rc<RefCell<Var>>>::new();
        if self.expect_next_symbol(")".to_string()).is_ok() {
            return Ok(params)
        }
        let ty = self.base_type()?;
        let first_arg = self.peekable.peek();

        if let Some(Token::Ident { name }) = first_arg {
            self.peekable.next();

            params.push(self.new_var(name, ty));
        } else {
            return Err("token not found".to_string())
        }

        while let Ok(_) = self.expect_next_symbol(",".to_string()) {
            let ty = self.base_type()?;
            match self.peekable.peek() {
                Some(Token::Ident { name }) => {
                    self.peekable.next();

                    params.push(self.new_var(name, ty));
                },
                Some(token) => {
                    return Err(format!("expect ident, result: {:?}", token))
                }
                _ => {
                    return Err("token not found".to_string())
                }
            }
        }

        self.expect_next_symbol(")".to_string())?;

        Ok(params)
    }

    pub(in super) fn base_type(&mut self) -> Result<Rc<Type>, String> {
        self.expect_next_reserved("int".to_string())?;
        let mut ty = Type::Int;

        while let Some(Token::Reserved { op }) = self.peekable.peek() {
            if op == "*" {
                ty = Type::Ptr { base: Rc::new(ty) };
                self.peekable.next();
            } else {
                break
            }
        }

        Ok(Rc::new(ty))
    }

    pub(in super) fn new_var(&self, name: &String, ty: Rc<Type>) -> Rc<RefCell<Var>> {
        Rc::new(
            RefCell::new(
                Var {
                    name: name.to_string(),
                    offset: Offset::Unset,
                    ty: Rc::clone(&ty)
                }
            )
        )
    }

    pub(in super) fn read_type_suffix(&mut self, base: Rc<Type>) -> Result<Rc<Type>, String> {
        match self.expect_next_symbol("[".to_string()) {
            Ok(_) => {
                match self.peekable.next() {
                    Some(Token::Num { val, .. }) => {
                        if let Err(e) = self.expect_next_symbol("]".to_string()) {
                            Err(e)
                        } else {
                            let nested_base = self.read_type_suffix(base)?;
                            Ok(Rc::new(Type::Array { base: nested_base, len: *val as usize }))
                        }
                    },
                    _ => {
                        Err("expect num after [".to_string())
                    }
                }
            }
            Err(_) => Ok(base)
        }
    }

    pub(in super) fn new_add(lhs: ExprWrapper, rhs: ExprWrapper) -> Result<Expr, String> {
        match (lhs.ty.as_ref(), rhs.ty.as_ref()) {
            (Type::Int, Type::Int) => {
                Ok(Expr::Add { lhs, rhs })
            },
            (Type::Ptr { .. }, Type::Int) => {
                Ok(Expr::PtrAdd { lhs, rhs })
            },
            (Type::Array { .. }, Type::Int) => {
                Ok(Expr::PtrAdd { lhs, rhs })
            },
            (Type::Int, Type::Ptr { .. }) => {
                Ok(Expr::PtrAdd { lhs: rhs, rhs: lhs })
            },
            (Type::Int, Type::Array { .. }) => {
                Ok(Expr::PtrAdd { lhs: rhs, rhs: lhs })
            }
            (_, _) => {
                return Err("invalid operands at +".to_string());
            }
        }
    }

    pub(in super) fn new_sub(lhs: ExprWrapper, rhs: ExprWrapper) -> Result<Expr, String> {
       match (lhs.ty.as_ref(), rhs.ty.as_ref()) {
            (Type::Int, Type::Int) => {
                Ok(Expr::Sub { lhs, rhs })
            },
            (Type::Ptr { .. }, Type::Int) => {
                Ok(Expr::PtrSub { lhs, rhs })
            },
            (Type::Array { .. }, Type::Int) => {
                Ok(Expr::PtrSub { lhs, rhs })
            },
            (Type::Ptr { .. }, Type::Ptr { .. }) => {
                Ok(Expr::PtrDiff { lhs, rhs })
            },
            (_, _) => {
                return Err("invalid operands at -".to_string());
            }
        }
    }
}
