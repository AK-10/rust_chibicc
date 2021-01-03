use crate::parser::Parser;
use crate::node::{ Stmt, ExprWrapper, Expr };
use crate::token::Token;
use crate::program::Var;
use crate::_type::Type;

impl<'a> Parser<'_> {
    pub(in super) fn find_lvar(&self, name: &String) -> Option<&Var> {
        self.locals.iter().find(|item| { item.name == *name })
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
    // declaration := basetype ident ("=" expr) ";"
    pub(in super) fn declaration(&mut self) -> Result<Stmt, String> {
        let ty = self.base_type()?;
        match self.peekable.peek() {
            Some(Token::Ident { name }) => {
                self.peekable.next();
                let expr =
                    if let Err(_) = self.expect_next_reserved("=".to_string()) {
                        // int a; みたいな場合はローカル変数への追加だけ行う. (push rax, 3 みたいなのはしない)
                        let var = self.new_var(name, &ty);
                        self.locals.push(var);
                        self.expect_next_symbol(";".to_string())?;

                        Expr::Null
                    } else {
                        let lhs = self.new_var(name, &ty);
                        self.locals.push(lhs.clone());

                        let rhs = self.expr()?;
                        self.expect_next_symbol(";".to_string())?;

                        Expr::Assign{ var: Expr::Var(lhs).to_expr_wrapper(), val: rhs.to_expr_wrapper() }
                    };

                Ok(Stmt::ExprStmt { val: ExprWrapper { ty, expr: Box::new(expr) } })
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
    pub(in super) fn parse_func_params(&mut self) -> Result<Vec<Var>, String> {
        self.expect_next_symbol("(".to_string())?;

        let mut params = Vec::<Var>::new();
        if self.expect_next_symbol(")".to_string()).is_ok() {
            return Ok(params)
        }
        let ty = self.base_type()?;
        let first_arg = self.peekable.peek();

        if let Some(Token::Ident { name }) = first_arg {
            self.peekable.next();

            let offset = (params.len() + 1) * 8;
            params.push(Var { name: name.clone(), offset, ty });
        } else {
            return Err("token not found".to_string())
        }

        while let Ok(_) = self.expect_next_symbol(",".to_string()) {
            let ty = self.base_type()?;
            match self.peekable.peek() {
                Some(Token::Ident { name }) => {
                    self.peekable.next();

                    let offset = (params.len() + 1) * 8;
                    params.push(Var { name: name.clone(), offset, ty });
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

    pub(in super) fn base_type(&mut self) -> Result<Type, String> {
        self.expect_next_reserved("int".to_string())?;

        let ty = Type::Int;

        // ポインタ型はネストしても一つとしてみなす
        match self.peekable.peek() {
            Some(Token::Reserved { op }) if op == "*" => {
                self.consume_pointer();
                Ok(Type::Ptr { base: Box::new(ty) })
            }
            _ => {
                Ok(ty)
            }
        }
    }

    pub(in super) fn consume_pointer(&mut self) {
        while let Some(Token::Reserved { op }) = self.peekable.peek() {
            if op == "*" { self.peekable.next(); } else { break }
        }
    }

    pub(in super) fn new_var(&self, name: &String, ty: &Type) -> Var {
        let offset = (self.locals.len() + 1) * 8;

        Var { name: name.to_string(), offset, ty: ty.clone() }
    }
}
