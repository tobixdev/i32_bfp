use std::mem;

use dynasmrt::{Assembler, Register, x64::{Rq, X64Relocation}};
use dynasmrt::{dynasm, DynasmApi};
use crate::ast::Expr;

pub struct CompilationContext {
    ops: Assembler<X64Relocation>,
    available_registers: Vec<Rq>
}

impl CompilationContext {
    pub fn new() -> CompilationContext {
        CompilationContext {
            ops: dynasmrt::x64::Assembler::new().unwrap(),
            available_registers: vec![Rq::R8, Rq::R9, Rq::R10, Rq::R11, Rq::R12, Rq::R13, Rq::R14, Rq::R15]
        }
    }

    fn next_register(&mut self) -> Rq {
        self.available_registers.pop().expect("All registers used!")
    }

    pub fn compile(mut self, expr: &Expr) -> extern "win64" fn(i32) -> i32 {
        let offset = self.ops.offset();
        dynasm!(self.ops
            ; .arch x64
        );
        let result_register = expr.compile(&mut self);
        dynasm!(self.ops
            ; mov rax, Rq(result_register.code())
            ; ret
        );

        let buf = self.ops.finalize().unwrap();

        let expr_fn: extern "win64" fn(i32) -> i32 = unsafe { mem::transmute(buf.ptr(offset)) };
        expr_fn
    }
}

pub trait Compilable {
    fn compile(&self, ctx: &mut CompilationContext) -> Rq;
}

impl Compilable for Expr {
    fn compile(&self, mut ctx: &mut CompilationContext) -> Rq {
        match self {
            Expr::Number(number) => compile_number(*number, &mut ctx),
            _ => panic!("Unknown expression node")
        }
    }
}

fn compile_number(number: i32, ctx: &mut CompilationContext) -> Rq {
    let register = ctx.next_register();
    dynasm!(ctx.ops
        ; mov Rq(register.code()), QWORD number as _
    );
    register
}