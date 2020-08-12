use crate::node::Node;
use crate::token::Token;
use std::slice::Iter;
use std::iter::Peekable;

// 優先順位
// == !=
// < <= > >=
// + -
// * /
// 単項+ 単項-
// ()

// struct Parser {
//     codes: Vec<Node>,
//     input: Vec<Token>
// }

// impl Parser {

// }

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Node>, String> {
    let peekable_tokens = &mut tokens.iter().peekable();

    program(peekable_tokens)
}

fn program(peekable: &mut Peekable<Iter<Token>>) -> Result<Vec<Node>, String> {
    let mut nodes: Vec<Node> = Vec::new();
    while let Some(_) = peekable.peek() {
        nodes.push(stmt(peekable)?);
    };

    Ok(nodes)
}

fn stmt(peekable: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    let expr = expr(peekable);
    match peekable.next() {
        Some(Token::Reserved { op }) if *op == String::from(";") => expr,
        _ => Err("delemiter not found".to_string())
    }
}

fn expr(peekable: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    assign(peekable)
}

// TODO: impl
fn assign(peekable: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    let mut node = equality(peekable);
    if let Some(token) = peekable.peek() {
        match
    }
}

fn equality(peekable: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    let mut node = relational(peekable)?;

    while let Some(token) = peekable.peek() {
        match token {
            Token::Reserved { op } if *op == "==" => {
                peekable.next();

                let rhs = relational(peekable)?;
                node = Node::Eq { lhs: Box::new(node), rhs: Box::new(rhs) };
            }
            Token::Reserved { op } if *op == "!=" => {
                peekable.next();

                let rhs = relational(peekable)?;
                node = Node::Neq { lhs: Box::new(node), rhs: Box::new(rhs) };
            }
            _ => { return Ok(node); }
        }
    }

    Ok(node)
}

fn relational(peekable: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    let mut node = add(peekable)?;

    while let Some(token) = peekable.peek() {
        match token {
            Token::Reserved { op } if *op == "<" => {
                peekable.next();

                let rhs = add(peekable)?;
                node = Node::Lt { lhs: Box::new(node), rhs: Box::new(rhs) };
            }
            Token::Reserved { op } if *op == "<=" => {
                peekable.next();

                let rhs = add(peekable)?;
                node = Node::Le { lhs: Box::new(node), rhs: Box::new(rhs) };
            }
            Token::Reserved { op } if *op == ">" => {
                peekable.next();

                let rhs = add(peekable)?;
                node = Node::Gt { lhs: Box::new(node), rhs: Box::new(rhs) };
            }
            Token::Reserved { op } if *op == ">=" => {
                peekable.next();

                let rhs = add(peekable)?;
                node = Node::Ge { lhs: Box::new(node), rhs: Box::new(rhs) };
            }
            _ => { return Ok(node); }
        }
    }

    Ok(node)
}

fn add(peekable: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    let mut node = mul(peekable)?;

    while let Some(token) = peekable.peek() {
        match token {
            // "+" mul
            Token::Reserved { op } if *op == "+" => {
                peekable.next();

                let rhs = mul(peekable)?;
                node = Node::Add { lhs: Box::new(node), rhs: Box::new(rhs) };
            },
            // "-" mul
            Token::Reserved { op } if *op == "-" => {
                peekable.next();

                let rhs = mul(peekable)?;
                node = Node::Sub { lhs: Box::new(node), rhs: Box::new(rhs) };
            },
            // mul
            _ => { return Ok(node); }
        };
    }

    Ok(node)
}

fn mul(peekable: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    let mut node = unary(peekable)?;

    while let Some(token) = peekable.peek() {
        match token {
            // "*" primary
            Token::Reserved { op } if *op == "*" => {
                peekable.next();

                let rhs = unary(peekable)?;
                node = Node::Mul { lhs: Box::new(node), rhs: Box::new(rhs) };
            },

            // "/" primary
            Token::Reserved { op } if *op == "/" => {
                peekable.next();

                let rhs = unary(peekable)?;
                node = Node::Div { lhs: Box::new(node), rhs: Box::new(rhs) };
            },
            _ => {
                return Ok(node);
            }
        }
    }

    Ok(node)
}

fn unary(peekable: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    if let Some(token) = peekable.peek() {

        match token {
            Token::Reserved { op } if *op == "+" => {
                peekable.next();

                primary(peekable)
            },
            Token::Reserved { op } if *op == "-" => {
                peekable.next();

                let rhs = unary(peekable)?;
                Ok(Node::Sub {
                    lhs: Box::new(Node::Num{ val: 0 }),
                    rhs: Box::new(rhs)
                })
            },
            _ => {
                primary(peekable)
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

fn primary(peekable: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    let token = peekable.next();
    match token {
        // ERR: compile error
        // expected tuple struct or tuple variant, found associated function `String::from`
        // Some(Token::Reserved { op: String::from("(") }) => {}
        Some(Token::Reserved { op }) if *op == "(" => {
            let expr = expr(peekable);
            match peekable.next() {
                Some(Token::Reserved { op }) if *op == ")" => expr,
                _ => Err("fail primary".to_string())
            }
        }
        // num
        Some(Token::Num { val, .. }) => {
            Ok(Node::Num { val: *val })
        }
        // unexpected
        _ => {
            Err("unexpected token at primary".to_string())
        }
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
        Token::Eof
    ];

    let result = parse(input).unwrap();

    let expect = Node::Sub {
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
    };

    assert_eq!(result, expect);
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
        Token::Eof
    ];

    let result = parse(input).unwrap();

    let expect = Node::Eq {
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
    };

    assert_eq!(result, expect);
}