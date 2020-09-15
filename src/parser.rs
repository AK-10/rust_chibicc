use crate::node::Node;
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

    pub fn parse(&mut self) -> Result<Function, String> {
        let node = self.program()?;

        Ok(Function::new(node, self.locals.to_vec()))
    }

    // program := stmt*
    fn program(&mut self) -> Result<Vec<Node>, String> {
        let mut nodes: Vec<Node> = Vec::new();

        while let Some(token) = self.peekable.peek() {
            if let Token::Eof = token {
                break;
            }

            nodes.push(self.stmt()?);
        };

        Ok(nodes)
    }

    // stmt := expr ";"
    //       | "return" expr ";"
    //       | "if" "(" expr ")" stmt ("else" stmt)?
    //       | "while" "(" expr ")" stmt
    //       | "for" "(" expr? ";" expr? ";" expr? ")" stmt
    fn stmt(&mut self) -> Result<Node, String> {
        match self.peekable.peek() {
            Some(Token::Reserved { op }) if *op == "return" => {
                self.peekable.next();

                let expr = self.expr()?;
                match self.peekable.next() {
                    Some(Token::Reserved { op }) if *op == ";" => Ok(Node::Return { val: Box::new(expr) }),
                    _ => Err("delemiter not found".to_string())
                }
            }
            Some(Token::Reserved { op }) if *op == "if" => {
                self.if_stmt()
            }
            Some(Token::Reserved { op }) if *op == "while" => {
                self.while_stmt()
            }
            _ => {
                let expr = self.expr()?;

                match self.peekable.next() {
                    Some(Token::Reserved { op }) if *op == ";" => Ok(Node::ExprStmt { val: Box::new(expr) }),
                    _ => Err("delemiter not found".to_string())
                }
            }
        }
    }

    // expr := assign
    fn expr(&mut self) -> Result<Node, String> {
        self.assign()
    }

    // assign := equality ("=" assign)?
    fn assign(&mut self) -> Result<Node, String> {
        let mut node = self.equality()?;
        if let Some(token) = self.peekable.peek() {
            match token {
                Token::Reserved { op } if *op == "=" => {
                    self.peekable.next();
                    node = Node::Assign {
                        var: Box::new(node),
                        val: Box::new(self.assign()?)
                    }
                }
                _ => {}
            }
        };

        Ok(node)
    }

    // equality := relational ("==" relational | "!=" relational)*
    fn equality(&mut self) -> Result<Node, String> {
        let mut node = self.relational()?;

        while let Some(token) = self.peekable.peek() {
            match token {
                Token::Reserved { op } if *op == "==" => {
                    self.peekable.next();

                    let rhs = self.relational()?;
                    node = Node::Eq { lhs: Box::new(node), rhs: Box::new(rhs) };
                }
                Token::Reserved { op } if *op == "!=" => {
                    self.peekable.next();

                    let rhs = self.relational()?;
                    node = Node::Neq { lhs: Box::new(node), rhs: Box::new(rhs) };
                }
                _ => { return Ok(node); }
            }
        }

        Ok(node)
    }

    // relational := add ("<" add | "<=" add | ">" add | ">=" add)*
    fn relational(&mut self) -> Result<Node, String> {
        let mut node = self.add()?;

        while let Some(token) = self.peekable.peek() {
            match token {
                Token::Reserved { op } if *op == "<" => {
                    self.peekable.next();

                    let rhs = self.add()?;
                    node = Node::Lt { lhs: Box::new(node), rhs: Box::new(rhs) };
                }
                Token::Reserved { op } if *op == "<=" => {
                    self.peekable.next();

                    let rhs = self.add()?;
                    node = Node::Le { lhs: Box::new(node), rhs: Box::new(rhs) };
                }
                Token::Reserved { op } if *op == ">" => {
                    self.peekable.next();

                    let rhs = self.add()?;
                    node = Node::Gt { lhs: Box::new(node), rhs: Box::new(rhs) };
                }
                Token::Reserved { op } if *op == ">=" => {
                    self.peekable.next();

                    let rhs = self.add()?;
                    node = Node::Ge { lhs: Box::new(node), rhs: Box::new(rhs) };
                }
                _ => { return Ok(node); }
            }
        }

        Ok(node)
    }

    fn add(&mut self) -> Result<Node, String> {
        let mut node = self.mul()?;

        while let Some(token) = self.peekable.peek() {
            match token {
                // "+" mul
                Token::Reserved { op } if *op == "+" => {
                    self.peekable.next();

                    let rhs = self.mul()?;
                    node = Node::Add { lhs: Box::new(node), rhs: Box::new(rhs) };
                },
                // "-" mul
                Token::Reserved { op } if *op == "-" => {
                    self.peekable.next();

                    let rhs = self.mul()?;
                    node = Node::Sub { lhs: Box::new(node), rhs: Box::new(rhs) };
                },
                // mul
                _ => { return Ok(node); }
            };
        }

        Ok(node)
    }

    fn mul(&mut self) -> Result<Node, String> {
        let mut node = self.unary()?;

        while let Some(token) = self.peekable.peek() {
            match token {
                // "*" primary
                Token::Reserved { op } if *op == "*" => {
                    self.peekable.next();

                    let rhs = self.unary()?;
                    node = Node::Mul { lhs: Box::new(node), rhs: Box::new(rhs) };
                },

                // "/" primary
                Token::Reserved { op } if *op == "/" => {
                    self.peekable.next();

                    let rhs = self.unary()?;
                    node = Node::Div { lhs: Box::new(node), rhs: Box::new(rhs) };
                },
                _ => {
                    return Ok(node);
                }
            }
        }

        Ok(node)
    }

    fn unary(&mut self) -> Result<Node, String> {
        if let Some(token) = self.peekable.peek() {

            match token {
                Token::Reserved { op } if *op == "+" => {
                    self.peekable.next();

                    self.primary()
                },
                Token::Reserved { op } if *op == "-" => {
                    self.peekable.next();

                    let rhs = self.unary()?;
                    Ok(Node::Sub {
                        lhs: Box::new(Node::Num{ val: 0 }),
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
    fn primary(&mut self) -> Result<Node, String> {
        let token = self.peekable.next();

        match token {
            // ERR: compile error
            // expected tuple struct or tuple variant, found associated function `String::from`
            // Some(Token::Reserved { op: String::from("(") }) => {}
            Some(Token::Reserved { op }) if *op == "(" => {
                let expr = self.expr();
                match self.peekable.next() {
                    Some(Token::Reserved { op }) if *op == ")" => expr,
                    _ => Err("fail primary".to_string())
                }
            }
            // num
            Some(Token::Num { val, .. }) => {
                Ok(Node::Num { val: *val })
            }
            // local var
            Some(Token::Ident { name }) => {
                if let Some(var) = self.find_lvar(name) {
                    Ok(Node::Var { name: name.clone(), offset: var.offset })
                } else {
                    let offset = (self.locals.len() + 1) * 8;
                    let var = Var { name: name.clone(), offset: offset };
                    self.locals.push(var);

                    Ok(Node::Var { name: name.clone(), offset: offset })
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

    fn if_stmt(&mut self) -> Result<Node, String> {
        self.peekable.next();

        let cond = self.primary()?;
        let then = self.stmt()?;
        let els = match self.peekable.peek() {
            Some(Token::Reserved { op }) if *op == "else" => {
                self.peekable.next();

                Some(self.stmt()?)
            },
            _ => None
        };

        Ok(Node::If {
            cond: Box::new(cond),
            then: Box::new(then),
            els: els.map(|x| Box::new(x)),
        })
    }

    fn while_stmt(&mut self) -> Result<Node, String> {
        self.peekable.next();

        let cond = self.primary()?;
        let then = self.stmt()?;

        Ok(Node::While {
            cond: Box::new(cond),
            then: Box::new(then)
        })
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
        Node::ExprStmt { val: Box::new(
            Node::Sub {
                lhs: Box::new(
                    Node::Add {
                        lhs: Box::new(Node::Num { val: 1 }),
                        rhs: Box::new(
                            Node::Mul {
                                lhs: Box::new(Node::Num { val: 2 }),
                                rhs: Box::new(Node::Num { val: 3 })
                            }
                        )
                    }
                ),
                rhs: Box::new(Node::Num {val: 20 })
            }
        )}
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
        Node::ExprStmt { val: Box::new(
            Node::Assign {
                var: Box::new(
                    Node::Var {
                        name: "foo".to_string(),
                        offset: 8
                    }
                ),
                val: Box::new(Node::Num {val: 1 })
            }
        )},
        Node::Return {
            val: Box::new(
                Node::Var {
                    name: "foo".to_string(),
                    offset: 8
                }
            )
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
        Node::ExprStmt { val: Box::new(
            Node::Eq {
                lhs: Box::new(
                    Node::Lt {
                        lhs: Box::new(
                            Node::Ge {
                                lhs: Box::new(Node::Num { val: 1 }),
                                rhs: Box::new(Node::Num { val: 1 })
                            }
                        ),
                        rhs: Box::new(Node::Num { val: 1 }),
                    }
                ),
                rhs: Box::new(Node::Num {val: 2 })
            }
        )}
    ];

    assert_eq!(result.nodes, expect);
}