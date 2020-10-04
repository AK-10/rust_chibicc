use crate::node::{ Stmt, Expr };
use crate::token::Token;
use crate::program::{ Function, Var };
use std::slice::Iter;
use std::iter::Peekable;

// 優先順位
// == !=
// < <= > >=
// + -
// * /
// 単項+ 単項-
// ()

pub struct Parser<'a> {
    pub input: &'a Vec<Token>,
    peekable: Peekable<Iter<'a, Token>>,
    pub locals: Vec<Var>
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a Vec<Token>) -> Self {
        Self {
            input: input,
            peekable: input.iter().peekable(),
            locals: Vec::new()
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Function>, String> {
       self.program()
    }

    // program := stmt*
    fn program(&mut self) -> Result<Vec<Function>, String> {
        let mut nodes: Vec<Function> = Vec::new();

        while let Some(token) = self.peekable.peek() {
            // eofでbreakしないと，以降の処理でpeek()するので全体としてErrになる(Noneでエラーにするような処理がprimaryにある)
            if let Token::Eof = token {
                break
            }
            nodes.push(self.function()?);
        };

        Ok(nodes)
    }

    // function := ident "(" ")" "{" stmt* "}"
    fn function(&mut self) -> Result<Function, String> {
        if let Some(Token::Ident{ name }) = self.peekable.next() {
            self.expect_next("(".to_string())?;
            self.expect_next(")".to_string())?;
            self.expect_next("{".to_string())?;

            let mut nodes = Vec::new();

            while let Err(_) = self.expect_next("}".to_string()) {
                nodes.push(self.stmt()?);
            };

            // self.expect_next("}".to_string())?;
            let locals = self.locals.to_vec();
            self.locals.clear();

            Ok(Function::new(name.to_string(), nodes, locals))
        } else {
            Err("expect ident, but different".to_string())
        }
    }

    // stmt := expr ";"
    //       | "return" expr ";"
    //       | "if" "(" expr ")" stmt ("else" stmt)?
    //       | "while" "(" expr ")" stmt
    //       | "for" "(" expr? ";" expr? ";" expr? ")" stmt
    fn stmt(&mut self) -> Result<Stmt, String> {
        let tk = self.peekable.peek();
        match tk {
            Some(Token::Reserved { op }) if *op == "return" => {
                self.peekable.next();

                let expr = self.expr()?;
                self.expect_next(";".to_string())?;

                Ok(Stmt::Return { val: expr })
            }
            Some(Token::Reserved { op }) if *op == "{" => {
                self.peekable.next();
                let mut stmts: Vec<Stmt> = Vec::new();

                while let Err(_) = self.expect_next("}".to_string()) {
                    let stmt = self.stmt()?;
                    stmts.push(stmt);
                }

                Ok(Stmt::Block { stmts })
            }
            Some(Token::Reserved { op }) if *op == "if" => {
                self.if_stmt()
            }
            Some(Token::Reserved { op }) if *op == "while" => {
                self.while_stmt()
            }
            Some(Token::Reserved { op }) if *op == "for" => {
                self.for_stmt()
            }
            _ => {
                let expr_stmt = self.expr_stmt();
                self.expect_next(";".to_string())?;

                expr_stmt
            }
        }
    }

    // expr := assign
    fn expr(&mut self) -> Result<Expr, String> {
        self.assign()
    }

    // assign := equality ("=" assign)?
    fn assign(&mut self) -> Result<Expr, String> {
        let node = self.equality();
        let var = (&node).as_ref().ok().and_then(|nd| {
            if let Expr::Var { var } = nd {
                return Some(var)
            }

            None
        });

        if let Some(v) = var {
            let is_assign = self.expect_next("=".to_string());
            if let Ok(_) = is_assign {
                return Ok(Expr::Assign {
                    var: v.clone(),
                    val: Box::new(self.expr()?)
                })
            }
        }

        node
    }

    // equality := relational ("==" relational | "!=" relational)*
    fn equality(&mut self) -> Result<Expr, String> {
        let mut node = self.relational()?;

        while let Some(token) = self.peekable.peek() {
            match token {
                Token::Reserved { op } if *op == "==" => {
                    self.peekable.next();

                    let rhs = self.relational()?;
                    node = Expr::Eq { lhs: Box::new(node), rhs: Box::new(rhs) };
                }
                Token::Reserved { op } if *op == "!=" => {
                    self.peekable.next();

                    let rhs = self.relational()?;
                    node = Expr::Neq { lhs: Box::new(node), rhs: Box::new(rhs) };
                }
                _ => { return Ok(node); }
            }
        }

        Ok(node)
    }

    // relational := add ("<" add | "<=" add | ">" add | ">=" add)*
    fn relational(&mut self) -> Result<Expr, String> {
        let mut node = self.add()?;

        while let Some(token) = self.peekable.peek() {
            match token {
                Token::Reserved { op } if *op == "<" => {
                    self.peekable.next();

                    let rhs = self.add()?;
                    node = Expr::Lt { lhs: Box::new(node), rhs: Box::new(rhs) };
                }
                Token::Reserved { op } if *op == "<=" => {
                    self.peekable.next();

                    let rhs = self.add()?;
                    node = Expr::Le { lhs: Box::new(node), rhs: Box::new(rhs) };
                }
                Token::Reserved { op } if *op == ">" => {
                    self.peekable.next();

                    let rhs = self.add()?;
                    node = Expr::Gt { lhs: Box::new(node), rhs: Box::new(rhs) };
                }
                Token::Reserved { op } if *op == ">=" => {
                    self.peekable.next();

                    let rhs = self.add()?;
                    node = Expr::Ge { lhs: Box::new(node), rhs: Box::new(rhs) };
                }
                _ => { return Ok(node); }
            }
        }

        Ok(node)
    }

    fn add(&mut self) -> Result<Expr, String> {
        let mut node = self.mul()?;

        while let Some(token) = self.peekable.peek() {
            match token {
                // "+" mul
                Token::Reserved { op } if *op == "+" => {
                    self.peekable.next();

                    let rhs = self.mul()?;
                    node = Expr::Add { lhs: Box::new(node), rhs: Box::new(rhs) };
                },
                // "-" mul
                Token::Reserved { op } if *op == "-" => {
                    self.peekable.next();

                    let rhs = self.mul()?;
                    node = Expr::Sub { lhs: Box::new(node), rhs: Box::new(rhs) };
                },
                // mul
                _ => { return Ok(node); }
            };
        }

        Ok(node)
    }

    fn mul(&mut self) -> Result<Expr, String> {
        let mut node = self.unary()?;

        while let Some(token) = self.peekable.peek() {
            match token {
                // "*" primary
                Token::Reserved { op } if *op == "*" => {
                    self.peekable.next();

                    let rhs = self.unary()?;
                    node = Expr::Mul { lhs: Box::new(node), rhs: Box::new(rhs) };
                },

                // "/" primary
                Token::Reserved { op } if *op == "/" => {
                    self.peekable.next();

                    let rhs = self.unary()?;
                    node = Expr::Div { lhs: Box::new(node), rhs: Box::new(rhs) };
                },
                _ => {
                    return Ok(node);
                }
            }
        }

        Ok(node)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if let Some(token) = self.peekable.peek() {

            match token {
                Token::Reserved { op } if *op == "+" => {
                    self.peekable.next();

                    self.primary()
                },
                Token::Reserved { op } if *op == "-" => {
                    self.peekable.next();

                    let rhs = self.unary()?;
                    Ok(Expr::Sub {
                        lhs: Box::new(Expr::Num { val: 0 }),
                        rhs: Box::new(rhs)
                    })
                },
                _ => {
                    self.primary()
                }
            }
        } else {
            Err("expect token, but token not found".to_string())
        }
    }

    // ERR: compile error
    // error: expected `,`
    //    --> src/parser.rs:106:39
    //     |
    // 106 |     if let Some(Token::Reserved { op: '('.to_string() }) = token {
    //     |
    // fn primary(peekable: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    //     let token = peekable.next();

    //     if let Some(Token::Reserved { op: '('.to_string() }) = token {
    //         let expr = expr(peekable);
    //         match peekable.next() {
    //             Some(Token::Reserved { op: ')', .. }) =>  { return expr; },
    //             _ => { return Err("fail primary".to_string()); }
    //         };
    //     // num
    //     } else if let Some(Token::Num { val, .. }) = token {
    //         return Ok(Node::Num { val: *val })

    //     // unexpected
    //     } else {
    //         return Err("unexpected token at primary".to_string());
    //     }
    // }

    // primary = "(" expr ")" | ident | num
    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.peekable.peek();

        match token {
            // ERR: compile error
            // expected tuple struct or tuple variant, found associated function `String::from`
            // Some(Token::Reserved { op: String::from("(") }) => {}
            Some(Token::Reserved { op }) if *op == "(" => {
                self.peekable.next();
                let expr = self.expr();
                self.expect_next(")".to_string())?;

                expr
            }
            // num
            Some(Token::Num { val, .. }) => {
                self.peekable.next();
                Ok(Expr::Num { val: *val })
            }
            // local var
            Some(Token::Ident { name }) => {
                // function call
                self.peekable.next();
                if let Ok(_) = self.expect_next("(".to_string()) {

                    if let Ok(_) = self.expect_next(")".to_string()) {
                        return Ok(Expr::FnCall { fn_name: name.clone(), args: Vec::new() })
                    }
                    let args = self.parse_args()?;
                    self.expect_next(")".to_string())?;

                    return Ok(Expr::FnCall { fn_name: name.clone(), args: args })
                }
                // variable
                if let Some(var) = self.find_lvar(name) {
                    Ok(Expr::Var { var: var.clone() })
                } else {
                    let offset = (self.locals.len() + 1) * 8;
                    let var = Var { name: name.clone(), offset: offset };
                    self.locals.push(var.clone());

                    Ok(Expr::Var { var: var })
                }
            }
            // unexpected
            _ => {
                Err("unexpected token at primary".to_string())
            }
        }
    }

    fn find_lvar(&self, name: &String) -> Option<&Var> {
        self.locals.iter().find(|item| { item.name == *name })
    }

    fn if_stmt(&mut self) -> Result<Stmt, String> {
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

    fn while_stmt(&mut self) -> Result<Stmt, String> {
        self.peekable.next();

        let cond = self.primary()?;
        let then = self.stmt()?;

        Ok(Stmt::While {
            cond: Box::new(cond),
            then: Box::new(then)
        })
    }

    fn for_stmt(&mut self) -> Result<Stmt, String> {
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

    fn expr_stmt(&mut self) -> Result<Stmt, String> {
        Ok(Stmt::ExprStmt { val: self.expr()? })
    }

    fn expect_next(&mut self, word: String) -> Result<(), String> {
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

    fn parse_args(&mut self) -> Result<Vec<Expr>, String> {
        // 最初の一個だけ読んでおく
        let mut args = vec![self.expr()?];
        while let Ok(_) = self.expect_next(",".to_string()) {
            args.push(self.expr()?);
        }

        Ok(args)
    }
}

#[test]
fn parse_arithmetic_test() {
    let input = vec![
        Token::Num { val: 1, t_str: "1".to_string() },
        Token::Reserved { op: "+".to_string() },
        Token::Num { val: 2, t_str: "2".to_string() },
        Token::Reserved { op: "*".to_string() },
        Token::Num { val: 3, t_str: "3".to_string() },
        Token::Reserved { op: "-".to_string() },
        Token::Num { val: 20, t_str: "20".to_string() },
        Token::Reserved { op: ";".to_string() },
        Token::Eof
    ];

    let mut parser = Parser::new(&input);
    let result = parser.parse().unwrap();

    let expect = vec![
        Stmt::ExprStmt {
            val: Expr::Sub {
                lhs: Box::new(
                    Expr::Add {
                        lhs: Box::new(Expr::Num { val: 1 }),
                        rhs: Box::new(
                            Expr::Mul {
                                lhs: Box::new(Expr::Num { val: 2 }),
                                rhs: Box::new(Expr::Num { val: 3 })
                            }
                        )
                    }
                ),
                rhs: Box::new(Expr::Num {val: 20 })
            }
        }
    ];

    assert_eq!(result.nodes, expect);
}

#[test]
fn parse_return_test() {
    let input = vec![
        Token::Ident { name: "foo".to_string() },
        Token::Reserved { op: "=".to_string() },
        Token::Num { val: 1, t_str: "1".to_string() },
        Token::Reserved { op: ";".to_string() },
        Token::Reserved { op: "return".to_string() },
        Token::Ident { name: "foo".to_string() },
        Token::Reserved { op: ";".to_string() },
        Token::Eof
    ];

    let mut parser = Parser::new(&input);
    let result = parser.parse().unwrap();
    let expect = vec![
        Stmt::ExprStmt {
            val: Expr::Assign {
                var: Var {
                        name: "foo".to_string(),
                        offset: 8
                    },
                val: Box::new(Expr::Num {val: 1 })
            }
        },
        Stmt::Return {
            val: Expr::Var {
                var: Var {
                    name: "foo".to_string(),
                    offset: 8
                }
            }
        }
    ];

    assert_eq!(result.nodes, expect);
}

#[test]
fn parse_cmp_test() {
    let input = vec![
        Token::Num { val: 1, t_str: "1".to_string() },
        Token::Reserved { op: ">=".to_string() },
        Token::Num { val: 1, t_str: "1".to_string() },
        Token::Reserved { op: "<".to_string() },
        Token::Num { val: 1, t_str: "1".to_string() },
        Token::Reserved { op: "==".to_string() },
        Token::Num { val: 2, t_str: "2".to_string() },
        Token::Reserved { op: ";".to_string() },
        Token::Eof
    ];

    let mut parser = Parser::new(&input);
    let result = parser.parse().unwrap();
    let expect = vec![
        Stmt::ExprStmt {
            val: Expr::Eq {
                lhs: Box::new(
                    Expr::Lt {
                        lhs: Box::new(
                            Expr::Ge {
                                lhs: Box::new(Expr::Num { val: 1 }),
                                rhs: Box::new(Expr::Num { val: 1 })
                            }
                        ),
                        rhs: Box::new(Expr::Num { val: 1 }),
                    }
                ),
                rhs: Box::new(Expr::Num {val: 2 })
            }
        }
    ];

    assert_eq!(result.nodes, expect);
}