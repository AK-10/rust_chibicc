extern crate rust_chibicc;
use rust_chibicc::{strtol};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        return
    }

    let chars = &mut args.get(1)
        .expect("第一引数が取得できませんでした")
        .chars()
        .peekable();

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    let first_num = strtol::<usize>(chars).expect("最初のトークンが数字ではないです");
    println!("  mov rax {}", first_num);

    while let Some(ch) = chars.next() {
        match ch {
            '+' => {
                let num = strtol::<usize>(chars).expect("parseできませんでした");
                println!("  add rax {}", num);
            },
            '-' => {
                let num = strtol::<usize>(chars).expect("parseできませんでした");
                println!("  sub rax {}", num);
            },
            _ => {
                eprintln!("予期しない文字です: {}", ch);
                return
            }
        }
    }

    println!("  ret");
}
