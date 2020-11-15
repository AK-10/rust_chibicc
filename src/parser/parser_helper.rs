use crate::parser::Parser;
use crate::node::{ Stmt, ExprWrapper };
use crate::token::Token;
use crate::program::{ Var };

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

        self.expect_next("(".to_string())?;

        // 初期化，条件，処理後はない場合がある
        let init = self.assign().ok();
        self.expect_next(";".to_string())?;

        let cond = self.expr().ok();
        self.expect_next(";".to_string())?;

        let inc = self.assign().ok();
        self.expect_next(")".to_string())?;

        let then = self.stmt()?;

        Ok(Stmt::For {
            init: init.map(|x| ExprWrapper::new(x)),
            cond: cond.map(|x| ExprWrapper::new(x)),
            inc: inc.map(|x| ExprWrapper::new(x)),
            then: Box::new(then)
        })
    }

    pub(in super) fn expr_stmt(&mut self) -> Result<Stmt, String> {
        Ok(Stmt::ExprStmt { val: ExprWrapper::new(self.expr()?) })
    }

    pub(in super) fn expect_next(&mut self, word: String) -> Result<(), String> {
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
        while let Ok(_) = self.expect_next(",".to_string()) {
            args.push(ExprWrapper::new(self.expr()?));
        }

        Ok(args)
    }

    // 関数呼び出しにおける引数をparseする
    // params := ident ("," ident)*
    pub(in super) fn parse_func_params(&mut self) -> Result<Vec<Var> ,String> {
        self.expect_next("(".to_string())?;

        let mut params = Vec::<Var>::new();
        if self.expect_next(")".to_string()).is_ok() {
            return Ok(params)
        }

        let first_args = self.peekable.peek();
        if let Some(Token::Ident { name }) = first_args {
            self.peekable.next();

            let offset = (params.len() + 1) * 8;
            params.push(Var { name: name.clone(), offset });
        } else {
            return Err("token not found".to_string())
        }

        while let Ok(_) = self.expect_next(",".to_string()) {
            match self.peekable.peek() {
                Some(Token::Ident { name }) => {
                    self.peekable.next();
                    let offset = (params.len() + 1) * 8;

                    params.push(Var { name: name.clone(), offset: offset })
                },
                Some(token) => {
                    return Err(format!("expect ident, result: {:?}", token))
                }
                _ => {
                    return Err("token not found".to_string())
                }
            }
        }

        self.expect_next(")".to_string())?;

        Ok(params)
    }
}
