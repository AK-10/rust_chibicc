use crate::node::{ Stmt, Expr, ExprWrapper };
use crate::token::{ Token, TokenIter, /* TokenIterErr */ };
use crate::program::{ Function, Var, Program };
use crate::_type::Type;
use crate::token::token_type::*;
use crate::scopes::{ TagScope, VarScope, VarOrTypeDef };

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
const TYPE_NAMES: [&str; 5] = ["int", "short", "long", "char", "struct"];

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
        self.program()
    }

    // program := (global-var | function)*
    fn program(&mut self) -> Result<Program, String> {
        let mut nodes: Vec<Function> = Vec::new();

        while let Some(token) = self.peekable.peek() {
            // eofでbreakしないと，以降の処理でpeek()するので全体としてErrになる(Noneでエラーにするような処理がprimaryにある)
            if let Token::Eof = token {
                break
            }
            if self.is_function() {
                nodes.push(self.function()?);
            } else {
                let gvar = self.global_var()?;
                self.globals.push(gvar);
            }
        };

        Ok(Program {
            fns: nodes,
            globals: self.globals.clone()
        })
    }

    // function := basetype declarator "(" params ")" "{" stmt* "}"
    // params := param ("," param)*
    // param := basetype declarator type-suffix
    fn function(&mut self) -> Result<Function, String> {
        self.base_type()?;

        if let Some(Token::Ident(ident)) = self.peekable.next() {
            // scopeを保存するため，コピーを持っておく
            let sc = self.enter_scope();

            // parse params
            let params = self.parse_func_params()?;
            self.locals = params.clone();

            self.expect_next_symbol("{".to_string())?;

            let mut nodes = Vec::new();

            while let Err(_) = self.expect_next_symbol("}".to_string()) {
                nodes.push(self.stmt()?);
            };

            self.leave_scope(sc);

            let locals = self.locals.to_vec();
            self.locals.clear();

            Ok(Function::new(Rc::clone(&ident.name), nodes, locals, params))
        } else {
            Err("expect ident, but different".to_string())
        }
    }

    // stmt := expr ";"
    //       | "return" expr ";"
    //       | "if" "(" expr ")" stmt ("else" stmt)?
    //       | "while" "(" expr ")" stmt
    //       | "for" "(" expr? ";" expr? ";" expr? ")" stmt
    //       | "{" stmt "}"
    //       | "typedef" basetype declarator ("[" num "]")* ";"
    //       | declaration
    fn stmt(&mut self) -> Result<Stmt, String> {
        let tk = self.peekable.peek();
        match tk.map(|t| t.tk_str()) {
            Some(t) if t.as_str() == "return" => {
            // Some(Token::Reserved { op }) if *op == "return" => {
                self.peekable.next();

                let expr = self.expr()?;
                self.expect_next_symbol(";")?;

                Ok(Stmt::Return { val: ExprWrapper::new(expr) })
            }
            Some(t) if t.as_str() == "{" => {
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
            Some(t) if t.as_str() == "if" => {
                self.if_stmt()
            }
            Some(t) if t.as_str() == "while" => {
                self.while_stmt()
            }
            Some(t) if t.as_str() == "for" => {
                self.for_stmt()
            }
            Some(_) if self.is_typename() => {
                self.declaration()
            }
            Some(t) if t.as_str() == "typedef" => {
                self.peekable.next();

                let mut ty = self.base_type()?;
                let name = &mut String::new();
                ty = self.declarator(&mut ty, name)?;
                ty = self.read_type_suffix(ty)?;

                self.expect_next_symbol(";")?;

                self.push_scope_with_typedef(&Rc::new(name.to_string()), &ty);

                Ok(Stmt::ExprStmt {
                    val: ExprWrapper::new(
                        Expr::Null
                    )
                })
            }
            _ => {
                let expr_stmt = self.expr_stmt();
                self.expect_next_symbol(";")?;

                expr_stmt
            }
        }
    }

    // expr := assign
    fn expr(&mut self) -> Result<Expr, String> {
        self.assign()
    }

    // assign := equality ("=" assign)?
    fn assign(&mut self) -> Result<Expr, String> {
        let node = self.equality();
        (&node).as_ref().ok().and_then(|nd| {
            if let Expr::Var(var) = nd {
                return Some(var)
            }

            None
        });

        let is_assign = self.expect_next_reserved("=".to_string());
        if let Ok(_) = is_assign {
            let val = self.expr()?;
            return Ok(Expr::Assign {
                var: ExprWrapper::new(node?),
                val: ExprWrapper::new(val)
            })
        }

        node
    }

    // equality := relational ("==" relational | "!=" relational)*
    fn equality(&mut self) -> Result<Expr, String> {
        let mut node = self.relational()?;

        while let Some(token) = self.peekable.peek() {
            match token {
                Token::Reserved(Reserved { op, .. }) if op.as_str() == "==" => {
                    self.peekable.next();

                    let rhs = self.relational()?;
                    node = Expr::Eq { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                }
                Token::Reserved(Reserved { op, .. }) if op.as_str() == "!=" => {
                    self.peekable.next();

                    let rhs = self.relational()?;
                    node = Expr::Neq { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                }
                _ => { return Ok(node); }
            }
        }

        // while let Some(Token::Reserved(Reserved { op, .. })) = self.peekable.peek() {
        //     self.peekable.next();
        //     let rhs = self.relational()?;

        //     match op.as_str() {
        //         "==" => {
        //             node = Expr::Eq {
        //                 lhs: ExprWrapper::new(node),
        //                 rhs: ExprWrapper::new(rhs)
        //             };
        //         },
        //         "!=" => {
        //             node = Expr::Neq {
        //                 lhs: ExprWrapper::new(node),
        //                 rhs: ExprWrapper::new(rhs)
        //             };
        //         },
        //         _ => break
        //     }
        // }

        Ok(node)
    }

    // relational := add ("<" add | "<=" add | ">" add | ">=" add)*
    fn relational(&mut self) -> Result<Expr, String> {
        let mut node = self.add()?;

        while let Some(Token::Reserved(Reserved { op, .. })) = self.peekable.peek() {
            match op.as_str() {
                "<" => {
                    self.peekable.next();
                    let rhs = self.add()?;

                    node = Expr::Lt {
                        lhs: ExprWrapper::new(node),
                        rhs: ExprWrapper::new(rhs)
                    };
                },
                "<=" => {
                    self.peekable.next();
                    let rhs = self.add()?;

                    node = Expr::Le {
                        lhs: ExprWrapper::new(node),
                        rhs: ExprWrapper::new(rhs)
                    };
                },
                ">" => {
                    self.peekable.next();
                    let rhs = self.add()?;

                    node = Expr::Gt {
                        lhs: ExprWrapper::new(node),
                        rhs: ExprWrapper::new(rhs)
                    };
                },
                ">=" => {
                    self.peekable.next();
                    let rhs = self.add()?;

                    node = Expr::Ge {
                        lhs: ExprWrapper::new(node),
                        rhs: ExprWrapper::new(rhs)
                    };
                },
                _ => break
            }
        }

        Ok(node)
    }

    // add := mul ("+" | "-")*
    fn add(&mut self) -> Result<Expr, String> {
        let mut node = self.mul()?;

        while let Some(Token::Reserved(Reserved { op, .. })) = self.peekable.peek() {
            match op.as_str() {
                "+" => {
                    self.peekable.next();
                    let rhs = self.mul()?;

                    node = Parser::new_add(
                        ExprWrapper::new(node),
                        ExprWrapper::new(rhs)
                    )?;
                },
                "-" => {
                    self.peekable.next();
                    let rhs = self.mul()?;

                    node = Parser::new_sub(
                        ExprWrapper::new(node),
                        ExprWrapper::new(rhs)
                    )?;
                },
                _ => break
            }
        }

        Ok(node)
    }

    fn mul(&mut self) -> Result<Expr, String> {
        let mut node = self.unary()?;

        while let Some(Token::Reserved(Reserved { op, .. })) = self.peekable.peek() {
            match op.as_str() {
                "*" => {
                    self.peekable.next();
                    let rhs = self.unary()?;

                    node = Expr::Mul {
                        lhs: ExprWrapper::new(node),
                        rhs: ExprWrapper::new(rhs)
                    };
                },
                "/" => {
                    self.peekable.next();
                    let rhs = self.unary()?;

                   node = Expr::Div {
                        lhs: ExprWrapper::new(node),
                        rhs: ExprWrapper::new(rhs)
                    };
                },
                _ => break
            }
        }

        Ok(node)
    }

    // unary := ("+" | "-" | "*" | "&")? unary
    //        | postfix
    fn unary(&mut self) -> Result<Expr, String> {
        let tk = self.peekable.peek();

        match tk {
            Some(Token::Reserved(Reserved { op, .. })) => {
                match op.as_str() {
                    "+" => {
                        self.peekable.next();
                        self.primary()
                    },
                    "-" => {
                        self.peekable.next();
                        let rhs = self.unary()?;
                        Ok(Expr::Sub {
                            lhs: ExprWrapper::new(Expr::Num { val: 0 }),
                            rhs: ExprWrapper::new(rhs)
                         })
                    },
                    "*" => {
                        self.peekable.next();
                        let operand = self.unary()?;
                        Ok(Expr::Deref { operand: ExprWrapper::new(operand) })
                    },
                    "&" => {
                        self.peekable.next();
                        let operand = self.unary()?;
                        Ok(Expr::Addr { operand: ExprWrapper::new(operand) })
                    },
                    _ => self.postfix()
                }
            },
            _ => self.postfix()
        }
    }

    // postfix := primary ("[" expr "]" | "." ident | "->" ident)*
    fn postfix(&mut self) -> Result<Expr, String> {
        let mut node = self.primary()?;

        loop {
            if let Ok(_) = self.expect_next_symbol("[") {
                // x[y] is short for *(x + y)
                let expr = self.expr()?;
                let exp = Parser::new_add(
                    node.to_expr_wrapper(),
                    expr.to_expr_wrapper()
                )?;

                match self.expect_next_symbol("]".to_string()) {
                    Ok(_) => {
                        node = Expr::Deref { operand: exp.to_expr_wrapper() };
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
                node = Expr::Deref { operand: node.to_expr_wrapper() };
                node = self.struct_ref(node)?;
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
    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.peekable.peek();

        match token {
            Some(Token::Symbol(symbol)) if symbol.sym.as_str() == "(" => {
                self.peekable.next();

                if self.expect_next_symbol("{".to_string()).is_ok() {
                    return self.stmt_expr()
                }

                let expr = self.expr();
                self.expect_next_symbol(")".to_string())?;

                expr
            }
            // num
            Some(Token::Num(Num { val, .. })) => {
                self.peekable.next();
                Ok(Expr::Num { val: *val })
            }
            // local var
            Some(Token::Ident(Ident { name, .. })) => {
                // function call
                self.peekable.next();
                if let Ok(_) = self.expect_next_symbol("(") {
                    let args = self.parse_args()?;

                    return Ok(Expr::FnCall { fn_name: Rc::clone(name), args })
                }
                // variable
                if let Some(VarScope { target, .. }) = self.find_var(&name) {
                    if let VarOrTypeDef::Var(var) = target {
                        Ok(Expr::Var(Rc::clone(var)))
                    } else {
                        let msg = format!("undefined variable: {}", name);
                        Err(msg)
                    }
                } else {
                    Err(format!("undefined variable: {}", name))
                }
            }
            Some(Token::Str(Str { bytes, .. })) => {
                self.peekable.next();
                let ty = Type::Array {
                    base: Box::new(Type::Char),
                    len: bytes.len()
                };

                let label = self.new_label();
                let var = self.new_gvar_with_contents(&label, Box::new(ty), &bytes);
                self.globals.push(Rc::clone(&var));

                Ok(Expr::Var(var))
            }
            Some(Token::Reserved(Reserved { op, .. })) if op.as_str() == "sizeof" => {
                self.peekable.next();
                let node = self.unary()?;
                let size = node.detect_type().size();

                Ok(Expr::Num { val: size as isize })
            }
            // unexpected
            unexpected => {
                let msg = format!("{:?} is unexpected token at primary", unexpected);
                Err(msg)
            }
        }
    }
}
