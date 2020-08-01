pub mod token;
pub mod lexer;
pub mod node;
pub mod parser;

use crate::node::Node;

pub fn gen(node: Node) {
    match node {
        Node::Add { lhs, rhs } => {
            gen_both_side(*lhs, *rhs);
            println!("  add rax, rdi");
        },
        Node::Sub { lhs, rhs } => {
            gen_both_side(*lhs, *rhs);
            println!("  sub rax, rdi");
        },
        Node::Mul { lhs, rhs } => {
            gen_both_side(*lhs, *rhs);
            println!("  imul rax, rdi");
        },
        Node::Div { lhs, rhs } => {
            gen_both_side(*lhs, *rhs);

            // idiv命令は符号あり除算を行う命令
            // rdxとraxをとってそれを合わせたものを128bit整数とみなす
            // それを引数のレジスタの64bit整数で割り，商をrax, 余をrdxにセットする
            // cqo命令を使うと、RAXに入っている64ビットの値を128ビットに伸ばして
            // rdxとraxにセットすることができる
            println!("  cqo");
            println!("  idiv rdi");
        },
        Node::Num { val } => {
            println!("  push {}", val);
        }
    };

    println!("  push rax");
}

fn gen_both_side(lhs: Node, rhs: Node) {
    gen(lhs);
    gen(rhs);

    println!("  pop rdi");
    println!("  pop rax");
}
