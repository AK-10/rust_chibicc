use crate::node::Node;

#[derive(Debug)]
pub struct Function {
    nodes: Vec<Node>,
    locals: Vec<Var>,
    stack_size: u64
}

// 元のコードは以下, lenはname.lenで代用
// struct LVar {
//   LVar *next; // 次の変数かNULL
//   char *name; // 変数の名前
//   int len;    // 名前の長さ
//   int offset; // RBPからのオフセット
// };
#[derive(Debug)]
pub struct Var {
    name: String,
    offset: i64
}