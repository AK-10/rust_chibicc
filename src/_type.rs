#[derive(PartialEq, Debug, Clone)]
pub enum Type {
    Int,
    Ptr {
        base: Box<Type>
    },
    // Array {
    //     base: Box<Type>,
    //     size: u64,
    //     len: u64
    // }
}

