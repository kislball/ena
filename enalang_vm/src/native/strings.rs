use crate::{machine, native};
use enalang_ir as ir;
use flexstr::{shared_fmt, ToSharedStr};

pub fn strlen(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::String(st) = ctx.vm.pop()? {
        ctx.vm.push(ir::Value::Number(st.len() as f64))
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn concat(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::String(a), ir::Value::String(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        ctx.vm.push(ir::Value::String(shared_fmt!("{a}{b}")))
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn split(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::String(a), ir::Value::String(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        let vals: Vec<&str> = a.split(b.as_str()).collect();

        for val in &vals {
            ctx.vm.push(ir::Value::String(val.to_shared_str()))?;
        }

        ctx.vm.push(ir::Value::Number(vals.len() as f64))?;

        Ok(())
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn contains(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::String(a), ir::Value::String(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        ctx.vm
            .push(ir::Value::Boolean(a.to_string().contains(b.as_str())))
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn chars(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::String(a) = ctx.vm.pop()? {
        let chars: Vec<char> = a.chars().collect();

        for ch in &chars {
            ctx.vm.push(ir::Value::String(ch.to_shared_str()))?;
        }

        ctx.vm.push(ir::Value::Number(chars.len() as f64))?;

        Ok(())
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn group() -> native::NativeGroup {
    let mut group = native::NativeGroup::new("string");

    group.add_native("len", strlen).unwrap();
    group.add_native("concat", concat).unwrap();
    group.add_native("split", split).unwrap();
    group.add_native("contains", contains).unwrap();
    group.add_native("chars", chars).unwrap();

    group
}
