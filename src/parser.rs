use crate::node::{ Stmt, Expr, ExprWrapper };
use crate::token::{ Token, TokenIter, /* TokenIterErr */ };
use crate::program::{ Function, Var, Program };
use crate::_type::Type;

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
const TYPE_NAMES: [&str; 3] = ["int", "char", "struct"];

pub struct Parser<'a> {
    pub input: &'a Vec<Token>,
    peekable: TokenIter<'a>,
    // 関数の引数，関数内で宣言された変数を保持する, 関数のスコープから外れたらリセットする
    pub locals: Vec<Rc<RefCell<Var>>>,
    pub globals: Vec<Rc<RefCell<Var>>>,
    pub scope: Vec<Rc<RefCell<Var>>>,
    pub label_cnt: usize
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a Vec<Token>) -> Self {
        Self {
            input,
            peekable: TokenIter::new(input),
            locals: Vec::new(),
            globals: Vec::new(),
            scope: Vec::new(),
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

    // function := basetype ident "(" params ")" "{" stmt* "}"
    // params := param ("," param)*
    // param := basetype ident
    fn function(&mut self) -> Result<Function, String> {
        self.base_type()?;

        if let Some(Token::Ident{ name }) = self.peekable.next() {
            // scopeを保存するため，コピーを持っておく
            let sc = self.scope.clone();

            // parse params
            let params = self.parse_func_params()?;
            self.locals = params.clone();

            self.expect_next_symbol("{".to_string())?;

            let mut nodes = Vec::new();

            while let Err(_) = self.expect_next_symbol("}".to_string()) {
                nodes.push(self.stmt()?);
            };

            self.scope = sc;

            let locals = self.locals.to_vec();
            self.locals.clear();

            Ok(Function::new(name.to_string(), nodes, locals, params))
        } else {
            Err("expect ident, but different".to_string())
        }
    }

    // stmt := expr ";"
    //       | "return" expr ";"
    //       | "if" "(" expr ")" stmt ("else" stmt)?
    //       | "while" "(" expr ")" stmt
    //       | "for" "(" expr? ";" expr? ";" expr? ")" stmt
    //       | declaration
    fn stmt(&mut self) -> Result<Stmt, String> {
        let tk = self.peekable.peek();
        match tk {
            Some(Token::Reserved { op }) if *op == "return" => {
                self.peekable.next();

                let expr = self.expr()?;
                self.expect_next_symbol(";".to_string())?;

                Ok(Stmt::Return { val: ExprWrapper::new(expr) })
            }
            Some(Token::Symbol(op)) if *op == "{" => {
                self.peekable.next();
                let mut stmts: Vec<Stmt> = Vec::new();

                let sc = self.scope.clone();
                while let Err(_) = self.expect_next_symbol("}".to_string()) {
                    let stmt = self.stmt()?;
                    stmts.push(stmt);
                }
                self.scope = sc;

                Ok(Stmt::Block { stmts })
            }
            Some(Token::Reserved { op }) if *op == "if" => {
                self.if_stmt()
            }
            Some(Token::Reserved { op }) if *op == "while" => {
                self.while_stmt()
            }
            Some(Token::Reserved { op }) if *op == "for" => {
                self.for_stmt()
            }
            Some(Token::Reserved { op }) if TYPE_NAMES.contains(&op.as_str()) => {
                self.declaration()
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
                Token::Reserved { op } if *op == "==" => {
                    self.peekable.next();

                    let rhs = self.relational()?;
                    node = Expr::Eq { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                }
                Token::Reserved { op } if *op == "!=" => {
                    self.peekable.next();

                    let rhs = self.relational()?;
                    node = Expr::Neq { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                }
                _ => { return Ok(node); }
            }
        }

        Ok(node)
    }

    // relational := add ("<" add | "<=" add | ">" add | ">=" add)*
    fn relational(&mut self) -> Result<Expr, String> {
        let mut node = self.add()?;

        while let Some(token) = self.peekable.peek() {
            match token {
                Token::Reserved { op } if *op == "<" => {
                    self.peekable.next();

                    let rhs = self.add()?;
                    node = Expr::Lt { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                }
                Token::Reserved { op } if *op == "<=" => {
                    self.peekable.next();

                    let rhs = self.add()?;
                    node = Expr::Le { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                }
                Token::Reserved { op } if *op == ">" => {
                    self.peekable.next();

                    let rhs = self.add()?;
                    node = Expr::Gt { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                }
                Token::Reserved { op } if *op == ">=" => {
                    self.peekable.next();

                    let rhs = self.add()?;
                    node = Expr::Ge { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                }
                _ => { return Ok(node); }
            }
        }

        Ok(node)
    }

    // add := mul ("+" | "-")*
    fn add(&mut self) -> Result<Expr, String> {
        let mut node = self.mul()?;

        while let Some(token) = self.peekable.peek() {
            match token {
                // "+" mul
                Token::Reserved { op } if *op == "+" => {
                    self.peekable.next();

                    let lhs = ExprWrapper::new(node);
                    let rhs = ExprWrapper::new(self.mul()?);

                    node = Parser::new_add(lhs, rhs)?;
                },
                // "-" mul
                Token::Reserved { op } if *op == "-" => {
                    self.peekable.next();
                    let lhs = ExprWrapper::new(node);
                    let rhs = ExprWrapper::new(self.mul()?);

                    node = Parser::new_sub(lhs, rhs)?;
                },
                // mul
                _ => { return Ok(node); }
            };
        }

        Ok(node)
    }

    fn mul(&mut self) -> Result<Expr, String> {
        let mut node = self.unary()?;

        while let Some(token) = self.peekable.peek() {
            match token {
                // "*" primary
                Token::Reserved { op } if *op == "*" => {
                    self.peekable.next();

                    let rhs = self.unary()?;
                    node = Expr::Mul { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                },

                // "/" primary
                Token::Reserved { op } if *op == "/" => {
                    self.peekable.next();

                    let rhs = self.unary()?;
                    node = Expr::Div { lhs: ExprWrapper::new(node), rhs: ExprWrapper::new(rhs) };
                },
                _ => {
                    return Ok(node);
                }
            }
        }


        Ok(node)
    }

    // unary := ("+" | "-" | "*" | "&")? unary
    //        | postfix
    fn unary(&mut self) -> Result<Expr, String> {
        let tk = self.peekable.peek();
        match tk {
            Some(Token::Reserved { op }) if *op == "+" => {
                self.peekable.next();

                self.primary()
            },
            Some(Token::Reserved { op }) if *op == "-" => {
                self.peekable.next();

                let rhs = self.unary()?;
                Ok(Expr::Sub {
                    lhs: ExprWrapper::new(Expr::Num { val: 0 }),
                    rhs: ExprWrapper::new(rhs)
                })
            },
            Some(Token::Reserved { op }) if *op == "*" => {
                self.peekable.next();
                let operand = self.unary()?;

                Ok(Expr::Deref { operand: ExprWrapper::new(operand) })
            },
            Some(Token::Reserved { op }) if *op == "&" => {
                self.peekable.next();
                let operand = self.unary()?;

                Ok(Expr::Addr { operand: ExprWrapper::new(operand) })
            }
            _ => {
                self.postfix()
            }
        }
    }

    // postfix := primary ("[" expr "]" | "." ident)*
    fn postfix(&mut self) -> Result<Expr, String> {
        let mut node = self.primary()?;

        loop {
            if let Ok(_) = self.expect_next_symbol("[") {
                // x[y] is short for *(x + y)
                let expr = self.expr()?;
                let exp = Parser::new_add(node.to_expr_wrapper(), expr.to_expr_wrapper())?;

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
            // ERR: compile error
            // expected tuple struct or tuple variant, found associated function `String::from`
            // Some(Token::Reserved { op: String::from("(") }) => {}
            Some(Token::Symbol(op)) if op == "(" => {
                self.peekable.next();

                if self.expect_next_symbol("{".to_string()).is_ok() {
                    return self.stmt_expr()
                }

                let expr = self.expr();
                self.expect_next_symbol(")".to_string())?;

                expr
            }
            // num
            Some(Token::Num { val, .. }) => {
                self.peekable.next();
                Ok(Expr::Num { val: *val })
            }
            // local var
            Some(Token::Ident { name }) => {
                // function call
                self.peekable.next();
                if let Ok(_) = self.expect_next_symbol("(".to_string()) {
                    // 引数なし
                    if let Ok(_) = self.expect_next_symbol(")".to_string()) {
                        return Ok(Expr::FnCall { fn_name: name.clone(), args: Vec::new() })
                    }
                    let args = self.parse_args()?;
                    self.expect_next_symbol(")".to_string())?;

                    return Ok(Expr::FnCall { fn_name: name.clone(), args })
                }
                // variable
                if let Some(ref var) = self.find_var(&name) {
                    Ok(Expr::Var(Rc::clone(var)))
                } else {
                    Err(format!("undefined variable: {:?}", name).to_string())
                }
            }
            Some(Token::Str(contents)) => {
                self.peekable.next();
                let ty = Type::Array {
                    base: Rc::new(Type::Char),
                    len: contents.len()
                };

                let label = self.new_label();
                let var = self.new_gvar_with_contents(&label, Rc::new(ty), &contents);
                self.globals.push(Rc::clone(&var));

                Ok(Expr::Var(var))
            }
            Some(Token::Reserved { op }) if op == "sizeof" => {
                self.peekable.next();
                let node = self.unary()?;
                let size = node.detect_type().size();

                Ok(Expr::Num { val: size as isize })
            }
            // unexpected
            _ => {
                Err("unexpected token at primary".to_string())
            }
        }
    }
}
