use std::slice;
use std::{collections::HashMap, mem, ops::Deref};
use std::time::{SystemTime};

use dynasmrt::{Assembler, AssemblyOffset, ExecutableBuffer, Register, x64::{X64Relocation}, x86::Rd};
use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};
use crate::ast::Expr;
use crate::code_repository::CodeRepository;

pub struct CompilationContext<'a> {
    ops: Assembler<X64Relocation>,
    available_registers: Vec<Rd>,
    available_parameter_registers: Vec<Rd>,
    var: HashMap<String, Rd>,
    code_repository: &'a CodeRepository
}

impl CompilationContext<'_> {
    pub fn new<'a>(code_repository: &'a CodeRepository) -> CompilationContext<'a> {
        CompilationContext {
            ops: dynasmrt::x64::Assembler::new().unwrap(),
            available_registers: vec![Rd::ESP, Rd::EBX, Rd::EBP, Rd::ESI, Rd::EDI, Rd::EDX],
            available_parameter_registers: vec![Rd::ECX],
            var: HashMap::new(),
            code_repository: code_repository
        }
    }

    fn next_register(&mut self) -> Result<Rd, String> {
        self.available_registers.pop().ok_or("No more registers available!".to_string())
    }

    pub fn assign_register_to_variable(&mut self, var: String) -> Result<Rd, String> {
        let result= self.available_parameter_registers.pop()
            .ok_or("No more parameter registers available!".to_string());
        if let Ok(reg) = result {
            self.var.insert(var, reg);
        }
        result
    }

    fn free_if_possible(&mut self, reg: Rd) {
        if reg != Rd::ECX {
            self.available_parameter_registers.push(reg);
        }
    }

    pub fn compile(mut self, expr: &Expr) -> Result<Runable, String> {
        println!("JIT> Compiler called. Starting assembly ...");
        let start = SystemTime::now();
        let offset = self.ops.offset();
        dynasm!(self.ops
            ; .arch x64
        );
        let result_register = expr.compile(&mut self)?;
        dynasm!(self.ops
            ; mov rax, Rq(result_register.code())
            ; ret
        );
        let buf = self.ops.finalize().unwrap();
        
        let since_the_epoch = start.duration_since(start).unwrap().as_nanos();
        
        println!("JIT> Compilation finished in {} ns. Code has size {} @{:p}.", since_the_epoch, buf.len(), buf.ptr(offset));

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
    fn compile(&self, ctx: &mut CompilationContext) -> Result<Rd, String>;
}

impl Compilable for Expr {
    fn compile(&self, mut ctx: &mut CompilationContext) -> Result<Rd, String> {
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

fn compile_number(number: i32, ctx: &mut CompilationContext) -> Result<Rd, String> {
    let register = ctx.next_register()?;
    dynasm!(ctx.ops
        ; mov Rq(register.code()), QWORD number as _
    );
    Ok(register)
}

fn compile_var(name: &String, ctx: &mut CompilationContext) -> Result<Rd, String> {
    let register = ctx.var.get(name).ok_or("Variable was not defined".to_string()).map(|rd| *rd)?;
    Ok(register)
}

fn compile_add(lhs: &Expr, rhs: &Expr, mut ctx: &mut CompilationContext) -> Result<Rd, String> {
    let lhs_reg = lhs.compile(&mut ctx)?;
    let rhs_reg = rhs.compile(&mut ctx)?;
    let new_reg = ctx.next_register()?;
    dynasm!(ctx.ops
        ; mov Rd(new_reg.code()), Rd(lhs_reg.code())
        ; add Rd(new_reg.code()), Rd(rhs_reg.code())
    );
    ctx.free_if_possible(lhs_reg);
    ctx.free_if_possible(rhs_reg);
    Ok(new_reg)
}

fn compile_sub(lhs: &Expr, rhs: &Expr, mut ctx: &mut CompilationContext) -> Result<Rd, String> {
    let lhs_reg = lhs.compile(&mut ctx)?;
    let rhs_reg = rhs.compile(&mut ctx)?;
    let new_reg = ctx.next_register()?;
    dynasm!(ctx.ops
        ; mov Rd(new_reg.code()), Rd(lhs_reg.code())
        ; sub Rd(new_reg.code()), Rd(rhs_reg.code())
    );
    ctx.free_if_possible(lhs_reg);
    ctx.free_if_possible(rhs_reg);
    Ok(new_reg)
}

fn compile_mul(lhs: &Expr, rhs: &Expr, mut ctx: &mut CompilationContext) -> Result<Rd, String> {
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

fn compile_div(lhs: &Expr, rhs: &Expr, mut ctx: &mut CompilationContext) -> Result<Rd, String> {
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

fn compile_eq(lhs: &Expr, rhs: &Expr, mut ctx: &mut CompilationContext) -> Result<Rd, String> {
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

fn compile_neq(lhs: &Expr, rhs: &Expr, mut ctx: &mut CompilationContext) -> Result<Rd, String> {
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

fn compile_function_call(name: &String, param: &Option<Box<Expr>>, mut ctx: &mut CompilationContext) -> Result<Rd, String> {
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
        ; mov rcx, QWORD code_repo_ptr as i64
        ; mov r8, QWORD name.len() as _
        ; mov r9, Rq(new_reg.code())
        ; mov rax, QWORD call_function as _ 
        ; sub rsp, BYTE 0x28
        ; call rax
        ; add rsp, BYTE 0x28
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