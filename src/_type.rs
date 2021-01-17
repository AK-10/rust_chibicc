#[derive(PartialEq, Debug, Clone)]
pub enum Type {
    Int,
    Ptr {
        base: Box<Type>
    },
    Array {
        base: Box<Type>,
        len: usize
    }
}

impl Type {
    pub fn size(&self) -> usize {
        match self {
            Type::Int => 8,
            Type::Ptr { .. } => 8,
            Type::Array { base, len } => {
                base.size() * len
            }
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
