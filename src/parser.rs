use crate::node::{ Stmt, Expr, ExprWrapper };
use crate::token::{ Token, TokenIter, TokenType };
use crate::program::{ Function, Var, Program };
use crate::_type::Type;
use crate::token::token_type::*;
use crate::scopes::{ TagScope, VarScope, ScopeElement };

use std::rc::Rc;
use std::cell::RefCell;

mod parser_helper;

// 優先順位
// == !=
// < <= > >=
// + -
// * /
// 単項+ 単項-
// ()
const TYPE_NAMES: [&str; 7] = ["int", "short", "long", "char", "struct", "void", "_Bool"];

pub struct Parser<'a> {
    pub input: &'a Vec<Token>,
    peekable: TokenIter<'a>,
    // 関数の引数，関数内で宣言された変数を保持する, 関数のスコープから外れたらリセットする
    // All local variable instances created during parsing are accumelated to this list
    pub locals: Vec<Rc<RefCell<Var>>>,
    // Likewise, global variable are accumulated to this list
    pub globals: Vec<Rc<RefCell<Var>>>,
    // C has two block scopes; one is for variables/typedefs and
    // the other is for struct tags.
    pub var_scope: Vec<VarScope>,
    pub tag_scope: Vec<TagScope>,
    pub label_cnt: usize
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a Vec<Token>) -> Self {
        Self {
            input,
            peekable: TokenIter::new(input),
            locals: Vec::new(),
            globals: Vec::new(),
            var_scope: Vec::new(),
            tag_scope: Vec::new(),
            label_cnt: 0
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        match self.program() {
            Ok(prog) => Ok(prog),
            Err(e) => {
                let msg = self.peekable.peek()
                    .map(|tok| {
                        tok.error_message(&*e)
                    })
                    .unwrap_or("eof detected".to_string());
                Err(msg)
            }
        }
    }

    // program := (global-var | function)*
    fn program(&mut self) -> Result<Program, String> {
        let mut nodes: Vec<Function> = Vec::new();

        while let Some(token) = self.peekable.peek() {
            // eofでbreakしないと，以降の処理でpeek()するので全体としてErrになる(Noneでエラーにするような処理がprimaryにある)
            if let TokenType::Eof = token.token_type {
                break
            }
            if self.is_function() {
                if let Some(f) = self.function()? {
                    nodes.push(f);
                }
            } else {
                self.global_var()?;
            }
        };

        Ok(Program {
            fns: nodes,
            globals: self.globals.clone()
        })
    }

    // function := basetype declarator "(" params? ")" ("{" stmt* "}" | ";")
    // params := param ("," param)*
    // param := basetype declarator type-suffix
    fn function(&mut self) -> Result<Option<Function>, String> {
        self.locals.clear();

        let mut sclass = None;
        let mut ty = self.base_type(&mut sclass)?;
        let name = &mut String::new();

        ty = self.declarator(&mut ty, name)?;

        // add function type to the scope
        self.new_gvar(name, Box::new(Type::Func(ty)), None, false);

        // clone scope for saving current scope
        let sc = self.enter_scope();

        // parse params
        let params = self.read_func_params()?;
        self.locals = params.clone();

        // prototype declaration
        if let Ok(_) = self.expect_next_symbol(";") {
            self.leave_scope(sc);
            return Ok(None)
        }

        // read function body
        self.expect_next_symbol("{".to_string())?;

        let mut nodes = Vec::new();

        while let Err(_) = self.expect_next_symbol("}") {
            nodes.push(self.stmt()?);
        };

        self.leave_scope(sc);

        let locals = self.locals.to_vec();

        // construct function object
        Ok(Some(Function::new(Rc::new(name.to_string()), nodes, locals, params, sclass.map_or(false, |sc| sc.is_static()))))
    }

    // stmt := expr ";"
    //       | "return" expr ";"
    //       | "if" "(" expr ")" stmt ("else" stmt)?
    //       | "while" "(" expr ")" stmt
    //       | "for" "(" (expr? | declaration) ";" expr? ";" expr? ")" stmt
    //       | "{" stmt "}"
    //       | "break" ";"
    //       | "continue" ";"
    //       | "goto" ident ";"
    //       | ident ":" stmt
    //       | declaration
    //       | expr ";"
    fn stmt(&mut self) -> Result<Stmt, String> {
        match self.peekable.peek() {
            Some(tok) => {
                match tok.token_type.tk_str().as_str() {
                    "return" => {
                        self.peekable.next();

                        let expr = self.expr()?;
                        self.expect_next_symbol(";")?;

                        Ok(Stmt::Return { val: expr })
                    }
                    "{" => {
                        self.peekable.next();
                        let mut stmts: Vec<Stmt> = Vec::new();

                        let sc = self.enter_scope();
                        while let Err(_) = self.expect_next_symbol("}".to_string()) {
                            let stmt = self.stmt()?;
                            stmts.push(stmt);
                        }

                        self.leave_scope(sc);

                        Ok(Stmt::Block { stmts })
                    }
                    "if" => {
                        self.if_stmt()
                    }
                    "while" => {
                        self.while_stmt()
                    }
                    "for" => {
                        self.for_stmt()
                    }
                    "break" => {
                        self.peekable.next();
                        self.expect_next_symbol(";")?;
                        return Ok(Stmt::Break)
                    }
                    "continue" => {
                        self.peekable.next();
                        self.expect_next_symbol(";")?;
                        return Ok(Stmt::Continue)
                    }
                    "goto" => {
                        self.peekable.next();
                        let tok = self.expect_next_ident()?;
                        let goto = Stmt::Goto(tok.token_type.tk_str());
                        self.expect_next_symbol(";")?;

                        Ok(goto)
                    }
                    _ => {
                        let pos = self.peekable.current_position();
                        if let Ok(tk) = self.expect_next_ident() {
                            if let Ok(_) = self.expect_next_symbol(":") {
                                let node = Stmt::Label(Box::new(self.stmt()?), tk.token_type.tk_str());
                                return Ok(node)
                            } else {
                                let _ = self.peekable.back_to(pos);
                            }
                        }
                        if self.is_typename() {
                            return self.declaration()
                        }
                        let expr_stmt = self.expr_stmt();
                        self.expect_next_symbol(";")?;

                        expr_stmt
                    }
                }
            }
            _ => {
                Err("token not found".to_string())
            }
        }
    }

    // expr := assign ("," assign)*
    fn expr(&mut self) -> Result<ExprWrapper, String> {
        let mut node = self.assign()?;

        while let Ok(_) = self.expect_next_symbol(",") {
            let lhs = Stmt::ExprStmt { val: node };
            node = Expr::Comma { lhs, rhs: self.assign()? }.to_expr_wrapper();
        }

        Ok(node)
    }

    // assign    := logor (assign-op assign)?
    // assign-op := "=" | "+=" | "-=" | "*=" | "/="
    fn assign(&mut self) -> Result<ExprWrapper, String> {
        let var = self.logor()?;

        if let Ok(_) = self.expect_next_reserved("=") {
            let val = self.expr()?;
            return Ok(Expr::Assign {
                var,
                val
            }.to_expr_wrapper())
        }

        if let Ok(_) = self.expect_next_reserved("*=") {
            let val = self.expr()?;
            return Ok(Expr::MulEq {
                var,
                val
            }.to_expr_wrapper())
        }

        if let Ok(_) = self.expect_next_reserved("/=") {
            let val = self.expr()?;
            return Ok(Expr::DivEq {
                var,
                val
            }.to_expr_wrapper())
        }

        if let Ok(_) = self.expect_next_reserved("+=") {
            let val = self.expr()?;
            if var.ty.has_base() {
                return Ok(Expr::PtrAddEq {
                    var,
                    val
                }.to_expr_wrapper())
            }
            return Ok(Expr::AddEq {
                var,
                val
            }.to_expr_wrapper())
        }

        if let Ok(_) = self.expect_next_reserved("-=") {
            let val = self.expr()?;
            if var.ty.has_base() {
                return Ok(Expr::PtrSubEq {
                    var,
                    val
                }.to_expr_wrapper())
            }
            return Ok(Expr::SubEq {
                var,
                val
            }.to_expr_wrapper())
        }

        Ok(var)
    }

    // logor := logand ("||" logand)*
    fn logor(&mut self) -> Result<ExprWrapper, String> {
        let mut lhs = self.logand()?;
        while let Some(TokenType::Reserved(Reserved { op, .. })) = self.peekable.peek().map(|tok| &tok.token_type) {
            if op.as_ref() == "||" {
                self.peekable.next();
                lhs = Expr::LogOr {
                    lhs,
                    rhs: self.logand()?
                }.to_expr_wrapper();
            } else {
                break
            }
        }

        Ok(lhs)
    }
    // logand := bitor ("&&" bitor)*
    fn logand(&mut self) -> Result<ExprWrapper, String> {
        let mut lhs = self.bitor()?;
        while let Some(TokenType::Reserved(Reserved { op, .. })) = self.peekable.peek().map(|tok| &tok.token_type) {
            if op.as_ref() == "&&" {
                self.peekable.next();
                lhs = Expr::LogAnd {
                    lhs,
                    rhs: self.bitor()?
                }.to_expr_wrapper();
            } else {
                break
            }
        }

        Ok(lhs)
    }

    // bitor := bitxor ("|" bitxor)*
    fn bitor(&mut self) -> Result<ExprWrapper, String> {
        let mut lhs = self.bitxor()?;
        while let Some(TokenType::Reserved(Reserved { op, .. })) = self.peekable.peek().map(|tok| &tok.token_type) {
            if op.as_ref() == "|" {
                self.peekable.next();
                lhs = Expr::BitOr {
                    lhs,
                    rhs: self.bitxor()?
                }.to_expr_wrapper()
            } else {
                break
            }
        }

        Ok(lhs)
    }

    // bitxor := bitand ("^" bitand)*
    fn bitxor(&mut self) -> Result<ExprWrapper, String> {
        let mut lhs = self.bitand()?;
        while let Some(TokenType::Reserved(Reserved { op, .. })) = self.peekable.peek().map(|tok| &tok.token_type) {
            if op.as_ref() == "^" {
                self.peekable.next();
                lhs = Expr::BitXor {
                    lhs,
                    rhs: self.bitand()?
                }.to_expr_wrapper()
            } else {
                break
            }
        }

        Ok(lhs)
    }

    // bitand := equality ("&" equality)*
    fn bitand(&mut self) -> Result<ExprWrapper, String> {
        let mut lhs = self.equality()?;
        while let Some(TokenType::Reserved(Reserved { op, .. })) = self.peekable.peek().map(|tok| &tok.token_type) {
            if op.as_ref() == "&" {
                self.peekable.next();
                lhs = Expr::BitAnd {
                    lhs,
                    rhs: self.equality()?
                }.to_expr_wrapper()
            } else {
                break
            }
        }

        Ok(lhs)
    }

    // equality := relational ("==" relational | "!=" relational)*
    fn equality(&mut self) -> Result<ExprWrapper, String> {
        let mut node = self.relational()?;

        while let Some(token) = self.peekable.peek() {
            match &token.token_type {
                TokenType::Reserved(Reserved { op, .. }) if op.as_str() == "==" => {
                    self.peekable.next();

                    let rhs = self.relational()?;
                    node = Expr::Eq {
                        lhs: node,
                        rhs
                    }.to_expr_wrapper();
                }
                TokenType::Reserved(Reserved { op, .. }) if op.as_str() == "!=" => {
                    self.peekable.next();

                    let rhs = self.relational()?;
                    node = Expr::Neq {
                        lhs: node,
                        rhs
                    }.to_expr_wrapper();
                }
                _ => { return Ok(node); }
            }
        }

        Ok(node)
    }

    // relational := add ("<" add | "<=" add | ">" add | ">=" add)*
    fn relational(&mut self) -> Result<ExprWrapper, String> {
        let mut node = self.add()?;

        while let Some(TokenType::Reserved(Reserved { op, .. })) = self.peekable.peek().map(|tok| &tok.token_type) {
            match op.as_str() {
                "<" => {
                    self.peekable.next();
                    let rhs = self.add()?;

                    node = Expr::Lt {
                        lhs: node,
                        rhs
                    }.to_expr_wrapper();
                },
                "<=" => {
                    self.peekable.next();
                    let rhs = self.add()?;

                    node = Expr::Le {
                        lhs: node,
                        rhs
                    }.to_expr_wrapper();
                },
                ">" => {
                    self.peekable.next();
                    let rhs = self.add()?;

                    node = Expr::Gt {
                        lhs: node,
                        rhs
                    }.to_expr_wrapper();
                },
                ">=" => {
                    self.peekable.next();
                    let rhs = self.add()?;

                    node = Expr::Ge {
                        lhs: node,
                        rhs
                    }.to_expr_wrapper();
                },
                _ => break
            }
        }

        Ok(node)
    }

    // add := mul ("+" | "-")*
    fn add(&mut self) -> Result<ExprWrapper, String> {
        let mut node = self.mul()?;

        while let Some(TokenType::Reserved(Reserved { op, .. })) = self.peekable.peek().map(|tok| &tok.token_type) {
            match op.as_str() {
                "+" => {
                    self.peekable.next();
                    let rhs = self.mul()?;

                    node = Parser::new_add(
                        node,
                        rhs
                    )?;
                },
                "-" => {
                    self.peekable.next();
                    let rhs = self.mul()?;

                    node = Parser::new_sub(
                        node,
                        rhs
                    )?;
                },
                _ => break
            }
        }

        Ok(node)
    }

    // mul := cast ("*" cast | "/" cast)*
    fn mul(&mut self) -> Result<ExprWrapper, String> {
        let mut node = self.cast()?;

        while let Some(TokenType::Reserved(Reserved { op, .. })) = self.peekable.peek().map(|tok| &tok.token_type) {
            match op.as_str() {
                "*" => {
                    self.peekable.next();
                    let rhs = self.cast()?;

                    node = Expr::Mul {
                        lhs: node,
                        rhs
                    }.to_expr_wrapper();
                },
                "/" => {
                    self.peekable.next();
                    let rhs = self.unary()?;

                   node = Expr::Div {
                        lhs: node,
                        rhs
                    }.to_expr_wrapper();
                },
                _ => break
            }
        }

        Ok(node)
    }

    // cast := "(" type-name ")" cast | unary
    fn cast(&mut self) -> Result<ExprWrapper, String> {
        let pos = self.peekable.current_position();

        if let Ok(_) = self.expect_next_symbol("(") {
            if self.is_typename() {
                let ty = self.type_name()?;
                self.expect_next_symbol(")")?;
                return Ok(Expr::Cast(ty, self.cast()?).to_expr_wrapper())
            }
            let _ = self.peekable.back_to(pos);
        }

        self.unary()
    }

    // unary := ("+" | "-" | "*" | "&" | "!" | "~")? cast
    //        | ("++" | "--") unary
    //        | postfix
    fn unary(&mut self) -> Result<ExprWrapper, String> {
        let tk = self.peekable.peek();

        match tk.map(|tok| &tok.token_type) {
            Some(TokenType::Reserved(Reserved { op, .. })) => {
                match op.as_str() {
                    "+" => {
                        self.peekable.next();
                        self.cast()
                    },
                    "-" => {
                        self.peekable.next();
                        let rhs = self.cast()?;
                        Ok(Expr::Sub {
                            lhs: Expr::Num { val: 0 }.to_expr_wrapper(),
                            rhs
                         }.to_expr_wrapper())
                    },
                    "*" => {
                        self.peekable.next();
                        Ok(Expr::Deref { operand: self.cast()? }.to_expr_wrapper())
                    },
                    "&" => {
                        self.peekable.next();
                        Ok(Expr::Addr { operand: self.cast()? }.to_expr_wrapper())
                    },
                    "!" => {
                        self.peekable.next();
                        Ok(Expr::Not(self.cast()?).to_expr_wrapper())
                    }
                    "~" => {
                        self.peekable.next();
                        Ok(Expr::BitNot(self.cast()?).to_expr_wrapper())
                    }
                    "++" => {
                        self.peekable.next();
                        let unary = self.unary()?;
                        if !unary.expr.is_lvalue() {
                            return Err(format!("{:?} is not an lvalue", unary))
                        }
                        Ok(Expr::PreInc(unary).to_expr_wrapper())
                    },
                    "--" => {
                        self.peekable.next();
                        let unary = self.unary()?;
                        if !unary.expr.is_lvalue() {
                            return Err(format!("{:?} is not an lvalue", unary))
                        }
                        Ok(Expr::PreDec(unary).to_expr_wrapper())
                    }
                    _ => self.postfix()
                }
            },
            _ => self.postfix()
        }
    }

    // postfix := primary ("[" expr "]" | "." ident | "->" ident | "++" | "--")*
    fn postfix(&mut self) -> Result<ExprWrapper, String> {
        let mut node = self.primary()?;

        loop {
            if let Ok(_) = self.expect_next_symbol("[") {
                // x[y] is short for *(x + y)
                let expr = self.expr()?;
                let exp = Parser::new_add(
                    node,
                    expr
                )?;

                match self.expect_next_symbol("]".to_string()) {
                    Ok(_) => {
                        node = Expr::Deref { operand: exp }.to_expr_wrapper();
                    },
                    _ => return Err("expect ] after [ expr".to_string())
                }

                continue;
            }

            if let Ok(_) = self.expect_next_symbol(".") {
                node = self.struct_ref(node)?;

                continue;
            }

            if let Ok(_) = self.expect_next_reserved("->") {
                node = Expr::Deref { operand: node }.to_expr_wrapper();
                node = self.struct_ref(node)?;
                continue
            }

            if let Ok(_) = self.expect_next_reserved("++") {
                node = Expr::PostInc(node).to_expr_wrapper();
                continue
            }
            if let Ok(_) = self.expect_next_reserved("--") {
                node = Expr::PostDec(node).to_expr_wrapper();
                continue
            }

            return Ok(node);
        }
    }

    // primary := "(" "{" stmt-expr-tail
    //          | "(" expr ")"
    //          | "sizeof" unary
    //          | ident func-args?
    //          | str
    //          | num
    fn primary(&mut self) -> Result<ExprWrapper, String> {
        let token = self.peekable.peek();

        match token.map(|tok| &tok.token_type) {
            Some(TokenType::Symbol(symbol)) if symbol.sym.as_str() == "(" => {
                self.peekable.next();

                if self.expect_next_symbol("{".to_string()).is_ok() {
                    return self.stmt_expr()
                }

                let expr = self.expr();
                self.expect_next_symbol(")".to_string())?;

                expr
            }
            // sizeof
            Some(TokenType::Reserved(Reserved { op, .. })) if op.as_str() == "sizeof" => {
                self.peekable.next();

                let pos = self.peekable.current_position();
                if let Ok(_) = self.expect_next_symbol("(") {
                    if self.is_typename() {
                        let ty = self.type_name()?;
                        if ty.is_incomplete() {
                            return Err("incomplete type".to_string())
                        }

                        let size = ty.size();
                        self.expect_next_symbol(")")?;

                        return Ok(Expr::Num { val: size as isize }.to_expr_wrapper())
                    };
                    // typeof unaryとして扱うため一つ戻す
                    // "(" expression ")" を扱えるようにしたい
                    // back_toを使わずself.expr()を使っても良さげだが，オリジナルに倣う
                    let _ = self.peekable.back_to(pos);
                }
                // unary at here => "*"* (ident) | "(" expression ")" | num
                let node = self.unary()?;
                if node.ty.is_incomplete() {
                    return Err("incomplete type".to_string())
                }
                let size = node.ty.size();

                Ok(Expr::Num { val: size as isize }.to_expr_wrapper())
            }
            // num
            Some(TokenType::Num(Num { val, .. })) => {
                self.peekable.next();
                Ok(Expr::Num { val: *val }.to_expr_wrapper())
            }
            // local var
            Some(TokenType::Ident(Ident { name, .. })) => {
                // function call
                self.peekable.next();
                if let Ok(_) = self.expect_next_symbol("(") {
                    let args = self.parse_args()?;
                    let expr = Box::new(Expr::FnCall { fn_name: Rc::clone(&name), args });

                    let ty = match self.find_func(&*name)? {
                        Some(ret_type) => {
                            ret_type
                        },
                        _ => Box::new(Type::Int)
                    };

                    return Ok(ExprWrapper {
                        ty,
                        expr
                    })
                 }
                // variable
                if let Some(VarScope { target, .. }) = self.find_var(&name) {
                    match target {
                        ScopeElement::Var(var) => {
                            Ok(Expr::Var(Rc::clone(var)).to_expr_wrapper())
                        },
                        ScopeElement::Enum(_, val) => {
                            Ok(Expr::Num { val: *val }.to_expr_wrapper())
                        },
                        _ => {
                            let msg = format!("undefined variable: {}", name);
                            Err(msg)
                        }
                    }
                } else {
                    Err(format!("undefined variable: {}", name))
                }
            }
            // str
            Some(TokenType::Str(Str { bytes, .. })) => {
                self.peekable.next();
                let ty = Type::Array {
                    base: Box::new(Type::Char),
                    is_incomplete: false,
                    len: bytes.len()
                };

                let label = self.new_label();
                // bytesはmoveして良さげだが，やり方がわからずcloneしている
                let var = self.new_gvar(&label, Box::new(ty), Some(bytes.clone()), true);

                Ok(Expr::Var(var).to_expr_wrapper())
            }
            // unexpected
            unexpected => {
                let msg = format!("{:?} is unexpected token at primary", unexpected);
                Err(msg)
            }
        }
    }
}
