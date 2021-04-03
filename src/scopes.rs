use crate::_type::Type;
use crate::program::Var;

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub struct TagScope {
    pub name: Rc<String>,
    pub ty: Rc<Type>
}

impl TagScope {
    pub fn new(name: Rc<String>, ty: Rc<Type>) -> Self {
        Self { name, ty }
    }
}

#[derive(Clone)]
pub enum VarOrTypeDef {
    Var(Rc<RefCell<Var>>),
    TypeDef(Rc<Type>)
}

// Scope for local variables, global variables or typedefs
// get var by name, or get typedef by name
//
// [example]
// int x = 10; x; typedef struct { int y; char z };
// name | target
// "x" -> Var { name: x }
// "t" -> Type::Struct { member: [y(int), z(char)] }

#[derive(Clone)]
pub struct VarScope {
    pub name: Rc<String>,
    pub target: VarOrTypeDef
}

impl VarScope {
    pub fn new_var(name: &Rc<String>, var: &Rc<RefCell<Var>>) -> Self {
        Self {
            name: Rc::clone(name),
            target: VarOrTypeDef::Var(Rc::clone(var))
        }
    }

    pub fn new_typedef(name: &Rc<String>, ty: &Rc<Type>) -> Self {
        Self {
            name: Rc::clone(name),
            target: VarOrTypeDef::TypeDef(Rc::clone(ty))
        }
    }
}

#[derive(Clone)]
pub struct Scope(pub Vec<VarScope>, pub Vec<TagScope>);

impl Scope {
    pub fn new(var_scope: Vec<VarScope>, tag_scope: Vec<TagScope>) -> Self {
        Self(var_scope, tag_scope)
    }
}
