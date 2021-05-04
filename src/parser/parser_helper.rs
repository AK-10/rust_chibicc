use crate::parser::{ Parser, TYPE_NAMES };
use crate::node::{ Stmt, ExprWrapper, Expr };
use crate::token::Token;
use crate::token::token_type::*;
use crate::program::{ Var, Offset, align_to };
use crate::_type::{ Type, Member, TypeCounter };
use crate::scopes::{ TagScope, VarScope, Scope, ScopeElement };

use std::rc::Rc;
use std::cell::RefCell;

impl<'a> Parser<'a> {
    // local変数 -> global変数の順に探す
    pub(in super) fn find_var(&self, name: &String) -> Option<&VarScope> {
        // 参考実装に倣って，スコープへの追加が遅い順に走査する
        self.var_scope.iter().rev()
            .find(|vsc| { vsc.name.as_str() == *name })
            .map(|vsc| vsc)
    }

    pub(in super) fn find_typedef(&self, tk: &Token) -> Option<Box<Type>> {
        if let Token::Ident(Ident { name, .. }) = tk {
            self.find_var(name).and_then(|sc| {
                if let ScopeElement::TypeDef(ref ty) = sc.target {
                    Some(Box::clone(ty))
                } else {
                    None
                }
            })
        } else {
            None
        }
    }

    // TODO: refactor, use `and_then`, `map` etc
    // return 'return type' of a function
    // 3 patterns
    //   - get function -> return Ok(ret_type)
    //   - fail find var -> return Ok(None)
    //   - success find var but not function Err
    pub(in super) fn find_func(&self, name: &String) -> Result<Option<Box<Type>>, String> {
        let func = self.find_var(name);
        match func {
            Some(vsc) => {
                match vsc.target {
                    ScopeElement::Var(ref var) => {
                        match var.borrow().ty.as_ref() {
                            Type::Func(ret_type) => {
                                Ok(Some(Box::clone(ret_type)))
                            },
                            _ => Err(format!("{} is not a function", name))
                        }
                    },
                    _ => Ok(None)
                }
            },
            _ => Ok(None)
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
            cond,
            then: Box::new(then),
            els: els.map(|x| Box::new(x)),
        })
    }

    pub(in super) fn while_stmt(&mut self) -> Result<Stmt, String> {
        self.peekable.next();

        let cond = self.primary()?;
        let then = self.stmt()?;

        Ok(Stmt::While {
            cond,
            then: Box::new(then)
        })
    }

    pub(in super) fn for_stmt(&mut self) -> Result<Stmt, String> {
        self.peekable.next();

        self.expect_next_symbol("(")?;

        // 初期化，条件，処理後はない場合がある
        let init = self.expr_stmt().ok();
        self.expect_next_symbol(";")?;

        let cond = self.expr().ok();
        self.expect_next_symbol(";")?;

        let inc = self.expr_stmt().ok();
        self.expect_next_symbol(")")?;

        let then = self.stmt()?;

        Ok(Stmt::For {
            init: Box::new(init),
            cond,
            inc: Box::new(inc),
            then: Box::new(then)
        })
    }

    // variable declaration
    // declaration := basetype declarator type-suffix ("=" expr)? ";"
    //              | basetype ";"
    pub(in super) fn declaration(&mut self) -> Result<Stmt, String> {
        let is_typedef = &mut Some(false);
        let mut ty = self.base_type(is_typedef)?;

        if let Ok(()) = self.expect_next_symbol(";") {
            return Ok(Stmt::ExprStmt { val: Expr::Null.to_expr_wrapper() })
        }

        let name = &mut String::new();

        ty = self.declarator(&mut ty, name)?;
        ty = self.read_type_suffix(ty)?;

        if let Some(true) = is_typedef {
            self.expect_next_symbol(";")?;
            self.push_scope_with_typedef(&Rc::new(name.to_string()), &ty);

            return Ok(Stmt::ExprStmt {
                val: ExprWrapper::new(Expr::Null)
            })
        }

        if let Type::Void = ty.as_ref() {
            return Err("variable declared void".to_string())
        }

        let var = self.new_var(name, Box::clone(&ty), true);

        let expr =
            if let Err(_) = self.expect_next_reserved("=") {
                self.locals.push(var);
                self.expect_next_symbol(";".to_string())?;

                Expr::Null
            } else {
                self.locals.push(Rc::clone(&var));

                let rhs = self.expr()?;
                self.expect_next_symbol(";".to_string())?;

                Expr::Assign { var: Expr::Var(var).to_expr_wrapper(), val: rhs }
            };

        Ok(Stmt::ExprStmt { val: ExprWrapper { ty: Box::clone(&ty), expr: Box::new(expr) } })
    }

    pub(in super) fn expr_stmt(&mut self) -> Result<Stmt, String> {
        Ok(Stmt::ExprStmt { val: self.expr()? })
    }

    // statement expression is a GNU C extension
    // stmt_expr := "(" "{" stmt stmt* "}" ")"
    // 呼び出し側で "(" "{" はすでに消費されている
    pub(in super) fn stmt_expr(&mut self) -> Result<ExprWrapper, String> {
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
                    Ok(Expr::StmtExpr(stmts).to_expr_wrapper())
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
        let mut args = vec![self.expr()?];
        while let Ok(_) = self.expect_next_symbol(",") {
            args.push(self.expr()?);
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
        let ty = &mut self.base_type(&mut None)?;
        let name = &mut String::new();

        if let Ok(_) = self.declarator(ty, name) {
            params.push(self.new_var(name, Box::clone(&ty), true));
        } else {
            return Err("token not found".to_string())
        }

        while let Ok(_) = self.expect_next_symbol(",".to_string()) {
            let ty = &mut self.base_type(&mut None)?;
            let name = &mut String::new();
            match self.declarator(ty, name) {
                Ok(_) => {
                    params.push(self.new_var(name, Box::clone(&ty), true));
                }
                _ => {
                    return Err("error occured at parse_func_params".to_string())
                }
            }
        }

        self.expect_next_symbol(")".to_string())?;

        Ok(params)
    }

    // base-type = buildin-type | struct-decl | typedef-name
    // builtin-type = "void" | "char" | "_Bool" | "int" | "short" | "long" | "long" "long"
    //
    // Note that "typedef" can appear anywhere in a basetype.
    // "int" can appear anywhere if type is short, long or long long
    pub(in super) fn base_type(&mut self, is_typedef: &mut Option<bool>) -> Result<Box<Type>, String> {
        if !self.is_typename() {
            return Err("typename expected".to_string())
        }

        let mut ty = Box::new(Type::Int);
        let mut counter = 0;

        if let Some(true) = is_typedef {
            *is_typedef = Some(false);
        }

        while let (true, Some(tok)) = (self.is_typename(), self.peekable.peek()) {
            let tk_str = tok.tk_str();
            // handle storage class specifiers
            if tk_str.as_str() == "typedef" {
                // if let Some(false) = is_typedef {
                //     return Err("invalid storage class specifier".to_string())
                // }
                *is_typedef = Some(true);
                self.peekable.next();
                continue
            }

            if !["void", "_Bool", "char", "short", "int", "long"].contains(&tk_str.as_str()) {
                if counter > 0 {
                    break
                }

                if tk_str.as_str() == "struct" {
                    ty = self.struct_decl()?;
                } else {
                    ty = self.find_typedef(tok).unwrap();
                    self.peekable.next();
                }

                if counter <= 0 {
                    counter = 1 << 12; //Other.value
                }
                continue
            }

            counter += TypeCounter::new_from(tk_str.as_str()).value();

            ty = Box::new(Type::new_from(&counter)?);
            self.peekable.next();
        }
        Ok(ty)
    }

    // 😵
    // this function is hard for me.
    // original is https://github.com/rui314/chibicc/commit/d51097dc0f7049e3e1fd00f9021e95686ecfddf3
    pub(in super) fn declarator(&mut self, ty: &mut Box<Type>, name: &mut String) -> Result<Box<Type>, String> {
        while let Some(Token::Reserved(Reserved { op, .. })) = self.peekable.peek() {
            if op.as_str() == "*" {
                *ty = Box::new(Type::Ptr { base: Box::clone(&ty) });
                self.peekable.next();
            } else {
                break
            }
        }

        if let Ok(_) = self.expect_next_symbol("(") {
            let mut dummy = Box::new(Type::Dummy);
            dummy = self.declarator(&mut dummy, name)?;

            self.expect_next_symbol(")")?;

            dummy.replace_ptr_to(*self.read_type_suffix(Box::clone(&ty))?);

            return Ok(Box::clone(&dummy))
        }

        let tk = self.expect_next_ident()?;
        *name = tk.tk_str().to_string();

        self.read_type_suffix(Box::clone(&ty))
    }

    // abstract-declarator := "*"* ("(" abstract-declarator ")")? type-suffix
    // example:
    // sizeof(int **):
    //   ty at start of function -> int
    //   ty at end of while      -> int**
    //   return                  -> int**
    // sizeof(int*[4]):
    //   ty at start of function -> int
    //   ty at end of while      -> int*
    //   type_suffix             -> {ty}[4]
    //   return                  -> int*[4]
    //
    // sizeof(int(*)[4]):
    //   ty at start of function   -> int
    //   ty at end of while        -> int
    //   inner abstract-declarator -> int*
    //   type_suffix               -> {abstract-declarator}[4]
    //   return                    -> int*[4]
    pub(in super) fn abstract_declarator(&mut self, ty: &mut Box<Type>) -> Result<Box<Type>, String> {
        while let Some(Token::Reserved(Reserved { op, .. })) = self.peekable.peek() {
            if op.as_str() == "*" {
                *ty = Box::new(Type::Ptr { base: Box::clone(&ty) });
                self.peekable.next();
            } else {
                break
            }
        }

        if let Ok(_) = self.expect_next_symbol("(") {
            let mut dummy = Box::new(Type::Dummy);
            dummy = self.abstract_declarator(&mut dummy)?;

            self.expect_next_symbol(")")?;
            dummy.replace_ptr_to(*self.read_type_suffix(Box::clone(&ty))?);

            return Ok(dummy)
        }

        self.read_type_suffix(Box::clone(&ty))
    }

    pub(in super) fn new_var(&mut self, name: &String, ty: Box<Type>, is_local: bool) -> Rc<RefCell<Var>> {
        let var = Rc::new(
            RefCell::new(
                Var {
                    name: name.to_string(),
                    offset: Offset::Unset,
                    ty: Box::clone(&ty),
                    is_local,
                    contents: None
                }
            )
        );

        self.push_scope_with_var(&Rc::new(name.to_string()), &var);

        var
    }

    pub(in super) fn new_gvar(&mut self, name: &String, ty: Box<Type>, contents: Option<Vec<u8>>, emit: bool) -> Rc<RefCell<Var>> {
        let var = Rc::new(
            RefCell::new(
                Var {
                    name: name.to_string(),
                    offset: Offset::Unset,
                    ty: Box::clone(&ty),
                    is_local: false,
                    contents
                }
            )
        );

        if emit {
            self.globals.push(Rc::clone(&var))
        }

        self.push_scope_with_var(&Rc::new(name.to_string()), &var);

        var
    }

    // global-var := basetype declarator type-suffix ";"
    pub(in super) fn global_var(&mut self) -> Result<(), String> {
        let is_typedef = &mut Some(false);
        let mut base_ty = self.base_type(is_typedef)?;
        let name = &mut String::new();
        let base_ty = self.declarator(&mut base_ty, name)?;

        let ty = self.read_type_suffix(base_ty)?;
        self.expect_next_symbol(";")?;

        if let Some(true) = is_typedef {
            self.push_scope_with_typedef(&Rc::new(name.to_string()), &ty);
        } else {
            self.new_gvar(name, ty, None, true);
        }

        Ok(())
    }

    pub(in super) fn read_type_suffix(&mut self, base: Box<Type>) -> Result<Box<Type>, String> {
        match self.expect_next_symbol("[".to_string()) {
            Ok(_) => {
                match self.peekable.next() {
                    Some(Token::Num(Num { val, .. })) => {
                        if let Err(e) = self.expect_next_symbol("]".to_string()) {
                            Err(e)
                        } else {
                            let nested_base = self.read_type_suffix(base)?;
                            Ok(Box::new(Type::Array { base: nested_base, len: *val as usize }))
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

    // type-name := base-type abstract-declarator type-suffix
    pub(in super) fn type_name(&mut self) -> Result<Box<Type>, String> {
        let mut ty = self.base_type(&mut None)?;
        ty = self.abstract_declarator(&mut ty)?;

        self.read_type_suffix(ty)
    }

    pub(in super) fn new_add(lhs: ExprWrapper, rhs: ExprWrapper) -> Result<ExprWrapper, String> {
        match (lhs.ty.as_ref(), rhs.ty.as_ref()) {
            (l, r) if l.is_integer() && r.is_integer() => {
                Ok(Expr::Add { lhs, rhs }.to_expr_wrapper())
            },
            (l, r) if l.has_base() && r.is_integer() => {
                Ok(Expr::PtrAdd { lhs, rhs }.to_expr_wrapper())
            },
            (l, r) if l.is_integer() && r.has_base() => {
                Ok(Expr::PtrAdd { lhs: rhs, rhs: lhs }.to_expr_wrapper())
            },
            (_, _) => {
                return Err("invalid operands at +".to_string());
            }
        }
    }

    pub(in super) fn new_sub(lhs: ExprWrapper, rhs: ExprWrapper) -> Result<ExprWrapper, String> {
       match (lhs.ty.as_ref(), rhs.ty.as_ref()) {
            (l, r) if l.is_integer() && r.is_integer() => {
                Ok(Expr::Sub { lhs, rhs }.to_expr_wrapper())
            },
            (l, r) if l.has_base() && r.is_integer() => {
                Ok(Expr::PtrSub { lhs, rhs }.to_expr_wrapper())
            },
            (l, r) if l.has_base() && r.has_base() => {
                Ok(Expr::PtrDiff { lhs, rhs }.to_expr_wrapper())
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

        let is_typedef = &mut Some(false);
        let base = &mut if let Ok(ty) = self.base_type(is_typedef) {
            ty
        } else {
            let _ = self.peekable.back_to(pos);
            return false
        };
        let name = &mut String::new();

        match self.declarator(base, name) {
            Err(_) => {
                let _ = self.peekable.back_to(pos);
                return false
            },
            _ => {}
        };

        let is_fn = !name.is_empty() && self.expect_next_symbol("(".to_string()).is_ok();
        let _ = self.peekable.back_to(pos);

        is_fn
    }

    pub(in super) fn is_typename(&self) -> bool {
        self.peekable.peek().map(|tk| {
            if let Token::Reserved(Reserved { op, .. }) = tk {
                TYPE_NAMES.contains(&op.as_str()) || op.as_str() == "typedef"
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
    pub(in super) fn struct_decl(&mut self) -> Result<Box<Type>, String> {
        self.peekable.next();
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

        let ty = Box::new(
            Type::Struct {
               members,
               size: align_to(offset, align),
               align
            }
        );

        if let Some(t) = tag {
            self.push_tag_scope(&t, Box::clone(&ty));
        }

        Ok(ty)
    }

    //  struct-member := basetype ident ("[" num "]") ";"
    pub(in super) fn struct_member(&mut self) -> Result<Member, String> {
        let mut ty = self.base_type(&mut None)?;
        let name = &mut String::new();

        ty = self.declarator(&mut ty, name)?;
        let ty_with_suffix = &mut self.read_type_suffix(Box::clone(&ty))?;

        let _ = self.expect_next_symbol(";")?;

        Ok(Member::new(Box::clone(&ty_with_suffix), name.as_str()))
    }

    pub(in super) fn struct_ref(&mut self, expr_wrapper: ExprWrapper) -> Result<ExprWrapper, String> {
        let ty = expr_wrapper.expr.detect_type();
        if let Type::Struct { members, .. } = ty.as_ref() {
            let ident = self.expect_next_ident()?;
            let name = ident.tk_str();
            let member = members.iter()
                            .find(|mem| mem.name == name.as_str())
                            .ok_or_else(|| format!("no such member: {}", name))?;

            Ok(Expr::Member(expr_wrapper, member.clone()).to_expr_wrapper())
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

    pub(in super) fn push_tag_scope(&mut self, token: &Token, ty: Box<Type>) {
        let sc = TagScope::new(token.tk_str(), ty);
        self.tag_scope.push(sc);
    }

    pub(in super) fn push_scope_with_var(&mut self, name: &Rc<String>, var: &Rc<RefCell<Var>>) {
        let vsc = VarScope::new_var(name, var);
        self.var_scope.push(vsc);
    }

    pub(in super) fn push_scope_with_typedef(&mut self, name: &Rc<String>, ty: &Box<Type>) {
        let vsc = VarScope::new_typedef(name, ty);
        self.var_scope.push(vsc);
    }
}
