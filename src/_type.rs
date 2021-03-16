use crate::program::Offset;

use std::rc::Rc;

#[derive(PartialEq, Debug, Clone)]
pub struct Member {
    pub ty: Rc<Type>,
    pub name: String,
    pub offset: Offset
}

impl Member {
    pub fn new(ty: Rc<Type>, name: impl Into<String>, offset_value: usize) -> Self {
        Self {
            ty,
            name: name.into(),
            offset: Offset::Value(offset_value)
        }
    }
}

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
    Char,
    Struct {
        members: Vec<Member>,
        align: usize, // alignment sizeはこの値の倍数になる
        size: usize
    }
}

impl Type {
    pub fn size(&self) -> usize {
        match self {
            Self::Int => 8,
            Self::Ptr { .. } => 8,
            Self::Array { base, len } => base.size() * len,
            Self::Char => 1,
            Self::Struct { size, .. } => *size
            // Self::Struct { members, .. } => {
            //     members
            //         .iter()
            //         .fold(0, |acc, member| acc + member.ty.size())
            // }
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

    pub fn align(&self) -> usize {
        match self {
            Type::Int => 8,
            Type::Ptr { .. } => 8,
            Type::Array { base, .. } => base.align(),
            Type::Char => 1,
            Type::Struct { align, .. } => *align
        }
    }
}

