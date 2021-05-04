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
    Void,
    Bool,
    Dummy
}

impl Type {
    pub fn new_from<'a>(counter: &'a usize) -> Result<Self, String> {
        match *counter {
            1 => Ok(Type::Void), // void
            4 => Ok(Type::Bool), // bool
            16 => Ok(Type::Char), // char
            64 //short
            | 320 => Ok(Type::Short), // short + long
            256 => Ok(Type::Int), // int
            1024 // long
            | 1280 // long + int
            | 2048 // long + long
            | 2304 => Ok(Type::Long), // long + long + int
            _ => {
                let msg = format!("counter is {}, invalid type", counter);
                return Err(msg)
            }
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Type::Int => 4,
            Type::Short => 2,
            Type::Long => 8,
            Type::Ptr { .. } => 8,
            Type::Array { base, len } => base.size() * len,
            Type::Char => 1,
            Type::Void => 1,
            Type::Bool => 1,
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
            | Type::Char
            | Type::Bool => true,
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
            Type::Void => 1,
            Type::Bool => 1,
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

pub enum TypeCounter {
    Void,
    Bool,
    Char,
    Short,
    Int,
    Long,
    Other,
    Zero
}

impl TypeCounter {
    pub fn new_from<'a>(tk_str: &'a str) -> Self {
        match tk_str {
            "void" => TypeCounter::Void,
            "_Bool" => TypeCounter::Bool,
            "char" => TypeCounter::Char,
            "short" => TypeCounter::Short,
            "int" => TypeCounter::Int,
            "long" => TypeCounter::Long,
            _  => TypeCounter::Zero
        }
    }

    pub fn value(&self) -> usize {
        match self {
            TypeCounter::Void => 1 << 0,
            TypeCounter::Bool => 1 << 2,
            TypeCounter::Char => 1 << 4,
            TypeCounter::Short => 1 << 6,
            TypeCounter::Int => 1 << 8,
            TypeCounter::Long => 1 << 10,
            TypeCounter::Other => 1 << 12,
            TypeCounter::Zero => 0,
        }
    }
}

