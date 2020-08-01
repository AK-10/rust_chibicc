use crate::node::Node;
use crate::token::Token;

pub fn parse(tokens: Vec<Token>) -> Result<Node, String> {
    let mut peekable_tokens = tokens.iter().peekable();
    
    while let Some(token) = peekable_tokens.peek() {
        match token {

        }
}
