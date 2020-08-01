use crate::node::Node;
use crate::token::Token;
use std::{iter::Peekable, slice::Iter};

pub fn parse(tokens: Vec<Token>) -> Result<Node, String> {
    let mut peekable_tokens = tokens.iter().peekable();
    
    while let Some(token) = peekable_tokens.peek() {
        match token {

        }
    }
}

fn expr(peekables: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    let lhs = mul(&mut peekables)?;

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
    let lhs = primary(&mut peekables)?;

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

fn primary(peekable: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    while let Some(token) = peekables.next() {
        
    }

    return Err("fail mul".to_string())
}
