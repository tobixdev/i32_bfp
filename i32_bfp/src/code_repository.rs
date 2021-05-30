use std::{collections::HashMap, slice};
use dynasm::dynasm;
use dynasmrt::{DynasmApi, DynasmLabelApi};

use crate::{ast::{FunctionDef}, compiler::{CompilationContext, Runable, call_function}};



#[derive(Debug)]
pub struct CodeRepository {
    code: HashMap<String, Runable>,
    ast: HashMap<String, FunctionDef>,
    // The graveyard should alleviate segfaults which were happening. If our stub code is executed
    // it generates the real code for the called function and replaces it in the code map. 
    // This would mean that the existing code would be dropped, but out IP is still within that
    // code block once the call to the newly compiled code terminates. A simple (but hacky) solution
    // is to keep the code in memory. This is done by the graveyard.
    graveyard: Vec<Runable>
}

impl CodeRepository {
    pub fn new() -> CodeRepository {
        CodeRepository {
            code: HashMap::new(),
            ast: HashMap::new(),
            graveyard: Vec::new()
        }
    }
    
    pub fn add_placeholder(&mut self, function_def: FunctionDef) -> Result<(), String> {
        let mut ops = dynasmrt::x64::Assembler::new().unwrap();

        dynasm!(ops
            ; .arch x64
            ; ->fn_name:
            ; .bytes function_def.name.as_bytes()
        );

        let offset = ops.offset();

        let code_repository_ptr = self as *const CodeRepository;

        dynasm!(ops
            ; mov r9, rcx
            ; mov rcx, QWORD code_repository_ptr as i64
            ; lea rdx, [->fn_name]
            ; mov r8, QWORD function_def.name.len() as _
            ; mov rax, QWORD call_compiler as _
            ; sub rsp, BYTE 0x28
            ; call rax
            ; add rsp, BYTE 0x28
            ; ret
        );

        let runable = Runable::new(ops.finalize().unwrap(), offset);
        self.code.insert(function_def.name.clone(), runable);
        self.ast.insert(function_def.name.clone(), function_def);

        Ok(())
    }

    pub fn get_fn(&self, name: &str) -> Option<&Runable> {
        self.code.get(name)
    }

    pub fn pop_ast(&mut self, name: &str) -> Option<FunctionDef> {
        self.ast.remove(name)
    }

    pub fn print_code(&self, name: &str) {
        match self.code.get(name) {
            Some(runable) => {
                runable.print();
            }
            None => {
                println!("No code entry found for fn {}.", name);
            }
        }
    }
}

pub extern "win64" fn call_compiler(code_repository: &mut CodeRepository, buffer: *const u8, length: u64, arg: i32) -> i32 {
    let fn_name = unsafe { slice::from_raw_parts(buffer, length as usize) };
    let fn_name = std::str::from_utf8(fn_name).unwrap();
    println!("JIT> Uncompiled function {} called. Compiling ...", fn_name);

    let function_def = code_repository.pop_ast(fn_name).expect("Could not find function definition in repository.");
    let mut ctx = CompilationContext::new(code_repository);
    let mut result = Result::Ok(());
    if let Some(var) = function_def.parameter.clone() {
        result = result.and_then(|_| ctx.assign_register_to_variable(var).map(|_| ()));
    }
    let compiled = result.and_then(|_| ctx.compile(&function_def.body));
    
    let stub = code_repository.code.remove(fn_name).expect("Could not remove current code from code repository");
    code_repository.graveyard.push(stub);

    match compiled {
        Ok(runable) => {
            code_repository.code.insert(fn_name.to_string(), runable);
            println!("JIT> Calling newly compiled function");
            call_function(code_repository, buffer, length, arg)
        },
        Err(message) => {
            println!("JIT> Compiling failed with error {}.", message);
            println!("JIT> Definition was removed.");
            0
        }
    }
}