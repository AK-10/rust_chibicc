use crate::node::{ Stmt, Expr, ExprWrapper };
use crate::program::{ Program, Var };
use crate::_type::Type;

use std::cell::{ Cell, RefCell };
use std::convert::TryFrom;

const ARG_REG1: [&str; 6] = ["dil", "sil", "dl", "cl", "r8b", "r9b"];
const ARG_REG2: [&str; 6] = ["di", "si", "dx", "cx", "r8w", "r9w"];
const ARG_REG4: [&str; 6] = ["edi", "esi", "edx", "ecx", "r8d", "r9d"];
const ARG_REG8: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

pub struct CodeGenerator<'a> {
    prog: &'a Program,
    funcname: RefCell<String>,
    labelseq: Cell<usize>,
}

impl<'a> CodeGenerator<'a> {
    pub fn new(prog: &'a Program) -> Self {
        //println!("{:#?}", prog);
        Self {
            prog,
            funcname: RefCell::new(String::new()),
            labelseq: Cell::new(0)
        }
    }

    pub fn codegen(&self) {
        println!(".intel_syntax noprefix");
        self.emit_data();
        self.emit_text();
    }

    fn gen_expr(&self, expr_wrapper: &ExprWrapper) {
        match expr_wrapper.expr.as_ref() {
            Expr::AddEq { var, val }
            | Expr::PtrAddEq { var, val }
            | Expr::SubEq { var, val }
            | Expr::PtrSubEq { var, val }
            | Expr::MulEq { var, val }
            | Expr::DivEq { var, val } => {
                self.gen_lval(var);
                println!("  push [rsp]");
                load(var.ty.as_ref());
                self.gen_expr(val);
                self.gen_binary(expr_wrapper);
                store(expr_wrapper.ty.as_ref());
            }
            Expr::Add { lhs, rhs }
            | Expr::PtrAdd { lhs, rhs }
            | Expr::Sub { lhs, rhs }
            | Expr::PtrSub { lhs, rhs }
            | Expr::PtrDiff { lhs, rhs }
            | Expr::Mul { lhs, rhs }
            | Expr::Div { lhs, rhs }
            | Expr::Eq { lhs, rhs }
            | Expr::Neq { lhs, rhs }
            | Expr::Lt { lhs, rhs }
            | Expr::Le { lhs, rhs } => {
                self.gen_expr(lhs);
                self.gen_expr(rhs);

                self.gen_binary(expr_wrapper)
            }
            | Expr::Gt { lhs, rhs }
            | Expr::Ge { lhs, rhs } => {
                self.gen_expr(rhs);
                self.gen_expr(lhs);

                self.gen_binary(expr_wrapper)
            }
            Expr::Num { val } => {
                // `push` instraction cannot push a 64-bit integer. In order to push it,
                // we have to first load a large integer to aregister using movabs and then
                // push it to the stack
                //
                // if 64bit int is able to convert 32bit int, can be treated as 32bit
                if i32::try_from(*val).is_ok() {
                    println!("  push {}", val);
                } else {
                    println!("  movabs rax, {}", val);
                    println!("  push rax");
                }
            },
            Expr::Cast(ty, expr_wrapper) => {
                self.gen_expr(expr_wrapper);
                trancate(ty);
            }
            Expr::Var(_) => {
                self.gen_addr(expr_wrapper);
                match *expr_wrapper.ty {
                    Type::Array { .. } => {},
                    _ => { load(&expr_wrapper.ty); }
                }
            }
            Expr::Assign { var, val, .. } => {
                match *expr_wrapper.ty {
                    Type::Array { .. } => { panic!("not an lvalue") },
                    _ => { self.gen_addr(var); }
                }

                self.gen_expr(val);
                store(&expr_wrapper.ty);
            }
            Expr::PreInc(ew) => {
                self.gen_lval(ew);
                println!("  push [rsp]");
                load(ew.ty.as_ref());
                inc(ew.ty.as_ref());
                store(ew.ty.as_ref());
            }
            Expr::PreDec(ew) => {
                self.gen_lval(ew);
                println!("  push [rsp]");
                load(ew.ty.as_ref());
                dec(ew.ty.as_ref());
                store(ew.ty.as_ref());
            }
            Expr::PostInc(ew) => {
                self.gen_lval(ew);
                println!("  push [rsp]");
                load(ew.ty.as_ref());
                inc(ew.ty.as_ref());
                store(ew.ty.as_ref());
                dec(ew.ty.as_ref());
            }
            Expr::PostDec(ew) => {
                self.gen_lval(ew);
                println!("  push [rsp]");
                load(ew.ty.as_ref());
                dec(ew.ty.as_ref());
                store(ew.ty.as_ref());
                inc(ew.ty.as_ref());
            }
            Expr::Comma { lhs, rhs } => {
                //println!("{:#?}", expr_wrapper);
                self.gen_stmt(lhs);
                self.gen_expr(rhs);
            }
            Expr::FnCall { fn_name, args, .. } => {
                let arg_size = args.len();
                args.iter().for_each( |arg| {
                    self.gen_expr(arg);
                });

                // 引数をスタックにpushして各レジスタにpopすることで引数を渡す
                // sub(2, 4)のとき
                // push 2
                // push 4
                // ここで関数定義がsub(a, b)ならば，a -> rdi, b -> rsiになるようにすれば良い
                // このように引数の順序を保持するには,使うレジスタの逆からpopする必要がある
                // pop rsi -> 4
                // pop rdi -> 2
                // clang, gccでassemblyダンプしてみると何かわかるかもしれない
                // cc -S -mllvm --x86-asm-syntax=intel assign.c -O0
                for idx in (0..arg_size).rev() {
                    println!("  pop {}", ARG_REG8[idx])
                }

                // ABIの仕様で関数呼び出しの前にRSPを(16の倍数)にする必要がある
                // push, popは8バイト単位で変更を行うのでcall命令を行うときにスタックが(16の倍数)byteになっているとは限らない
                // やりたいことは, RSP(スタックの先頭のポインタ)が16の倍数でなければ8を追加する(スタックの方向的にsub rsp, 8をする)
                // RAX は variadic function のために0にセットする
                // 「x86 関数呼び出し アライメント」でぐぐるといろいろ出てくる
                let cur_labelseq = self.labelseq.get();
                self.labelseq.set(cur_labelseq + 1);

                // and rax, 15
                // 15 -> 00001111
                // andの結果が5ビット目より下位ビットが立っている場合16で割り切れない事になる
                // 5ビット目以上はandでは常に0になる（15のビットより）
                // つまり16で割り切れる場合，andによってZF = 1になる(アライメントを調整しなくて良い)
                // この場合, raxを0にセットしてcall命令を呼ぶだけ
                // 16で割り切れない場合，ZF = 0より
                // sub rsp, 8 mov rax, 0 をして関数を呼ぶ
                println!("  mov rax, rsp");
                println!("  and rax, 15"); // and: オペランドの論理積を計算し，第一引数に格納

                // jnz: フラグレジスタのZFが0の時(比較の結果，等しくない)，adr[,x]のアドレスへ分岐(実行が移動)する
                println!("  jnz .L.call.{}", self.labelseq.get());
                println!("  mov rax, 0");
                println!("  call {}", fn_name);
                println!("  jmp .L.end.{}", self.labelseq.get());
                println!(".L.call.{}:", self.labelseq.get());
                println!("  sub rsp, 8");
                println!("  mov rax, 0");

                println!("  call {}", fn_name);

                println!("  add rsp, 8");
                println!(".L.end.{}:", self.labelseq.get());
                println!("  push rax");
            }
            Expr::Addr { operand } => {
                self.gen_addr(operand);
            }
            Expr::Deref { operand } => {
                self.gen_expr(operand);
                match *expr_wrapper.ty {
                    Type::Array { .. } => {},
                    _ => { load(&expr_wrapper.ty); }
                }
            }
            Expr::Null => return,
            Expr::StmtExpr(stmts) => {
                stmts.iter().for_each(|stmt| {
                    self.gen_stmt(stmt);
                });
            },
            Expr::Member(_, __) => {
                self.gen_addr(expr_wrapper);
                match *expr_wrapper.ty {
                    Type::Array { .. } => {},
                    _ => { load(&expr_wrapper.ty); }
                }
            }
        }
    }

    fn gen_stmt(&self, stmt: &Stmt) {
        match stmt {
            Stmt::Return { val } => {
                self.gen_expr(val);

                println!("  pop rax");
                println!("  jmp .L.return.{}", self.funcname.borrow().as_str());
            }
            Stmt::ExprStmt { val } => {
                self.gen_expr(val);
                if let Expr::Null = *val.expr {
                } else {
                    println!("  add rsp, 8");
                }
            }
            Stmt::If { cond, then, els } => {
                // if (A) B else Cのアセンブリ疑似コード
                //   Aをコンパイルしたコード(この式の結果はstackにpushされているはず)
                //   pop rax
                //   cmp rax, 0
                //   je .L.else.XXX (rax == 0 出なければjumpしない(Bが実行される))
                //   Bをコンパイルしたコード
                //   jmp .L.end.XXX (elseブロックに行かないようにjumpする)
                // .L.else.XXX
                //   Cをコンパイルしたコード
                // .L.end.XXX

                self.labelseq.set(self.labelseq.get() + 1);
                let seq = self.labelseq.get();

                self.gen_expr(cond);
                // stackのトップにcondの結果が格納されているはず
                println!("  pop rax");
                // conditionの結果を0と比較する. (条件式が偽であることかをみている)
                // 等しい場合, je .L.else.XXX でelse blockの処理に飛ぶ(jump equal)
                println!("  cmp rax, 0");
                // else block exist
                if let Some(els_block) = els {
                    println!("  je .L.else.{}", seq);
                    self.gen_stmt(then);
                    println!("  jmp .L.end.{}", seq);
                    println!(".L.else.{}:", seq);
                    self.gen_stmt(els_block);
                    println!(".L.end.{}:", seq);
                // not exist
                } else {
                    println!("  je .L.end.{}", seq);
                    self.gen_stmt(then);
                    println!(".L.end.{}:", seq);
                }
            }
            Stmt::While { cond, then } => {
                self.labelseq.set(self.labelseq.get() + 1);
                let seq = self.labelseq.get();

                println!(".L.begin.{}:", seq);
                self.gen_expr(cond);
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  je .L.end.{}", seq);

                self.gen_stmt(then);
                println!("  jmp .L.begin.{}", seq);
                println!(".L.end.{}:", seq);
            }
            Stmt::For { init, cond, inc, then } => {
                self.labelseq.set(self.labelseq.get() + 1);
                let seq = self.labelseq.get();

                init.as_ref()
                    .as_ref()
                    .map(|stmt| self.gen_stmt(stmt));
                println!(".L.begin.{}:", seq);

                cond.as_ref().as_ref().map(|x| {
                    self.gen_expr(x);
                    println!("  pop rax");
                    println!("  cmp rax, 0");
                    println!("  je .L.end.{}", seq);
                });

                self.gen_stmt(then);

                inc.as_ref()
                    .as_ref()
                    .map(|x| self.gen_stmt(x));
                println!("  jmp .L.begin.{}", seq);
                println!(".L.end.{}:", seq);
            }
            Stmt::Block { stmts } => {
                stmts.iter().for_each(|stmt| self.gen_stmt(stmt));
            },
            Stmt::PureExpr(expr_wrapper) => self.gen_expr(expr_wrapper)
        };
    }

    fn gen_binary(&self, ew: &ExprWrapper) {
        println!("  pop rdi");
        println!("  pop rax");

        match ew.expr.as_ref() {
            Expr::Add { .. } | Expr::AddEq { .. } => {
                println!("  add rax, rdi");
            }
            Expr::PtrAdd { .. } | Expr::PtrAddEq { .. } => {
                println!("  imul rdi, {}", ew.ty.base_size());
                println!("  add rax, rdi");
            }
            Expr::Sub { .. } | Expr::SubEq { .. } => {
                println!(" sub rax, rdi");
            }
            Expr::PtrSub { .. } | Expr::PtrSubEq { .. } => {
                println!("  imul rdi, {}", ew.ty.base_size());
                println!("  sub rax, rdi");
            }
            Expr::PtrDiff { lhs, .. } => {
                println!("  sub rax, rdi");
                println!("  cqo");
                println!("  mov rdi, {}", lhs.ty.base_size());
                println!("  idiv rdi");
            }
            Expr::Mul { .. } | Expr::MulEq { .. } => {
                println!("  imul rax, rdi");
            }
            Expr::Div { .. } | Expr::DivEq { .. } => {
                // idiv命令は符号あり除算を行う命令
                // rdxとraxをとってそれを合わせたものを128bit整数とみなす
                // それを引数のレジスタの64bit整数で割り，商をrax, 余をrdxにセットする
                // cqo命令を使うと、RAXに入っている64ビットの値を128ビットに伸ばして
                // rdxとraxにセットすることができる
                println!("  cqo");
                println!("  idiv rdi");
            }
            Expr::Eq { .. } => {
                // cmp命令: 二つの引数のレジスタを比較して, フラグレジスタに結果を格納
                // sete命令: 指定のレジスタにフラグレジスタの値を格納. seteであれば==の時1になる
                //           8bitしか書き込めないのでalを指定している
                //           setneは!=のとき1になる
                // movzb命令: movzb dist, srcでsrcをdistに書き込む．またsrcで指定されたbitより上の桁は0埋めする
                // al: raxの下位8bitのエイリアス. alを変更するとraxも変更される
                println!("  cmp rax, rdi");
                println!("  sete al");
                println!("  movzb rax, al");
            }
            Expr::Neq { .. } => {
                println!("  cmp rax, rdi");
                println!("  setne al");
                println!("  movzb rax, al");
            }
            Expr::Gt { .. } => {
                println!("  cmp rax, rdi");
                println!("  setl al");
                println!("  movzb rax, al");
            }
            Expr::Ge { .. } => {
                println!("  cmp rax, rdi");
                println!("  setle al");
                println!("  movzb rax, al");
            }
            Expr::Lt { .. } => {
                println!("  cmp rax, rdi");
                println!("  setl al");
                println!("  movzb rax, al");
            }
            Expr::Le { .. } => {
                println!("  cmp rax, rdi");
                println!("  setle al");
                println!("  movzb rax, al");
            }
            _ => unreachable!()
        }

        println!("  push rax");
    }

    fn gen_lval(&self, ew: &ExprWrapper) {
        if let Type::Array { .. } = *ew.ty {
            panic!("not an lvalue");
        }
        self.gen_addr(ew)
    }

    // pushes the given node's address to the stack
    fn gen_addr(&self, expr_wrapper: &ExprWrapper) {
        match expr_wrapper.expr.as_ref() {
            Expr::Deref { operand } => {
                self.gen_expr(operand);
            }
            Expr::Var(var) => {
                if var.borrow().is_local {
                    // lea: アドレスのロード
                    println!("  lea rax, [rbp-{}]", var.borrow().offset.value());
                    println!("  push rax");
                } else {
                    println!("  push offset {}", var.borrow().name);
                }
            },
            Expr::Member(ew, member) => {
                self.gen_addr(ew);
                println!("  pop rax");
                println!("  add rax, {}", member.offset.value());
                println!("  push rax");
            },
            _ => {
                panic!("unexpected operand {:?}", expr_wrapper);
            }
        }
    }

    fn emit_data(&self) {
        println!(".data");
        self.prog.globals.iter().for_each(|v| {
            let var = v.borrow();
            println!("{}:", var.name);
            if let Some(contents) = &var.contents {
                contents.iter().for_each(|ch| {
                    println!("  .byte {}", ch);
                });
            } else {
                println!("  .zero {}", var.ty.size());
            }
        });
    }

    fn emit_text(&self) {
        println!(".text");
        self.prog.fns.iter().for_each(|func| {
            let mut node_iter = func.nodes.iter();
            *self.funcname.borrow_mut() = func.name.to_string();
            let funcname = self.funcname.borrow().to_string();
            if !func.is_static {
                println!(".global {}", funcname);
            }
            println!("{}:", funcname);

            // Prologue
            println!("  push rbp");
            println!("  mov rbp, rsp");
            println!("  sub rsp, {}", func.stack_size);

            func.params.iter().enumerate().for_each(|(i, var)| {
                load_arg(&*var.borrow(), i);
            });

            while let Some(node) = node_iter.next() {
                self.gen_stmt(node);
            };

            // Epilogue
            // 最後の式の結果がRAXに残っているのでそれが返り値になる
            println!(".L.return.{}:", funcname);
            println!("  mov rsp, rbp");
            println!("  pop rbp");
            println!("  ret");
        });
    }
}

fn load_arg(var: &Var, idx: usize) {
    match var.ty.size() {
        1 => println!("  mov [rbp-{}], {}", var.offset.value(), ARG_REG1[idx]),
        2 => println!("  mov [rbp-{}], {}", var.offset.value(), ARG_REG2[idx]),
        4 => println!("  mov [rbp-{}], {}", var.offset.value(), ARG_REG4[idx]),
        8 => println!("  mov [rbp-{}], {}", var.offset.value(), ARG_REG8[idx]),
        x => unreachable!(format!("{} is unexpected byte size of type", x))
    }
}

fn load(ty: &Type) {
    println!("  pop rax");
    match ty.size() {
        1 => println!("  movsx rax, byte ptr [rax]"),
        2 => println!("  movsx rax, word ptr [rax]"),
        4 => println!("  movsxd rax, dword ptr [rax]"),
        8 => println!("  mov rax, [rax]"),
        x => unreachable!(format!("{} is unexpected byte size of type", x))
    };

    println!("  push rax");
}

fn store(ty: &Type) {
    println!("  pop rdi");
    println!("  pop rax");

    if let Type::Bool = ty {
        // bool
        // => 0         = 0
        //    otherwise = 1
        // 0と比較して一致しないときゼロフラグがセットされるのでこれをdilに格納
        // またゼロ拡張したいのでmovzb rdi, dilする
        println!("  cmp rdi, 0");
        println!("  setne dil"); // dilはrdiの下位8bit, raxのalみたいなもん
        println!("  movzb rdi, dil");
    }

    match ty.size() {
        1 => println!("  mov [rax], dil"),
        2 => println!("  mov [rax], di"),
        4 => println!("  mov [rax], edi"),
        8 => println!("  mov [rax], rdi"),
        x => unreachable!(format!("{} is unexpected byte size of type", x))
    };

    println!("  push rdi");
}

fn trancate(ty: &Type) {
    println!("  pop rax");

    if let Type::Bool = ty {
        println!("  cmp rax, 0");
        println!("  setne al");
    }

    match ty.size() {
        1 => println!("  movsx rax, al"),
        2 => println!("  movsx rax, ax"),
        4 => println!("  movsxd rax, eax"),
        _ => {}
    }

    println!("  push rax");
}

fn inc(ty: &Type) {
    println!("  pop rax");
    let sz = if ty.has_base() {
        ty.base_size()
    } else {
        1
    };
    println!("  add rax, {}", sz);
    println!("  push rax");
}

fn dec(ty: &Type) {
    println!("  pop rax");
    let sz = if ty.has_base() {
        ty.base_size()
    } else {
        1
    };
    println!("  sub rax, {}", sz);
    println!("  push rax");
}
