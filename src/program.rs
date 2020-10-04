use crate::node::Stmt;

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub nodes: Vec<Stmt>,
    pub locals: Vec<Var>,
    pub stack_size: usize
}

impl Function {
    pub fn new(name: String, nodes: Vec<Stmt>, locals: Vec<Var>) -> Self {
        Self {
            name: name,
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
#[derive(Debug, Clone, PartialEq)]
pub struct Var {
    pub name: String,
    pub offset: usize
}
