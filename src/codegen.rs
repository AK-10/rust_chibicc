use crate::node::Node;
use crate::program::Function;

pub fn codegen(func: Function) {
    let mut node_iter = func.nodes.iter();
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    // Prologue
    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, {}", func.stack_size); // 208: 変数26個分(a-z)の領域を確保する 領域の単位は8byte

    while let Some(node) = node_iter.next() {
        gen(node);
    };

    // Epilogue
    // 最後の式の結果がRAXに残っているのでそれが返り値になる
    println!(".L.return:");
    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");

}

fn gen(node: &Node) {
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
        Node::Return { val } => {
            gen(val);

            println!("  pop rax");
            println!("  jmp .L.return");
            return
        }
        Node::ExprStmt { val } => {
            gen(val);
            println!("  add rsp, 8");
            return
        }
        Node::Var { offset, .. } => {
            gen_addr(*offset);
            load();
            return
        }
        Node::Assign { var, val } => {
            // なんとかしたい, 以下ができれば完璧
            // #[derive(Node)]
            // enum Assign { var: Var, val: Expr }
            // #[derive(Node)]
            // enum Var { name: String, offset: i64 }

            if let Node::Var { offset, .. } = **var {
                gen_addr(offset);
                gen(val);

                store();
            }
            return
        }
    };

    println!("  push rax");
}

fn gen_both_side(lhs: &Node, rhs: &Node) {
    gen(lhs);
    gen(rhs);

    println!("  pop rdi");
    println!("  pop rax");
}

fn gen_addr(offset: usize) {
    // lea: アドレスのロード
    println!("  lea rax, [rbp-{}]", offset);
    println!("  push rax");
}

fn load() {
    println!("  pop rax");
    println!("  mov rax, [rax]");
    println!("  push rax");
}

fn store() {
    println!("  pop rdi");
    println!("  pop rax");
    println!("  mov [rax], rdi");
    println!("  push rdi");
}
