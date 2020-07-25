extern crate rust_chibicc;
use rust_chibicc::lexer::tokenize;
use rust_chibicc::token::Token;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        return
    }


    let arg1 = args.get(1)
        .expect("第一引数が取得できませんでした");

    // https://stackoverflow.com/questions/54056268/temporary-value-is-freed-at-the-end-of-this-statement
    // tokenize(&mut arg1.chars().peekable()) <- Vec<Token>
    //     .iter() <- Iter<'_, Token>
    //     .peekable() <- Peekable<Token>
    // Vec<Token>がfreeされるので，一旦変数束縛しないといけない
    // https://doc.rust-jp.rs/book/second-edition/ch04-02-references-and-borrowing.html#a%E5%AE%99%E3%81%AB%E6%B5%AE%E3%81%84%E3%81%9F%E5%8F%82%E7%85%A7
    // let tokens = tokenize(&mut arg1.chars().peekable()).iter().peekable();
    let tokens = tokenize(&mut arg1.chars().peekable());
    let mut peekable_tokens = tokens.iter().peekable();

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    if let Some(Token::Num { val, .. }) = peekable_tokens.peek() {
        println!("  mov rax, {}", *val);
        peekable_tokens.next();
    }

    while let Some(tk) = peekable_tokens.peek() {
        match tk {
            Token::Reserved{ op, ..} => {
                peekable_tokens.next();
                if *op == '+' {
                    match peekable_tokens.next().unwrap() {
                        Token::Num { val, .. } => {
                            println!("  add rax, {}", *val);
                        },
                        tk @ _ => {
                            eprintln!("数字ではありません: {:?}", tk);
                            return
                        }
                    };
                } else if *op == '-' {
                    match peekable_tokens.next().unwrap() {
                        Token::Num { val, .. } => {
                            println!("  sub rax, {}", *val);
                        },
                        tk @ _ => {
                            eprintln!("数字ではありません: {:?}", tk);
                            return
                        }
                    };
                }
            },
            _ => {
                break;
            }
        }
    }

    println!("  ret");
    return
}
