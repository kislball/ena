use crate::vm::{ir, machine};
use rand::{self, Rng};

pub fn vm_debug(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let el = match vm.stack.pop() {
        Some(i) => i,
        None => ir::Value::Null,
    };

    println!("{el:?}");

    Ok(())
}

pub fn vm_get_random(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    vm.push(ir::Value::Number(rand::thread_rng().gen_range(0.0..=1.0)))?;
    Ok(())
}

pub fn vm_debug_stack(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    println!("\n=== stack debug ===\n{:?}", vm.stack);
    Ok(())
}

pub fn vm_debug_calls(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    println!("\n=== call stack debug ===\n{:?}", vm.call_stack);
    Ok(())
}

pub fn vm_get_annotation(vm: &mut machine::VM, ir: &ir::IR) -> Result<(), machine::VMError> {
    if let ir::Value::Block(name) = vm.pop()? {
        match ir.annotations.get(&name) {
            Some(i) => vm.push(ir::Value::String(i.clone())),
            None => vm.push(ir::Value::Null),
        }
    } else {
        Err(machine::VMError::ExpectedBlock)
    }
}

pub fn group<'a>() -> ir::NativeGroup<'a> {
    let mut group = ir::NativeGroup::new("ena.vm");

    group.add_native("debug", vm_debug).unwrap();
    group.add_native("debug_stack", vm_debug_stack).unwrap();
    group.add_native("debug_calls", vm_debug_calls).unwrap();
    group.add_native("random", vm_get_random).unwrap();
    group
        .add_native("get_annotation", vm_get_annotation)
        .unwrap();

    group
}
