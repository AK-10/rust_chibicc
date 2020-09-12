use crate::node::Node;

#[derive(Debug)]
pub struct Function {
    pub nodes: Vec<Node>,
    pub locals: Vec<Var>,
    pub stack_size: usize
}

impl Function {
    pub fn new(nodes: Vec<Node>, locals: Vec<Var>) -> Self {
        Self {
            nodes: nodes,
            stack_size: locals.last().map_or(0, |var| var.offset),
            locals: locals,
        }
    }
}

// 元のコードは以下, lenはname.lenで代用
// struct LVar {
//   LVar *next; // 次の変数かNULL
//   char *name; // 変数の名前
//   int len;    // 名前の長さ
//   int offset; // RBPからのオフセット
// };
#[derive(Debug, Clone)]
pub struct Var {
    pub name: String,
    pub offset: usize
}
