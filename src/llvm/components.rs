use std::collections::HashMap;

use llvm_sys::core::*;
use llvm_sys::prelude::*;

use super::types::*;
use super::nodes::*;

macro_rules! c_str {
    ($s:expr) => (
        format!("{}\0", $s).as_ptr() as *const i8
    );
}

pub struct Module {
    builder: LLVMBuilderRef,
    module: LLVMModuleRef,
    context: LLVMContextRef
}

impl Module {
    pub unsafe fn new() -> Module {
        let context: LLVMContextRef  = LLVMContextCreate();

        let module: LLVMModuleRef = LLVMModuleCreateWithName(c_str!("main"));
        let builder: LLVMBuilderRef = LLVMCreateBuilderInContext(context);

        Module {
            context,
            module,
            builder
        }
    }

    pub fn generate(&mut self, program: Node) {
        self.add_func(String::from("logger"), vec![String::from("message")], vec![], false);

        match program {
            Node::Program { body } => self.add_func(
                "main".to_string(),
                Vec::new(),
                body,
                true
            ),
            _ => panic!("Missing Program")
        }
    }

    pub fn dispose(&self) {
        unsafe {
            LLVMDisposeModule(self.module);
            LLVMDisposeBuilder(self.builder);
            LLVMContextDispose(self.context);
        }
    }

    pub fn get_module(&self) -> LLVMModuleRef {
        return self.module;
    }

    fn add_func(&mut self, name: String, params: Vec<String>, body: Vec<Node>, has_body: bool) {
        let mut args = params.iter()
            .map(|_parameter| int8_pointer_type())
            .collect::<Vec<LLVMTypeRef>>();

        let func: LLVMValueRef = unsafe {
            let func_type = LLVMFunctionType(
                void_type(self.context),
                args.as_mut_ptr(),
                args.len() as u32,
                0
            );
            LLVMAddFunction(self.module, c_str!(name.to_string()), func_type)
        };

        for (i, param) in params.iter().enumerate() {
            unsafe {
                let value = LLVMGetParam(func, i as u32);
                LLVMSetValueName2(value, c_str!(param.to_string()), param.len());
            };
        }

        if has_body {
            unsafe { LLVMAppendBasicBlockInContext(self.context, func, c_str!(name.to_string())) };
            let function: Function = Function::from_llvm_value(func, self);
            function.generate(self, body);
        }
    }

    fn get_func(&self, name: String) -> Function {
        let value = unsafe { LLVMGetNamedFunction(self.module, c_str!(name)) };
        Function::from_llvm_value(value, self)
    }
}

struct Function {
    params: HashMap<String, LLVMValueRef>,
    func: LLVMValueRef,
    builder: LLVMBuilderRef,
    block: LLVMBasicBlockRef
}

impl Function {
    fn from_llvm_value(func: LLVMValueRef, module: &Module) -> Function {
        let param_count = unsafe { LLVMCountParams(func) };

        let mut args: HashMap<String, LLVMValueRef> = HashMap::new();
        for i in 0..param_count {
            let param = unsafe { LLVMGetParam(func, i) };
            let mut length: usize = 0;
            let value_name = unsafe { LLVMGetValueName2(param, &mut length) };
            let name: String = string_pointer_to_string(value_name);
            args.insert(name, param);
        }

        let block = unsafe { LLVMGetEntryBasicBlock(func) };

        Function {
            params: args,
            func,
            builder: module.builder,
            block
        }
    }

    fn generate(&self, module: &mut Module, body: Vec<Node>) {
        for node in body {
            self.put_builder_to_end();
            match node {
                Node::NamedFunction { id, parameters, body } => {
                    let name = id_node_to_string(&*id);
                    let params = parameters.iter()
                        .map(|parameter| id_node_to_string(parameter))
                        .collect::<Vec<String>>();

                    module.add_func(name, params, body, true);
                },
                Node::FuncCall { id, arguments } => {
                    self.build_function_call(module, id, arguments);
                },
                _ => unimplemented!()
            }
        }

        self.put_builder_to_end();
        unsafe { LLVMBuildRetVoid(self.builder) };
    }

    fn build_function_call(&self, module: &Module, id: Box<Node>, arguments: Vec<Node> ) {
        let name = id_node_to_string(&*id);
        let func = module.get_func(name);

        let mut args: Vec<_> = vec![];
        for (i, arg) in arguments.iter().enumerate() {

            match arg {
                Node::String(content) => {
                    args.push(
                        unsafe { LLVMBuildPointerCast(
                            self.builder,
                            gen_local_string(
                                module.get_module(),
                                &content
                            ),
                            LLVMTypeOf(LLVMGetParam(func.get_function(), i as u32)),
                            c_str!("")
                        ) }
                    );
                },
                Node::Identifier { name } => {
                    args.push(
                        unsafe { LLVMBuildPointerCast(
                            self.builder,
                            self.get_value(name.to_string()),
                            LLVMTypeOf(LLVMGetParam(func.get_function(), i as u32)),
                            c_str!("")
                        ) }
                    );
                }
                _ => {}
            }
        }
        unsafe { LLVMBuildCall(self.builder, func.get_function(), args.as_mut_ptr(), args.len() as u32, c_str!("")) };
    }

    fn get_value(&self, name: String) -> LLVMValueRef {
        return *self.params.get(&name).unwrap();
    }

    fn get_function(&self) -> LLVMValueRef {
        return self.func;
    }

    fn put_builder_to_end(&self) {
        unsafe {
            LLVMPositionBuilderAtEnd(self.builder, self.block);
        }
    }
}

fn id_node_to_string(node: &Node) -> String {
    match node {
        Node::Identifier { name } => return name.to_string(),
        _ => unreachable!()
    }
}

fn string_pointer_to_string(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_str().unwrap().to_string() }
}
