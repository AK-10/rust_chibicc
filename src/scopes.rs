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
pub struct Scope(pub Vec<Rc<RefCell<Var>>>, pub Vec<TagScope>);

impl Scope {
    pub fn new(var_scope: Vec<Rc<RefCell<Var>>>, tag_scope: Vec<TagScope>) -> Self {
        Self(var_scope, tag_scope)
    }
}

