use crate::vm::{ir, machine};
use rand::{self, Rng};

pub fn vm_debug(ctx: ir::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let el = match ctx.vm.stack.pop() {
        Some(i) => i,
        None => ir::Value::Null,
    };

    println!("{el:?}");

    Ok(())
}

pub fn vm_get_random(ctx: ir::NativeHandlerCtx) -> Result<(), machine::VMError> {
    ctx.vm
        .push(ir::Value::Number(rand::thread_rng().gen_range(0.0..=1.0)))?;
    Ok(())
}

pub fn vm_debug_stack(ctx: ir::NativeHandlerCtx) -> Result<(), machine::VMError> {
    println!("\n=== stack debug ===\n{:?}", ctx.vm.stack);
    Ok(())
}

pub fn vm_debug_calls(ctx: ir::NativeHandlerCtx) -> Result<(), machine::VMError> {
    println!("\n=== call stack debug ===\n{:?}", ctx.vm.call_stack);
    Ok(())
}

pub fn vm_get_annotation(ctx: ir::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::Block(name) = ctx.vm.pop()? {
        match ctx.ir.annotations.get(&name) {
            Some(i) => ctx.vm.push(ir::Value::String(i.clone())),
            None => ctx.vm.push(ir::Value::Null),
        }
    } else {
        Err(machine::VMError::ExpectedBlock)
    }
}

pub fn group() -> ir::NativeGroup {
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
