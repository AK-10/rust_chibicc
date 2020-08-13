use crate::node::Node;

pub fn gen(nodes: Vec<Node>) {
    let mut node_iter = nodes.iter();

    while let Some(node) = node_iter.next() {
        gen_single(node);
        println!("  pop rax");
    };
}

fn gen_single(node: &Node) {
    match node {
        Node::Add { lhs, rhs } => {
            gen_both_side(lhs, rhs);
            println!("  add rax, rdi");
        },
        Node::Sub { lhs, rhs } => {
            gen_both_side(lhs, rhs);
            println!("  sub rax, rdi");
        },
        Node::Mul { lhs, rhs } => {
            gen_both_side(lhs, rhs);
            println!("  imul rax, rdi");
        },
        Node::Div { lhs, rhs } => {
            gen_both_side(lhs, rhs);

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
            return
        }
        Node::Eq { lhs, rhs } => {
            gen_both_side(lhs, rhs);

            // cmp命令: 二つの引数のレジスタを比較して, フラグレジスタに結果を格納
            // sete命令: 指定のレジスタにフラグレジスタの値を格納. seteであれば==の時1になる
            //           8bitしか書き込めないのでalを指定している
            // movzb命令: movzb dist, srcでsrcをdistに書き込む．またsrcで指定されたbitより上の桁は0埋めする
            // al: raxの下位8bitのエイリアス. alを変更するとraxも変更される
            println!("  cmp rax, rdi");
            println!("  sete al");
            println!("  movzb rax, al");
        }
        Node::Neq { lhs, rhs } => {
            gen_both_side(lhs, rhs);

            println!("  cmp rax, rdi");
            println!("  setne al");
            println!("  movzb rax, al");
        }
        Node::Gt { lhs, rhs } => {
            // setl を使うため，rhs, lhsを逆にする
            gen_both_side(rhs, lhs);

            println!("  cmp rax, rdi");
            println!("  setl al");
            println!("  movzb rax, al");
        }
        Node::Ge { lhs, rhs } => {
            // setle を使うため，rhs, lhsを逆にする
            gen_both_side(rhs, lhs);

            println!("  cmp rax, rdi");
            println!("  setle al");
            println!("  movzb rax, al");
        }
        Node::Lt { lhs, rhs } => {
            gen_both_side(lhs, rhs);

            println!("  cmp rax, rdi");
            println!("  setl al");
            println!("  movzb rax, al");
        }
        Node::Le { lhs, rhs } => {
            gen_both_side(lhs, rhs);

            println!("  cmp rax, rdi");
            println!("  setle al");
            println!("  movzb rax, al");
        }
    };

    println!("  push rax");
}

fn gen_both_side(lhs: &Node, rhs: &Node) {
    gen_single(lhs);
    gen_single(rhs);

    println!("  pop rdi");
    println!("  pop rax");
}
