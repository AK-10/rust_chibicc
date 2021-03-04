// extern crate rust_chibicc;
use rust_chibicc::tokenizer::Tokenizer;
use rust_chibicc::parser::Parser;
use rust_chibicc::codegen::CodeGenerator;
use std::env;

use std::fs;

fn read_file(path: impl Into<String>) -> String {
    let path_str = path.into();
    match fs::read_to_string(&path_str) {
        Ok(content) => content,
        Err(e) => {
            let msg = format!("cannot read {}, reason: {}", &path_str, e);
            panic!(msg);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        return
    }

    let arg1 = args.get(1)
        .expect("第一引数が取得できませんでした");

    let user_input = read_file(arg1);
    let tokens = match Tokenizer::new(&*user_input).tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("{}", e);
            return
        }
    };

    let mut parser = Parser::new(&tokens);
    let parsed = parser.parse();

    match parsed {
        Err(msg) => { eprintln!("{}", msg); },
        Ok(ast) => { CodeGenerator::new(&ast).codegen() }
    };
}
