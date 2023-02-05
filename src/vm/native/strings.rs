use crate::vm::{machine};
use crate::ir;
use flexstr::{local_fmt, ToLocalStr};

pub fn strlen(ctx: ir::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::String(st) = ctx.vm.pop()? {
        ctx.vm.push(ir::Value::Number(st.len() as f64))
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn concat(ctx: ir::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::String(a), ir::Value::String(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        ctx.vm.push(ir::Value::String(local_fmt!("{a}{b}")))
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn split(ctx: ir::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::String(a), ir::Value::String(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        let vals: Vec<&str> = a.split(b.as_str()).collect();

        for val in &vals {
            ctx.vm.push(ir::Value::String(val.to_local_str()))?;
        }

        ctx.vm.push(ir::Value::Number(vals.len() as f64))?;

        Ok(())
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn contains(ctx: ir::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::String(a), ir::Value::String(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        ctx.vm
            .push(ir::Value::Boolean(a.to_string().contains(b.as_str())))
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn chars(ctx: ir::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::String(a) = ctx.vm.pop()? {
        let chars: Vec<char> = a.chars().collect();

        for ch in &chars {
            ctx.vm.push(ir::Value::String(ch.to_local_str()))?;
        }

        ctx.vm.push(ir::Value::Number(chars.len() as f64))?;

        Ok(())
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn group() -> ir::NativeGroup {
    let mut group = ir::NativeGroup::new("string");

    group.add_native("len", strlen).unwrap();
    group.add_native("concat", concat).unwrap();
    group.add_native("split", split).unwrap();
    group.add_native("contains", contains).unwrap();
    group.add_native("chars", chars).unwrap();

    group
}
