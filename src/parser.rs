use crate::node::{ Stmt, Expr, ExprWrapper };
use crate::token::Token;
use crate::program::{ Function, Var };
use std::slice::Iter;
use std::iter::Peekable;
use std::rc::Rc;
use std::cell::RefCell;

mod parser_helper;

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
    // 関数の引数，関数内で宣言された変数を保持する, 関数のスコープから外れたらリセットする
    pub locals: Vec<Rc<RefCell<Var>>>
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a Vec<Token>) -> Self {
        Self {
            input,
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

    // function := basetype ident "(" params ")" "{" stmt* "}"
    // params := param ("," param)*
    // param := basetype ident
    fn function(&mut self) -> Result<Function, String> {
        self.base_type()?; // 一時的に無駄に消費するだけ(現状常にintなので)

        if let Some(Token::Ident{ name }) = self.peekable.next() {
            // parse params
            let params = self.parse_func_params()?;
            self.locals = params.clone();

            self.expect_next_symbol("{".to_string())?;

            let mut nodes = Vec::new();

            while let Err(_) = self.expect_next_symbol("}".to_string()) {
                nodes.push(self.stmt()?);
            };

            let locals = self.locals.to_vec();
            self.locals.clear();

            Ok(Function::new(name.to_string(), nodes, locals, params))
        } else {
            Err("expect ident, but different".to_string())
        }
    }

    // stmt := expr ";"
    //       | "return" expr ";"
    //       | "if" "(" expr ")" stmt ("else" stmt)?
    //       | "while" "(" expr ")" stmt
    //       | "for" "(" expr? ";" expr? ";" expr? ")" stmt
    //       | declaration
    fn stmt(&mut self) -> Result<Stmt, String> {
        let tk = self.peekable.peek();
        match tk {
            Some(Token::Reserved { op }) if *op == "return" => {
                self.peekable.next();

                let expr = self.expr()?;
                self.expect_next_symbol(";".to_string())?;

                Ok(Stmt::Return { val: ExprWrapper::new(expr) })
            }
            Some(Token::Symbol(op)) if *op == "{" => {
                self.peekable.next();
                let mut stmts: Vec<Stmt> = Vec::new();

                while let Err(_) = self.expect_next_symbol("}".to_string()) {
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
            Some(Token::Reserved { op }) if *op == "int" => {
                self.declaration()
            }
            _ => {
                let expr_stmt = self.expr_stmt();
                self.expect_next_symbol(";".to_string())?;

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
        (&node).as_ref().ok().and_then(|nd| {
            if let Expr::Var(var) = nd {
                return Some(var)
            }

            None
        });

        let is_assign = self.expect_next_reserved("=".to_string());
        if let Ok(_) = is_assign {
            let val = self.expr()?;
            return Ok(Expr::Assign {
                var: ExprWrapper::new(node?),
                val: ExprWrapper::new(val)
            })
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
                    node = Expr::Eq { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                }
                Token::Reserved { op } if *op == "!=" => {
                    self.peekable.next();

                    let rhs = self.relational()?;
                    node = Expr::Neq { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
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
                    node = Expr::Lt { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                }
                Token::Reserved { op } if *op == "<=" => {
                    self.peekable.next();

                    let rhs = self.add()?;
                    node = Expr::Le { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                }
                Token::Reserved { op } if *op == ">" => {
                    self.peekable.next();

                    let rhs = self.add()?;
                    node = Expr::Gt { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                }
                Token::Reserved { op } if *op == ">=" => {
                    self.peekable.next();

                    let rhs = self.add()?;
                    node = Expr::Ge { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                }
                _ => { return Ok(node); }
            }
        }

        Ok(node)
    }

    fn add(&mut self) -> Result<Expr, String> {
        let node = self.mul()?;

        while let Some(token) = self.peekable.peek() {
            match token {
                // "+" mul
                Token::Reserved { op } if *op == "+" => {
                    self.peekable.next();
                    let lhs = ExprWrapper::new(node);
                    let rhs = ExprWrapper::new(self.mul()?);

                    return Parser::new_add(lhs, rhs);
                },
                // "-" mul
                Token::Reserved { op } if *op == "-" => {
                    self.peekable.next();
                    let lhs = ExprWrapper::new(node);
                    let rhs = ExprWrapper::new(self.mul()?);

                    return Parser::new_sub(lhs, rhs);
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
                    node = Expr::Mul { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                },

                // "/" primary
                Token::Reserved { op } if *op == "/" => {
                    self.peekable.next();

                    let rhs = self.unary()?;
                    node = Expr::Div { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                },
                _ => {
                    return Ok(node);
                }
            }
        }


        Ok(node)
    }

    // unary := ("+" | "-" | "*" | "&")? unary
    //        | postfix
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
                        lhs: ExprWrapper::new(Expr::Num { val: 0 }),
                        rhs: ExprWrapper::new(rhs)
                    })
                },
                Token::Reserved { op } if *op == "*" => {
                    self.peekable.next();
                    let operand = self.unary()?;

                    Ok(Expr::Deref { operand: ExprWrapper::new(operand) })
                },
                Token::Reserved { op } if *op == "&" => {
                    self.peekable.next();
                    let operand = self.unary()?;

                    Ok(Expr::Addr { operand: ExprWrapper::new(operand) })
                }
                _ => {
                    self.primary()
                }
            }
        } else {
            self.postfix()
        }
    }

    fn postfix(&mut self) -> Result<Expr, String> {
        let node = self.primary()?;
        if let Ok(_) = self.expect_next_symbol("[".to_string()) {
            // x[y] is short for *(x + y)
            let expr = self.expr()?;
            let nd = Parser::new_add(node.to_expr_wrapper(), expr.to_expr_wrapper())?;

            match self.expect_next_symbol("]".to_string()) {
                Ok(_) => {
                    Ok(Expr::Deref { operand: nd.to_expr_wrapper() })
                },
                _ => Err("expect ] after [ expr".to_string())
            }
        } else {
            Ok(node)
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
            Some(Token::Symbol(op)) if op == "(" => {
                self.peekable.next();
                let expr = self.expr();
                self.expect_next_symbol(")".to_string())?;

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
                if let Ok(_) = self.expect_next_symbol("(".to_string()) {
                    // 引数なし
                    if let Ok(_) = self.expect_next_symbol(")".to_string()) {
                        return Ok(Expr::FnCall { fn_name: name.clone(), args: Vec::new() })
                    }
                    let args = self.parse_args()?;
                    self.expect_next_symbol(")".to_string())?;

                    return Ok(Expr::FnCall { fn_name: name.clone(), args })
                }
                // variable
                if let Some(ref var) = self.find_lvar(&name) {
                    Ok(Expr::Var(Rc::clone(var)))
                } else {
                    Err(format!("undefined variable: {:?}", name).to_string())
                }
            }
            // unexpected
            _ => {
                Err("unexpected token at primary".to_string())
            }
        }
    }
}

// #[test]
// fn parse_arithmetic_test() {
//     let input = vec![
//         Token::Num { val: 1, t_str: "1".to_string() },
//         Token::Reserved { op: "+".to_string() },
//         Token::Num { val: 2, t_str: "2".to_string() },
//         Token::Reserved { op: "*".to_string() },
//         Token::Num { val: 3, t_str: "3".to_string() },
//         Token::Reserved { op: "-".to_string() },
//         Token::Num { val: 20, t_str: "20".to_string() },
//         Token::Reserved { op: ";".to_string() },
//         Token::Eof
//     ];

//     let mut parser = Parser::new(&input);
//     let result = parser.parse().unwrap();

//     let expect = vec![
//         Stmt::ExprStmt {
//             val: Expr::Sub {
//                 lhs: Box::new(
//                     Expr::Add {
//                         lhs: Box::new(Expr::Num { val: 1 }),
//                         rhs: Box::new(
//                             Expr::Mul {
//                                 lhs: Box::new(Expr::Num { val: 2 }),
//                                 rhs: Box::new(Expr::Num { val: 3 })
//                             }
//                         )
//                     }
//                 ),
//                 rhs: Box::new(Expr::Num {val: 20 })
//             }
//         }
//     ];

//     assert_eq!(result.first().unwrap().nodes, expect);
// }

// #[test]
// fn parse_return_test() {
//     let input = vec![
//         Token::Ident { name: "foo".to_string() },
//         Token::Reserved { op: "=".to_string() },
//         Token::Num { val: 1, t_str: "1".to_string() },
//         Token::Reserved { op: ";".to_string() },
//         Token::Reserved { op: "return".to_string() },
//         Token::Ident { name: "foo".to_string() },
//         Token::Reserved { op: ";".to_string() },
//         Token::Eof
//     ];

//     let mut parser = Parser::new(&input);
//     let result = parser.parse().unwrap();
//     let expect = vec![
//         Stmt::ExprStmt {
//             val: Expr::Assign {
//                 var: Var {
//                         name: "foo".to_string(),
//                         offset: 8
//                     },
//                 val: Box::new(Expr::Num {val: 1 })
//             }
//         },
//         Stmt::Return {
//             val: Expr::Var {
//                 var: Var {
//                     name: "foo".to_string(),
//                     offset: 8
//                 }
//             }
//         }
//     ];

//     assert_eq!(result.first().unwrap().nodes, expect);
// }

// #[test]
// fn parse_cmp_test() {
//     let input = vec![
//         Token::Num { val: 1, t_str: "1".to_string() },
//         Token::Reserved { op: ">=".to_string() },
//         Token::Num { val: 1, t_str: "1".to_string() },
//         Token::Reserved { op: "<".to_string() },
//         Token::Num { val: 1, t_str: "1".to_string() },
//         Token::Reserved { op: "==".to_string() },
//         Token::Num { val: 2, t_str: "2".to_string() },
//         Token::Reserved { op: ";".to_string() },
//         Token::Eof
//     ];

//     let mut parser = Parser::new(&input);
//     let result = parser.parse().unwrap();
//     let expect = vec![
//         Stmt::ExprStmt {
//             val: Expr::Eq {
//                 lhs: Box::new(
//                     Expr::Lt {
//                         lhs: Box::new(
//                             Expr::Ge {
//                                 lhs: Box::new(Expr::Num { val: 1 }),
//                                 rhs: Box::new(Expr::Num { val: 1 })
//                             }
//                         ),
//                         rhs: Box::new(Expr::Num { val: 1 }),
//                     }
//                 ),
//                 rhs: Box::new(Expr::Num {val: 2 })
//             }
//         }
//     ];

//     assert_eq!(result.first().unwrap().nodes, expect);
// }
