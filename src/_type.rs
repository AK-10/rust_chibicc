use std::rc::Rc;

#[derive(PartialEq, Debug, Clone)]
pub enum Type {
    Int,
    Ptr {
        base: Rc<Type>
    },
    Array {
        base: Rc<Type>,
        len: usize
    },
    Char
}

impl Type {
    pub fn size(&self) -> usize {
        match self {
            Type::Int => 8,
            Type::Ptr { .. } => 8,
            Type::Array { base, len } => {
                base.size() * len
            },
            Type::Char => 1
        }
    }

    pub fn base_size(&self) -> usize {
        match self {
            Type::Ptr { base } => base.size(),
            Type::Array { base, .. } => base.size(),
            _ => panic!("expect base type, but does not base type")
        }
    }
}

