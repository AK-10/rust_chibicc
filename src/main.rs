extern crate rust_chibicc;
use rust_chibicc::lexer::tokenize;
use rust_chibicc::parser;
use rust_chibicc::codegen::gen;
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
    let tokens = tokenize(arg1.to_string()).expect("compile failed");
    let parsed = parser::parse(tokens);

    match parsed {
        Err(msg) => { eprintln!("{}", msg); },
        Ok(ast) => {
            println!(".intel_syntax noprefix");
            println!(".globl main");
            println!("main:");

            gen(ast);

            println!("  ret");
        }
    };
}
