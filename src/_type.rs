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

    pub fn is_integer(&self) -> bool {
        match self {
            Type::Int | Type::Char => true,
            _ => false
        }
    }

    pub fn has_base(&self) -> bool {
        match self {
            Type::Ptr { .. } | Type::Array { .. } => true,
            _ => false
        }
    }
}

