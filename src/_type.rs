use crate::program::Offset;

#[derive(PartialEq, Debug, Clone)]
pub struct Member {
    pub ty: Box<Type>,
    pub name: String,
    pub offset: Offset
}

impl Member {
    pub fn new(ty: Box<Type>, name: impl Into<String>) -> Self {
        Self {
            ty,
            name: name.into(),
            offset: Offset::Unset
        }
    }
}

// TODO: Strを追加
// codegenとかでchar * に変換する?
#[derive(PartialEq, Debug, Clone)]
pub enum Type {
    Int,
    Short,
    Long,
    Ptr {
        base: Box<Type>
    },
    Array {
        base: Box<Type>,
        len: usize
    },
    Char,
    Struct {
        members: Vec<Member>,
        align: usize, // alignment sizeはこの値の倍数になる
        size: usize
    },
    Func(Box<Type>),
    Dummy
}

impl Type {
    pub fn size(&self) -> usize {
        match self {
            Type::Int => 4,
            Type::Short => 2,
            Type::Long => 8,
            Type::Ptr { .. } => 8,
            Type::Array { base, len } => base.size() * len,
            Type::Char => 1,
            Type::Struct { size, .. } => *size,
            Type::Func(_) => 1,
            Type::Dummy => 0
        }
    }

    pub fn base_size(&self) -> usize {
        match self {
            Type::Ptr { base } => base.size(),
            Type::Array { base, .. } => base.size(),
            _ => panic!("expect base type, but does not have base type")
        }
    }

    pub fn is_integer(&self) -> bool {
        match self {
            Type::Int
            | Type::Short
            | Type::Long
            | Type::Char => true,
            _ => false
        }
    }

    pub fn has_base(&self) -> bool {
        match self {
            Type::Ptr { .. } | Type::Array { .. } => true,
            _ => false
        }
    }

    pub fn align(&self) -> usize {
        match self {
            Type::Int => 4,
            Type::Short => 2,
            Type::Long => 8,
            Type::Ptr { .. } => 8,
            Type::Array { base, .. } => base.align(),
            Type::Char => 1,
            Type::Struct { align, .. } => *align,
            Type::Func(_) => 1,
            Type::Dummy => 0
        }
    }

    pub fn replace_ptr_to(&mut self, dist: Type) {
        match self {
            Type::Ptr { base, .. } => {
                    base.replace_ptr_to(dist);
            },
            _ => *self = dist
        }
    }
}

