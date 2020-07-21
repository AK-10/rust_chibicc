use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        return
    }

    let arg1 = args.get(1)
        .and_then(|item| item.parse::<i64>().ok())
        .ok_or("第1引数は数字を与えてください");

    match arg1 {
        Ok(num) => {
            println!(".intel_syntax noprefix");
            println!(".globl main");
            println!("main:");
            println!("  mov rax, {}", num);
            println!("  ret");
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}
