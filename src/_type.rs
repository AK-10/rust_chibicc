#[derive(PartialEq, Debug)]
pub enum Type {
    Int,
    Ptr {
        base: Box<Type>
    }
}
