use crate::parser::Parser;
use crate::node::{ Stmt, Expr };
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
            cond: Box::new(cond),
            then: Box::new(then),
            els: els.map(|x| Box::new(x)),
        })
    }

    pub(in super) fn while_stmt(&mut self) -> Result<Stmt, String> {
        self.peekable.next();

        let cond = self.primary()?;
        let then = self.stmt()?;

        Ok(Stmt::While {
            cond: Box::new(cond),
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
            init: Box::new(init),
            cond: Box::new(cond),
            inc: Box::new(inc),
            then: Box::new(then)
        })
    }

    pub(in super) fn expr_stmt(&mut self) -> Result<Stmt, String> {
        Ok(Stmt::ExprStmt { val: self.expr()? })
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

    pub(in super) fn parse_args(&mut self) -> Result<Vec<Expr>, String> {
        // 最初の一個だけ読んでおく
        let mut args = vec![self.expr()?];
        while let Ok(_) = self.expect_next(",".to_string()) {
            args.push(self.expr()?);
        }

        Ok(args)
    }
}