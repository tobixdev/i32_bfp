use std::slice;
use std::{collections::HashMap, mem, ops::Deref};

use dynasmrt::x64::Rq;
use dynasmrt::{Assembler, AssemblyOffset, ExecutableBuffer, Register, x64::{X64Relocation}};
use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};
use crate::ast::Expr;
use crate::code_repository::CodeRepository;

pub struct CompilationContext<'a> {
    ops: Assembler<X64Relocation>,
    available_registers: Vec<Rq>,
    available_parameter_registers: Vec<Rq>,
    var: HashMap<String, Rq>,
    code_repository: &'a CodeRepository
}

impl CompilationContext<'_> {
    pub fn new<'a>(code_repository: &'a CodeRepository) -> CompilationContext<'a> {
        CompilationContext {
            ops: dynasmrt::x64::Assembler::new().unwrap(),
            available_registers: vec![Rq::RBX, Rq::R8, Rq::R9, Rq::R10, Rq::R11, Rq::R12, Rq::R13, Rq::R14, Rq::R15],
            available_parameter_registers: vec![Rq::RCX],
            var: HashMap::new(),
            code_repository: code_repository
        }
    }

    fn next_register(&mut self) -> Result<Rq, String> {
        self.available_registers.pop().ok_or("No more registers available!".to_string())
    }

    pub fn assign_register_to_variable(&mut self, var: String) -> Result<Rq, String> {
        let result= self.available_parameter_registers.pop()
            .ok_or("No more parameter registers available!".to_string());
        if let Ok(reg) = result {
            self.var.insert(var, reg);
        }
        result
    }

    fn free_if_possible(&mut self, reg: Rq) {
        if ![Rq::RAX, Rq::RCX].contains(&reg) {
            self.available_parameter_registers.push(reg);
        }
    }

    pub fn compile(mut self, expr: &Expr) -> Result<Runable, String> {
        println!("JIT> Compiler called. Starting assembly ...");
        let offset = self.ops.offset();
        dynasm!(self.ops
            ; .arch x64
            ; push rbx
            ; push r12
            ; push r13
            ; push r14
            ; push r15
        );
        let result_register = expr.compile(&mut self)?;
        dynasm!(self.ops
            ; mov rax, Rq(result_register.code())
            ; pop r15
            ; pop r14
            ; pop r13
            ; pop r12
            ; pop rbx
            ; ret
        );
        let buf = self.ops.finalize().unwrap();
        
        println!("JIT> Compilation finished. Code has size {} @{:p}.", buf.len(), buf.ptr(offset));

        Ok(Runable::new(buf, offset))
    }
}

#[derive(Debug)]
pub struct Runable {
    buf: ExecutableBuffer,
    offset: AssemblyOffset
}

impl Runable {
    pub fn new(buf: ExecutableBuffer, offset: AssemblyOffset) -> Runable {
        Runable {buf, offset}
    }

    pub fn call(&self, arg1: i32) -> i32 {
        let expr_fn: extern "win64" fn(i32) -> i32 = unsafe { mem::transmute(self.buf.ptr(self.offset)) };
        expr_fn(arg1)
    }

    pub fn print(&self) {
        println!("Code (size: {}):", self.buf.len());
        
        for byte in self.buf.deref() {
            print!("{:02x}", byte);
        }
        println!()
    }
}


pub trait Compilable {
    fn compile(&self, ctx: &mut CompilationContext) -> Result<Rq, String>;
}

impl Compilable for Expr {
    fn compile(&self, mut ctx: &mut CompilationContext) -> Result<Rq, String> {
        match self {
            Expr::Number(number) => compile_number(*number, &mut ctx),
            Expr::Var(var) => compile_var(&var, &mut ctx),
            Expr::Add(lhs, rhs) => compile_add(&lhs, &rhs, &mut ctx),
            Expr::Sub(lhs, rhs) => compile_sub(&lhs, &rhs, &mut ctx),
            Expr::Mul(lhs, rhs) => compile_mul(&lhs, &rhs, &mut ctx),
            Expr::Div(lhs, rhs) => compile_div(&lhs, &rhs, &mut ctx),
            Expr::Eq(lhs, rhs) => compile_eq(&lhs, &rhs, &mut ctx),
            Expr::Neq(lhs, rhs) => compile_neq(&lhs, &rhs, &mut ctx),
            Expr::FunctionCall(name, param) => compile_function_call(&name, &param, &mut ctx)
        }
    }
}

fn compile_number(number: i32, ctx: &mut CompilationContext) -> Result<Rq, String> {
    let register = ctx.next_register()?;
    dynasm!(ctx.ops
        ; mov Rq(register.code()), QWORD number as _
    );
    Ok(register)
}

fn compile_var(name: &String, ctx: &mut CompilationContext) -> Result<Rq, String> {
    let register = ctx.var.get(name).ok_or("Variable was not defined".to_string()).map(|rq| *rq)?;
    Ok(register)
}

fn compile_add(lhs: &Expr, rhs: &Expr, mut ctx: &mut CompilationContext) -> Result<Rq, String> {
    let lhs_reg = lhs.compile(&mut ctx)?;
    let rhs_reg = rhs.compile(&mut ctx)?;
    let new_reg = ctx.next_register()?;
    dynasm!(ctx.ops
        ; mov Rq(new_reg.code()), Rq(lhs_reg.code())
        ; add Rq(new_reg.code()), Rq(rhs_reg.code())
    );
    ctx.free_if_possible(lhs_reg);
    ctx.free_if_possible(rhs_reg);
    Ok(new_reg)
}

fn compile_sub(lhs: &Expr, rhs: &Expr, mut ctx: &mut CompilationContext) -> Result<Rq, String> {
    let lhs_reg = lhs.compile(&mut ctx)?;
    let rhs_reg = rhs.compile(&mut ctx)?;
    let new_reg = ctx.next_register()?;
    dynasm!(ctx.ops
        ; mov Rq(new_reg.code()), Rq(lhs_reg.code())
        ; sub Rq(new_reg.code()), Rq(rhs_reg.code())
    );
    ctx.free_if_possible(lhs_reg);
    ctx.free_if_possible(rhs_reg);
    Ok(new_reg)
}

fn compile_mul(lhs: &Expr, rhs: &Expr, mut ctx: &mut CompilationContext) -> Result<Rq, String> {
    let lhs_reg = lhs.compile(&mut ctx)?;
    let rhs_reg = rhs.compile(&mut ctx)?;
    let new_reg = ctx.next_register()?;
    dynasm!(ctx.ops
        ; mov eax, Rd(lhs_reg.code())
        ; mul Rd(rhs_reg.code())
        ; mov Rd(new_reg.code()), eax
    );
    ctx.free_if_possible(lhs_reg);
    ctx.free_if_possible(rhs_reg);
    Ok(new_reg)
}

fn compile_div(lhs: &Expr, rhs: &Expr, mut ctx: &mut CompilationContext) -> Result<Rq, String> {
    let lhs_reg = lhs.compile(&mut ctx)?;
    let rhs_reg = rhs.compile(&mut ctx)?;
    let new_reg = ctx.next_register()?;
    dynasm!(ctx.ops
        ; mov edx, 0
        ; mov eax, Rd(lhs_reg.code())
        ; div Rd(rhs_reg.code())
        ; mov Rd(new_reg.code()), eax
    );
    ctx.free_if_possible(lhs_reg);
    ctx.free_if_possible(rhs_reg);
    Ok(new_reg)
}

fn compile_eq(lhs: &Expr, rhs: &Expr, mut ctx: &mut CompilationContext) -> Result<Rq, String> {
    let lhs_reg = lhs.compile(&mut ctx)?;
    let rhs_reg = rhs.compile(&mut ctx)?;
    let new_reg = ctx.next_register()?;
    dynasm!(ctx.ops
        ; mov eax, 0
        ; cmp Rd(lhs_reg.code()), Rd(rhs_reg.code())
        ; sete al
        ; mov Rd(new_reg.code()), eax
    );
    ctx.free_if_possible(lhs_reg);
    ctx.free_if_possible(rhs_reg);
    Ok(new_reg)
}

fn compile_neq(lhs: &Expr, rhs: &Expr, mut ctx: &mut CompilationContext) -> Result<Rq, String> {
    let lhs_reg = lhs.compile(&mut ctx)?;
    let rhs_reg = rhs.compile(&mut ctx)?;
    let new_reg = ctx.next_register()?;
    dynasm!(ctx.ops
        ; mov eax, 0
        ; cmp Rd(lhs_reg.code()), Rd(rhs_reg.code())
        ; setne al
        ; mov Rd(new_reg.code()), eax
    );
    ctx.free_if_possible(lhs_reg);
    ctx.free_if_possible(rhs_reg);
    Ok(new_reg)
}

fn compile_function_call(name: &String, param: &Option<Box<Expr>>, mut ctx: &mut CompilationContext) -> Result<Rq, String> {
    let arg = param.as_ref().map(|e| e.compile(&mut ctx));
    let new_reg = ctx.next_register()?;
    let code_repo_ptr = ctx.code_repository as *const CodeRepository;
    dynasm!(ctx.ops
        ; lea rax, [->code]
        ; jmp rax
        ; ->fn_name:
        ; .bytes name.as_bytes()
        ; ->code:
        ; lea rdx, [->fn_name]
        ; push rcx
        ; push r8
        ; push r9
        ; push r10
        ; push r11
        ; mov rcx, QWORD code_repo_ptr as i64
        ; mov r8, QWORD name.len() as _
        ; mov r9, Rq(new_reg.code())
        ; mov rax, QWORD call_function as _ 
        ; sub rsp, BYTE 0x28
        ; call rax
        ; add rsp, BYTE 0x28
        ; pop r11
        ; pop r10
        ; pop r9
        ; pop r8
        ; pop rcx
        ; mov Rd(new_reg.code()), eax
    );
    if let Some(arg_reg) = arg {
        ctx.free_if_possible(arg_reg?);
    }
    Ok(new_reg)
}

pub extern "win64" fn call_function(repository: &CodeRepository, buffer: *const u8, length: u64, arg: i32) -> i32 {
    let fn_name = unsafe { slice::from_raw_parts(buffer, length as usize) };
    let fn_name = std::str::from_utf8(fn_name).unwrap();
    repository.get_fn(fn_name)
        .map(|func| func.call(arg))
        .unwrap_or(0)
}