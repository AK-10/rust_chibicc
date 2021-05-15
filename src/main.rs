// extern crate rust_chibicc;
use rust_chibicc::tokenizer::Tokenizer;
use rust_chibicc::parser::Parser;
use rust_chibicc::codegen::CodeGenerator;

use std::env;
use std::fs;
//use std::fs::File;
//use std::io::BufReader;
//use std::io::prelude::*;

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

//fn read_file_with_number<'a>(path: impl Into<String>) -> Vec<String> {
//    let path_str = path.into();
//    let f = File::open(path_str);
//    match f {
//        Ok(f) => {
//            let buf = BufReader::new(f);
//            let mut lines = Vec::<String>::new();
//            buf.lines().for_each(|line| {
//                lines.push(line.expect("failed read file"));
//            });
//
//            lines
//        },
//        Err(e) => panic!(e)
//    }
//}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        return
    }

    let filename = args.get(1).expect("expect 1 arguments");

    let user_input = read_file(filename);
    let tokens = match Tokenizer::new(&*user_input).tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("{}:{}", filename, e);
            return
        }
    };

    let mut parser = Parser::new(&tokens);
    let parsed = parser.parse();

    match parsed {
        Err(msg) => { eprintln!("{}:{}", filename, msg); },
        Ok(ast) => { CodeGenerator::new(&ast).codegen() }
    };
}
