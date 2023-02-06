use enalang_compiler::ir;
use crate::{machine, native};
use flexstr::{local_fmt, local_str};

pub fn into_string(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let val = ctx.vm.pop()?;

    let st = match val {
        ir::Value::Boolean(true) => local_str!("true"),
        ir::Value::Boolean(false) => local_str!("false"),
        ir::Value::String(st) => st,
        ir::Value::Null => local_str!("null"),
        ir::Value::Block(block_name) => local_fmt!("'{}", block_name),
        ir::Value::Number(num) => local_fmt!("{}", num),
        ir::Value::Pointer(pointer) => local_fmt!("{}->", pointer),
        ir::Value::Exception(err) => local_fmt!("{err:?}"),
        ir::Value::Atom(atom) => local_fmt!(":{atom}"),
    };

    ctx.vm.push(ir::Value::String(st))
}

pub fn into_number(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let val = ctx.vm.pop()?;

    let st: f64 = match val {
        ir::Value::Boolean(true) => 1.0,
        ir::Value::Boolean(false) => 0.0,
        ir::Value::Null => -1.0,
        ir::Value::Number(num) => num,
        ir::Value::Pointer(pointer) => pointer as f64,
        _ => return Err(machine::VMError::CannotConvert(val)),
    };

    ctx.vm.push(ir::Value::Number(st))
}

pub fn is_pointer(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let val = matches!(ctx.vm.pop()?, ir::Value::Pointer(_));
    ctx.vm.push(ir::Value::Boolean(val))
}

pub fn is_number(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let val = matches!(ctx.vm.pop()?, ir::Value::Number(_));
    ctx.vm.push(ir::Value::Boolean(val))
}

pub fn is_block(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let val = matches!(ctx.vm.pop()?, ir::Value::Block(_));
    ctx.vm.push(ir::Value::Boolean(val))
}

pub fn is_bool(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let val = matches!(ctx.vm.pop()?, ir::Value::Boolean(_));
    ctx.vm.push(ir::Value::Boolean(val))
}

pub fn is_string(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let val = matches!(ctx.vm.pop()?, ir::Value::String(_));
    ctx.vm.push(ir::Value::Boolean(val))
}

pub fn is_null(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let val = matches!(ctx.vm.pop()?, ir::Value::Null);
    ctx.vm.push(ir::Value::Boolean(val))
}

pub fn into_ptr(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::Number(num) = ctx.vm.pop()? {
        let ptr = num as usize;

        if num != ptr as f64 {
            return Err(machine::VMError::BadPointer(ptr));
        }

        ctx.vm.push(ir::Value::Pointer(ptr))?;

        Ok(())
    } else {
        Err(machine::VMError::ExpectedNumber)
    }
}

pub fn group() -> native::NativeGroup {
    let mut group = native::NativeGroup::new("");

    group.add_native("unsafe_into_ptr", into_ptr).unwrap();
    group.add_native("into_string", into_string).unwrap();
    group.add_native("into_number", into_number).unwrap();
    group.add_native("is_string", is_string).unwrap();
    group.add_native("is_null", is_null).unwrap();
    group.add_native("is_number", is_number).unwrap();
    group.add_native("is_pointer", is_pointer).unwrap();
    group.add_native("is_block", is_block).unwrap();
    group.add_native("is_bool", is_bool).unwrap();

    group
}
