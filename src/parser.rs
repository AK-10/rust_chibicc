use crate::node::Node;
use crate::token::Token;
use std::slice::Iter;
use std::iter::Peekable;

pub fn parse(tokens: Vec<Token>) -> Result<Node, String> {
    let peekable_tokens = &mut tokens.iter().peekable();

    expr(peekable_tokens)
}

fn expr(peekable: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    let mut node = mul(peekable)?;

    while let Some(token) = peekable.peek() {
        match token {
            // "+" mul
            Token::Reserved { op: '+', .. } => {
                peekable.next();

                let rhs = mul(peekable)?;
                node = Node::Add { lhs: Box::new(node), rhs: Box::new(rhs) };
            },
            // "-" mul
            Token::Reserved { op: '-', .. } => {
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
    let mut node = primary(peekable)?;

    while let Some(token) = peekable.peek() {
        // shadowing and move?
        // let token = *token;
        let tk = *token;
        match tk {
            // "*" primary
            Token::Reserved { op: '*', .. } => {
                peekable.next();

                let rhs = unary(peekable)?;
                node = Node::Mul { lhs: Box::new(node), rhs: Box::new(rhs) };
            },

            // "/" primary
            Token::Reserved { op: '/', .. } => {
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
            Token::Reserved { op: '+', .. } => {
                peekable.next();

                primary(peekable)
            },
            Token::Reserved { op: '-', .. } => {
                peekable.next();

                let rhs = primary(peekable)?;
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

fn primary(peekable: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    let token = peekable.next();

    // ( expr ): not work :(
    if let Some(Token::Reserved { op: '(', .. }) = token {
        let expr = expr(peekable);
        match peekable.next() {
            Some(Token::Reserved { op: ')', .. }) =>  { return expr; },
            _ => { return Err("fail primary".to_string()); }
        };
    // num
    } else if let Some(Token::Num { val, .. }) = token {
        return Ok(Node::Num { val: *val })

    // unexpected
    } else {
        return Err("unexpected token at primary".to_string());
    }
}

#[test]
fn parse_test() {
    let input = vec![
        Token::Num { val: 1, t_str: "1".to_string() },
        Token::Reserved { op: '+', t_str: "+".to_string() },
        Token::Num { val: 2, t_str: "2".to_string() },
        Token::Reserved { op: '*', t_str: "*".to_string() },
        Token::Num { val: 3, t_str: "3".to_string() },
        Token::Reserved { op: '-', t_str: "-".to_string() },
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
