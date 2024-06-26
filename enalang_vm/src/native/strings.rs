use crate::{define_native_group, machine, native};
use enalang_ir as ir;
use flexstr::{local_fmt, ToLocalStr};

pub fn strlen(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::String(st) = ctx.vm.pop()? {
        ctx.vm.push(ir::Value::Number(st.len() as f64))
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn concat(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let (ir::Value::String(a), ir::Value::String(b)) = (ctx.vm.pop()?, ctx.vm.pop()?) {
        ctx.vm.push(ir::Value::String(local_fmt!("{a}{b}")))
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn split(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
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
            ctx.vm.push(ir::Value::String(ch.to_local_str()))?;
        }

        ctx.vm.push(ir::Value::Number(chars.len() as f64))?;

        Ok(())
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

define_native_group! {
    group,
    "string",
    "len" => strlen,
    "concat" => concat,
    "split" => split,
    "contains" => contains,
    "chars" => chars
}
