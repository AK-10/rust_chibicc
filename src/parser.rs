use crate::node::Node;
use crate::token::Token;
use std::{iter::Peekable, slice::Iter};

pub fn parse(tokens: Vec<Token>) -> Result<Node, String> {
    let peekable_tokens = &mut tokens.iter().peekable();

    expr(peekable_tokens)
}

fn expr(peekables: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    let lhs = mul(peekables)?;

    while let Some(token) = peekables.next() {
        let rhs = mul(peekables)?;
        if let Token::Reserved { op: '+', .. } = token {
            return Ok(Node::Add { lhs: Box::new(lhs), rhs: Box::new(rhs) });
        } else if let Token::Reserved { op: '-', .. } = token {
            return Ok(Node::Sub { lhs: Box::new(lhs), rhs: Box::new(rhs) });
        } else {
            return Err("fail expr".to_string())
        }
    }

    return Err("fail expr".to_string())
}

fn mul(peekables: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    let lhs = primary(peekables)?;

    while let Some(token) = peekables.next() {
        let rhs = primary(peekables)?;
        if let Token::Reserved { op: '*', .. } = token {
            return Ok(Node::Mul { lhs: Box::new(lhs), rhs: Box::new(rhs) });
        } else if let Token::Reserved { op: '/', .. } = token {
            return Ok(Node::Mul { lhs: Box::new(lhs), rhs: Box::new(rhs) });
        } else {
            return Err("fail mul".to_string())
        }
    }

    return Err("fail mul".to_string())
}

fn primary(peekables: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    while let Some(token) = peekables.next() {
        if let Token::Reserved { op: '(', .. } = token {
            let expr = expr(peekables);
            if let Some(Token::Reserved { op: ')', .. }) = peekables.peek() {
                return expr;
            } else {
                return Err("fail primary".to_string());
            }
        } else if let Token::Num { val, .. } = token {
            return Ok(Node::Num { val: *val })
        } else {
            return Err("fail primary".to_string());
        }
    }

    return Err("fail primary".to_string())
}

#[test]
fn parse_test() {
    unimplemented!();
}
