use crate::node::Stmt;
use crate::_type::Type;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Program {
    pub fns: Vec<Function>,
    pub globals: Vec<Rc<RefCell<Var>>>
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub nodes: Vec<Stmt>,
    pub locals: Vec<Rc<RefCell<Var>>>, // Exprが持つVarと同じものを指したいためヒープにデータを置く
    pub params: Vec<Rc<RefCell<Var>>>,
    pub stack_size: usize
}

impl Function {
    pub fn new(name: String, nodes: Vec<Stmt>, locals: Vec<Rc<RefCell<Var>>>, params: Vec<Rc<RefCell<Var>>>) -> Self {
        let offset = locals.iter().fold(0, |acc, var| acc + var.borrow().ty.size());

        Self {
            name,
            nodes,
            stack_size: Self::align_to(offset, 8),
            locals: Self::calc_offsets(&locals),
            params
        }
    }

    // locals のoffset計算を行う.
    // localsの先頭はparamsが持つ要素のポインタなのでparamsも同時にoffset計算される
    fn calc_offsets(locals: &Vec<Rc<RefCell<Var>>>) -> Vec<Rc<RefCell<Var>>> {
        let mut reversed = locals.clone();
        let mut offset = 0;
        reversed.reverse();

        // reversed.iter.map()はなんかうまく行かない
        reversed.iter().for_each(|v| {
            let mut var = v.borrow_mut();
            offset += var.ty.size();
            var.offset = Offset::Value(offset);
        });

        reversed
    }

    // alignの倍数に調整する
    // !はbitwise not
    // &はbitwise and
    //
    // n -> 17, align -> 8のとき 24になってほしい
    // n + align - 1 -> 17 + 8 - 1 -> 24 -> 00011000
    // !(align - 1) -> !(7) -> !(00000111) -> 11111000
    // (n + align - 1) & !(align - 1) -> 00011000 & 11111000 -> 00011000
    fn align_to(n: usize, align: usize) -> usize {
        (n + align - 1) & !(align - 1)
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
    pub ty: Rc<Type>,
    pub is_local: bool,
     // 構文解析の時点では0
    pub offset: Offset,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Offset {
    Unset,
    Value(usize)
}

impl Offset {
    pub fn value(&self) -> usize {
        match self {
            Offset::Value(i) => *i,
            Offset::Unset => panic!("offset must be set")
        }
    }
}

