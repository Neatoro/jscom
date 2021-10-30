extern crate llvm_sys;

use super::nodes;
use super::nodes::*;
use llvm_sys::target::*;

mod components;
mod target;
mod types;

macro_rules! c_str {
    ($s:expr) => (
        format!("{}\0", $s).as_ptr() as *const i8
    );
}

pub fn build(node: Node) {
    unsafe {
        LLVM_InitializeAllTargetInfos();
        LLVM_InitializeAllTargets();
        LLVM_InitializeAllTargetMCs();
        LLVM_InitializeAllAsmParsers();
        LLVM_InitializeAllAsmPrinters();

        let mut module = components::Module::new();
        module.generate(node);

        get_target();

        let target_triple = get_target();
        println!("{}", target_triple);
        let mut target_machine = target::TargetMachine::new(c_str!(target_triple)).unwrap();
        target_machine.write_to_object_file(&module);

        module.dispose();
        target_machine.dispose();
    }
}

fn get_target() -> String {
    let output = std::process::Command::new(super::build_clang_path())
        .args(["--version"])
        .output()
        .expect("Executing Clang failed");

    let output_string = String::from_utf8(output.stdout).unwrap();
    let target_lines = output_string
        .lines()
        .filter(|line| line.starts_with("Target"))
        .collect::<Vec<&str>>();

    let target_line = *target_lines.get(0).unwrap();
    let mut split = target_line.split(" ");
    split.next();
    return split.next().unwrap().to_string();
}
