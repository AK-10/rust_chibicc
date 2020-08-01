use crate::node::Node;
use crate::token::Token;
use std::slice::Iter;
use std::iter::Peekable;

pub fn parse(tokens: Vec<Token>) -> Result<Node, String> {
    let peekable_tokens = &mut tokens.iter().peekable();
    // println!("{:?}", tokens);

    expr(peekable_tokens)
}

fn expr(peekables: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    let mut node = mul(peekables)?;

    while let Some(token) = peekables.next() {
        match token {
            Token::Reserved { op: '+', .. } => {
                let rhs = mul(peekables)?;
                node = Node::Add { lhs: Box::new(node), rhs: Box::new(rhs) };
            },
            Token::Reserved { op: '-', .. } => {
                let rhs = mul(peekables)?;
                node = Node::Sub { lhs: Box::new(node), rhs: Box::new(rhs) };
            },
            _ => { return Ok(node); }
        };
    }

    Ok(node)
}

fn mul(peekables: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    let mut node = primary(peekables)?;

    while let Some(token) = peekables.peek() {
        let token = *token;

        if let Token::Reserved { op: '*', .. } = token {
            peekables.next();

            let rhs = primary(peekables)?;
            node = Node::Mul { lhs: Box::new(node), rhs: Box::new(rhs) };

            peekables.next();
        } else if let Token::Reserved { op: '/', .. } = token {
            let rhs = primary(peekables)?;
            node = Node::Mul { lhs: Box::new(node), rhs: Box::new(rhs) };

            peekables.next();
        } else {
            return Ok(node);
        }
    }

    Ok(node)
}

fn primary(peekables: &mut Peekable<Iter<Token>>) -> Result<Node, String> {
    let token = peekables.next();

    if let Some(Token::Reserved { op: '(', .. }) = token {
        let expr = expr(peekables);
        println!("expr at primary: {:?}", expr);
        match peekables.next() {
            Some(Token::Reserved { op: ')', .. }) =>  { return expr; },
            Some(x) => { println!("x: {:?}", x); return Err("fail primary".to_string()); },
            _ => { return Err("fail primary".to_string()); }
        };
    } else if let Some(Token::Num { val, .. }) = token {
        return Ok(Node::Num { val: *val })
    } else {
        return Err("fail primary".to_string());
    }
}

#[test]
fn parse_test() {
    let input = vec![
        Token::Num { val: 1, t_str: "1".to_string() },
        Token::Reserved { op: '+', t_str: "+".to_string() },
        Token::Num { val: 2, t_str: "2".to_string() },
        Token::Reserved { op: '*', t_str: "+".to_string() },
        Token::Num { val: 3, t_str: "3".to_string() },
        Token::Reserved { op: '-', t_str: "-".to_string() },
        Token::Num { val: 20, t_str: "20".to_string() },
        Token::Eof
    ];

    let result = parse(input);

    println!("{:?}", result);
}
