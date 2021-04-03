use crate::parser::{ Parser, TYPE_NAMES };
use crate::node::{ Stmt, ExprWrapper, Expr };
use crate::token::Token;
use crate::token::token_type::*;
use crate::program::{ Var, Offset, align_to };
use crate::_type::{ Type, Member };
use crate::scopes::{ TagScope, VarScope, Scope, VarOrTypeDef };

use std::rc::Rc;
use std::cell::RefCell;

impl<'a> Parser<'a> {
    // local変数 -> global変数の順に探す
    pub(in super) fn find_var(&self, name: &String) -> Option<&VarScope> {
        self.var_scope.iter()
            .find(|vsc| { vsc.name.as_str() == *name })
            .map(|vsc| vsc)
    }

    pub(in super) fn find_typedef(&self, tk: &Token) -> Option<Rc<Type>> {
        if let Token::Ident(Ident { name, .. }) = tk {
            self.find_var(name).and_then(|sc| {
                if let VarOrTypeDef::TypeDef(ref ty) = sc.target {
                    Some(Rc::clone(ty))
                } else {
                    None
                }
            })
        } else {
            None
        }
    }

    pub(in super) fn find_tag(&self, tag_name: impl AsRef<String>) -> Option<&TagScope> {
        self.tag_scope.iter()
            .find(|tag| tag.name.as_str() == tag_name.as_ref().as_str())
    }

    pub(in super) fn if_stmt(&mut self) -> Result<Stmt, String> {
        self.peekable.next();

        // primaryだと()なしでも動くようになるが, Cコンパイラではなくなる
        // が面倒なので一旦これで(modern)
        let cond = self.primary()?;
        let then = self.stmt()?;
        let els = match self.peekable.peek() {
            Some(Token::Reserved(Reserved { op, .. })) if op.as_str() == "else" => {
                self.peekable.next();

                Some(self.stmt()?)
            },
            _ => None
        };

        Ok(Stmt::If {
            cond: ExprWrapper::new(cond),
            then: Box::new(then),
            els: els.map(|x| Box::new(x)),
        })
    }

    pub(in super) fn while_stmt(&mut self) -> Result<Stmt, String> {
        self.peekable.next();

        let cond = self.primary()?;
        let then = self.stmt()?;

        Ok(Stmt::While {
            cond: ExprWrapper::new(cond),
            then: Box::new(then)
        })
    }

    pub(in super) fn for_stmt(&mut self) -> Result<Stmt, String> {
        self.peekable.next();

        self.expect_next_symbol("(".to_string())?;

        // 初期化，条件，処理後はない場合がある
        let init = self.expr_stmt().ok();
        self.expect_next_symbol(";".to_string())?;

        let cond = self.expr().ok();
        self.expect_next_symbol(";".to_string())?;

        let inc = self.expr_stmt().ok();
        self.expect_next_symbol(")".to_string())?;

        let then = self.stmt()?;

        Ok(Stmt::For {
            init: Box::new(init),
            cond: cond.map(ExprWrapper::new),
            inc: Box::new(inc),
            then: Box::new(then)
        })
    }

    // variable declaration
    // declaration := basetype ident ("[" num "]")* ("=" expr) ";"
    //              | basetype ";"
    pub(in super) fn declaration(&mut self) -> Result<Stmt, String> {
        let ty = self.base_type()?;
        if let Ok(()) = self.expect_next_symbol(";") {
            return Ok(Stmt::ExprStmt { val: Expr::Null.to_expr_wrapper() })
        }

        match self.peekable.peek() {
            Some(Token::Ident(Ident { name, .. })) => {
                self.peekable.next();
                let ty = self.read_type_suffix(ty)?;

                let expr =
                    if let Err(_) = self.expect_next_reserved("=".to_string()) {
                        // int a; みたいな場合はローカル変数への追加だけ行う. (push rax, 3 みたいなのはしない)
                        let var = self.new_var(name, Rc::clone(&ty), true);
                        self.locals.push(var);
                        self.expect_next_symbol(";".to_string())?;

                        Expr::Null
                    } else {
                        let lhs = self.new_var(name, Rc::clone(&ty), true);
                        self.locals.push(Rc::clone(&lhs));

                        let rhs = self.expr()?;
                        self.expect_next_symbol(";".to_string())?;

                        Expr::Assign { var: Expr::Var(Rc::clone(&lhs)).to_expr_wrapper(), val: rhs.to_expr_wrapper() }
                    };

                Ok(Stmt::ExprStmt { val: ExprWrapper { ty: Rc::clone(&ty), expr: Box::new(expr) } })
            }
            _ => {
                return Err("expect ident, but not found".to_string())
            }
        }
    }

    pub(in super) fn expr_stmt(&mut self) -> Result<Stmt, String> {
        Ok(Stmt::ExprStmt { val: ExprWrapper::new(self.expr()?) })
    }

    // statement expression is a GNU C extension
    // stmt_expr := "(" "{" stmt stmt* "}" ")"
    // 呼び出し側で "(" "{" はすでに消費されている
    pub(in super) fn stmt_expr(&mut self) -> Result<Expr, String> {
        let sc = self.enter_scope();

        let mut stmts = Vec::<Stmt>::new();
        while let Err(_) = self.expect_next_symbol("}".to_string()) {
            stmts.push(self.stmt()?);
        }
        self.expect_next_symbol(")".to_string())?;

        self.leave_scope(sc);

        match stmts.last_mut(){
            // 最後のExprStmtをPureExprに変換する
            // StmtExprとして扱うと誤ったスタック操作になるため
            Some(last) => {
                if let Stmt::ExprStmt { val } = last {
                    *last = Stmt::PureExpr(val.clone());
                    Ok(Expr::StmtExpr(stmts))
                } else {
                    Err("stmt expr returning void is not supported".to_string())
                }
            }
            _ => Err("stmt expr returning void is not supported".to_string())
        }
    }

    pub(in super) fn expect_next_symbol(&mut self, word: impl Into<String>) -> Result<(), String> {
        let tk = self.peekable.peek();
        let expected = word.into();

        match tk {
            Some(Token::Symbol(Symbol { sym, .. })) if sym.as_str() == expected => {
                self.peekable.next();
                Ok(())
            },
            _ => {
                let msg = format!("expect symbol {}, but found {:?}", expected, tk);
                Err(msg)
            }
        }
    }

    pub(in super) fn expect_next_reserved(&mut self, word: impl Into<String>) -> Result<(), String> {
        let tk = self.peekable.peek();
        let expected = word.into();

        match tk {
            Some(Token::Reserved(Reserved { op, .. })) if op.as_str() == expected => {
                self.peekable.next();
                Ok(())
            },
            _ => {
                let msg = format!("expect reserved {}, but found {:?}", expected, tk);
                Err(msg)
            }
        }
    }

    // 関数呼び出しにおける引数をparseする
    pub(in super) fn parse_args(&mut self) -> Result<Vec<ExprWrapper>, String> {
        // no arguments
        if let Ok(_) = self.expect_next_symbol(")") {
            return Ok(vec![])
        }
        // 最初の一個だけ読んでおく
        let mut args = vec![ExprWrapper::new(self.expr()?)];
        while let Ok(_) = self.expect_next_symbol(",") {
            args.push(ExprWrapper::new(self.expr()?));
        }

        self.expect_next_symbol(")")?;

        Ok(args)
    }

    // 関数宣言における引数をparseする
    // params := ident ("," ident)*
    pub(in super) fn parse_func_params(&mut self) -> Result<Vec<Rc<RefCell<Var>>>, String> {
        self.expect_next_symbol("(".to_string())?;

        let mut params = Vec::<Rc<RefCell<Var>>>::new();
        if self.expect_next_symbol(")".to_string()).is_ok() {
            return Ok(params)
        }
        let ty = self.base_type()?;
        let first_arg = self.peekable.peek();

        if let Some(Token::Ident(Ident { name, .. })) = first_arg {
            self.peekable.next();

            params.push(self.new_var(name, ty, true));
        } else {
            return Err("token not found".to_string())
        }

        while let Ok(_) = self.expect_next_symbol(",".to_string()) {
            let ty = self.base_type()?;
            match self.peekable.peek() {
                Some(Token::Ident(Ident { name, .. })) => {
                    self.peekable.next();

                    params.push(self.new_var(name, ty, true));
                },
                Some(token) => {
                    return Err(format!("expect ident, result: {:?}", token))
                }
                _ => {
                    return Err("token not found".to_string())
                }
            }
        }

        self.expect_next_symbol(")".to_string())?;

        Ok(params)
    }

    // base_type = ("char" | "int" | struct-decl) "*"*
    pub(in super) fn base_type(&mut self) -> Result<Rc<Type>, String> {
        if !self.is_typename() {
            return Err("typename expected".to_string())
        }

        let mut ty = if let Ok(_) = self.expect_next_reserved("int") {
            Type::Int
        } else if let Ok(_) = self.expect_next_reserved("char") {
            Type::Char
        } else if let Ok(_) = self.expect_next_reserved("struct") {
            self.struct_decl()?.as_ref().clone()
        } else {
            let tk = self.expect_next_ident()?;
            self.find_typedef(&tk).ok_or(format!("{:?} is not type", tk.tk_str()))?.as_ref().clone()
        };

        while let Some(Token::Reserved(Reserved { op, .. })) = self.peekable.peek() {
            if op.as_str() == "*" {
                ty = Type::Ptr { base: Rc::new(ty) };
                self.peekable.next();
            } else {
                break
            }
        }

        Ok(Rc::new(ty))
    }

    pub(in super) fn new_var(&mut self, name: &String, ty: Rc<Type>, is_local: bool) -> Rc<RefCell<Var>> {
        let var = Rc::new(
            RefCell::new(
                Var {
                    name: name.to_string(),
                    offset: Offset::Unset,
                    ty: Rc::clone(&ty),
                    is_local,
                    contents: None
                }
            )
        );

        self.push_scope_with_var(&Rc::new(name.to_string()), &var);

        var
    }

    pub(in super) fn new_gvar_with_contents(&mut self, name: &String, ty: Rc<Type>, contents: &Vec<u8>) -> Rc<RefCell<Var>> {
        let var = Rc::new(
            RefCell::new(
                Var {
                    name: name.to_string(),
                    offset: Offset::Unset,
                    ty: Rc::clone(&ty),
                    is_local: false,
                    contents: Some(contents.clone())
                }
            )
        );

        self.push_scope_with_var(&Rc::new(name.to_string()), &var);

        var
    }

    pub(in super) fn global_var(&mut self) -> Result<Rc<RefCell<Var>>, String> {
        let base_ty = self.base_type()?;
        let ident = self.expect_next_ident()?;

        let ty = self.read_type_suffix(base_ty)?;
        self.expect_next_symbol(";")?;

        Ok(self.new_var(ident.tk_str().as_ref(), ty, false))
    }

    pub(in super) fn read_type_suffix(&mut self, base: Rc<Type>) -> Result<Rc<Type>, String> {
        match self.expect_next_symbol("[".to_string()) {
            Ok(_) => {
                match self.peekable.next() {
                    Some(Token::Num(Num { val, .. })) => {
                        if let Err(e) = self.expect_next_symbol("]".to_string()) {
                            Err(e)
                        } else {
                            let nested_base = self.read_type_suffix(base)?;
                            Ok(Rc::new(Type::Array { base: nested_base, len: *val as usize }))
                        }
                    },
                    _ => {
                        Err("expect num after [".to_string())
                    }
                }
            }
            Err(_) => Ok(base)
        }
    }

    pub(in super) fn new_add(lhs: ExprWrapper, rhs: ExprWrapper) -> Result<Expr, String> {
        match (lhs.ty.as_ref(), rhs.ty.as_ref()) {
            (l, r) if l.is_integer() && r.is_integer() => {
                Ok(Expr::Add { lhs, rhs })
            },
            (l, r) if l.has_base() && r.is_integer() => {
                Ok(Expr::PtrAdd { lhs, rhs })
            },
            (l, r) if l.is_integer() && r.has_base() => {
                Ok(Expr::PtrAdd { lhs: rhs, rhs: lhs })
            },
            (_, _) => {
                return Err("invalid operands at +".to_string());
            }
        }
    }

    pub(in super) fn new_sub(lhs: ExprWrapper, rhs: ExprWrapper) -> Result<Expr, String> {
       match (lhs.ty.as_ref(), rhs.ty.as_ref()) {
            (l, r) if l.is_integer() && r.is_integer() => {
                Ok(Expr::Sub { lhs, rhs })
            },
            (l, r) if l.has_base() && r.is_integer() => {
                Ok(Expr::PtrSub { lhs, rhs })
            },
            (l, r) if l.has_base() && r.has_base() => {
                Ok(Expr::PtrDiff { lhs, rhs })
            },
            (_, _) => {
                return Err("invalid operands at -".to_string());
            }
        }
    }

    pub(in super) fn expect_next_ident(&mut self) -> Result<Token, String> {
        if let Some(Token::Ident { .. }) = self.peekable.peek() {
            let tk = self.peekable.next().unwrap();
            Ok(tk.clone())
        } else {
            Err("expect identifier".to_string())
        }
    }

    // function := type ident "(" params* ")"
    // gvar := type ident ("=" expr ";")
    pub(in super) fn is_function(&mut self) -> bool {
        let pos = self.peekable.current_position();

        if !self.base_type().is_ok() {
            let _ = self.peekable.back_to(pos);
            return false
        };

        if !self.expect_next_ident().is_ok() {
            let _ = self.peekable.back_to(pos);
            return false
        }

        let is_fn = self.expect_next_symbol("(".to_string()).is_ok();
        let _ = self.peekable.back_to(pos);

        is_fn
    }

    pub(in super) fn is_typename(&self) -> bool {
        self.peekable.peek().map(|tk| {
            if let Token::Reserved(Reserved { op, .. }) = tk {
                TYPE_NAMES.contains(&op.as_str())
            } else {
                self.find_typedef(tk).is_some()
            }
        }).unwrap_or(false)
    }

    pub(in super) fn new_label(&mut self) -> String {
        let label = format!(".L.data.{}", self.label_cnt);
        self.label_cnt += 1;

        return label;
    }

    // struct-decl := "struct" ident
    // struct-decl := "struct" ident? "{" struct-member "}"
    //              | struct ident
    //              | struct ident { .. }
    //              | struct {}
    pub(in super) fn struct_decl(&mut self) -> Result<Rc<Type>, String> {
        // read a struct tag.
        let tag = self.expect_next_ident().ok();

        let lbrace = self.expect_next_symbol("{").ok();
        match (&tag, lbrace) {
            (Some(t), None) => {
                let sc = self.find_tag(t.tk_str());

                return sc
                    .map(|scope_tag| scope_tag.ty.clone())
                    .ok_or("unknown struct type".to_string())
            },
            _ => {}
        }

        let mut members = Vec::<Member>::new();
        let mut offset = 0;
        let mut align = 0;

        while let Err(_) = self.expect_next_symbol("}") {
            let mut member = self.struct_member()?;
            offset = align_to(offset, member.ty.align());
            member.offset = Offset::Value(offset);
            // offsetのインクリメントとmembers.pushが逆の場合,pushが走った時点でmemberの所有権はmembersにあるためエラーになる
            offset += member.ty.size();

            if align < member.ty.align() {
                align = member.ty.align();
            }

            members.push(member);
        }

        let ty = Rc::new(
            Type::Struct {
               members,
               size: align_to(offset, align),
               align
            }
        );

        if let Some(t) = tag {
            self.push_tag_scope(&t, Rc::clone(&ty));
        }

        Ok(ty)
    }

    //  struct-member := basetype ident ("[" num "]") ";"
    pub(in super) fn struct_member(&mut self) -> Result<Member, String> {
        let mut ty = self.base_type()?;

        let ident = self.expect_next_ident()?;
        ty = self.read_type_suffix(ty)?;

        let _ = self.expect_next_symbol(";")?;

        Ok(Member::new(ty, ident.tk_str().as_str()))
    }

    // TODO: AsRef<Type> Structに変えたい
    // pub(in super) fn find_member(&mut self, node: impl AsRef<Struct>, name: impl Into<String>) -> Option<Member> {
    //     node.members.find
    // }

    pub(in super) fn struct_ref(&mut self, expr: Expr) -> Result<Expr, String> {
        let ty = expr.detect_type();
        if let Type::Struct { members, .. } = ty.as_ref() {
            let ident = self.expect_next_ident()?;
            let name = ident.tk_str();
            // TODO: self.find_memberに置き換える
            let member = members.iter()
                            .find(|mem| mem.name == name.as_str())
                            .ok_or_else(|| format!("no such member: {}", name))?;

            Ok(Expr::Member(expr.to_expr_wrapper(), member.clone()))
        } else {
            Err("not_a struct".to_string())
        }
    }

    // begin a block scope
    pub(in super) fn enter_scope(&self) -> Scope {
        Scope::new(self.var_scope.clone(), self.tag_scope.clone())
    }

    // end a block scope
    pub(in super) fn leave_scope(&mut self, sc: Scope) {
        self.var_scope = sc.0;
        self.tag_scope = sc.1;
    }

    pub(in super) fn push_tag_scope(&mut self, token: &Token, ty: Rc<Type>) {
        let sc = TagScope::new(token.tk_str(), ty);
        self.tag_scope.push(sc);
    }

    pub(in super) fn push_scope_with_var(&mut self, name: &Rc<String>, var: &Rc<RefCell<Var>>) {
        let vsc = VarScope::new_var(name, var);
        self.var_scope.push(vsc);
    }

    pub(in super) fn push_scope_with_typedef(&mut self, name: &Rc<String>, ty: &Rc<Type>) {
        let vsc = VarScope::new_typedef(name, ty);
        self.var_scope.push(vsc);
    }
}
