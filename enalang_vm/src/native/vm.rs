use crate::{machine, native};
use enalang_ir as ir;
use flexstr::ToLocalStr;
use rand::{self, Rng};
use std::{fs::OpenOptions, io::Read};

pub fn vm_load(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::String(path) = ctx.vm.pop()? {
        let mut options = OpenOptions::new()
            .read(true)
            .open(path.as_str())
            .map_err(|x| machine::VMError::FS(x.to_string().to_local_str()))?;
        let mut v = Vec::<u8>::new();
        options
            .read_to_end(&mut v)
            .map_err(|x| machine::VMError::FS(x.to_string().to_local_str()))?;
        let ir = ir::from_vec(&v).map_err(|x| {
            machine::VMError::RuntimeException(ir::Value::String(x.to_string().to_local_str()))
        })?;
        let ir = ir.into_ir().map_err(|x| {
            machine::VMError::RuntimeException(ir::Value::String(x.to_string().to_local_str()))
        })?;
        ctx.vm.load(ir)
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn vm_debug(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let el = match ctx.vm.stack.pop() {
        Some(i) => i,
        None => ir::Value::Null,
    };

    println!("{el:?}");

    Ok(())
}

pub fn vm_get_random(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    ctx.vm
        .push(ir::Value::Number(rand::thread_rng().gen_range(0.0..=1.0)))?;
    Ok(())
}

pub fn vm_debug_stack(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    println!("\n=== stack debug ===\n{:?}", ctx.vm.stack);
    Ok(())
}

pub fn vm_debug_calls(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    println!("\n=== call stack debug ===\n{:?}", ctx.vm.call_stack);
    Ok(())
}

pub fn vm_get_annotation(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::Block(name) = ctx.vm.pop()? {
        match ctx.vm.scope_manager.blocks().annotations.get(&name) {
            Some(i) => ctx.vm.push(ir::Value::String(i.clone())),
            None => ctx.vm.push(ir::Value::Null),
        }
    } else {
        Err(machine::VMError::ExpectedBlock)
    }
}

pub fn group() -> native::NativeGroup {
    let mut group = native::NativeGroup::new("ena.vm");

    group.add_native("load", vm_load).unwrap();
    group.add_native("debug", vm_debug).unwrap();
    group.add_native("debug_stack", vm_debug_stack).unwrap();
    group.add_native("debug_calls", vm_debug_calls).unwrap();
    group.add_native("random", vm_get_random).unwrap();
    group
        .add_native("get_annotation", vm_get_annotation)
        .unwrap();

    group
}
