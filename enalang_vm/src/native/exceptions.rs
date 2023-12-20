use crate::{machine, native};
use enalang_ir as ir;
use flexstr::local_fmt;
use ir::Value;

pub fn try_exception(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let block = if let ir::Value::Block(block_name) = ctx.vm.pop()? {
        block_name
    } else {
        return Err(machine::VMError::ExpectedBlock);
    };

    if let Err(err) = ctx.vm.run_block(&block) {
        ctx.vm
            .push(ir::Value::Exception(Box::new(ir::Value::String(
                local_fmt!("{err:?}"),
            ))))?;
    }

    Ok(())
}

pub fn into_exception(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let exception = ir::Value::Exception(Box::from(ctx.vm.pop()?));
    ctx.vm.push(exception)
}

pub fn unwrap_exception(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::Exception(err) = ctx.vm.pop()? {
        ctx.vm.push(*err)
    } else {
        Err(machine::VMError::ExpectedException)
    }
}

pub fn throw_exception(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::Exception(err) = ctx.vm.pop()? {
        Err(machine::VMError::RuntimeException(*err))
    } else {
        Err(machine::VMError::ExpectedException)
    }
}

pub fn is_exception(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::Exception(_) = ctx.vm.pop()? {
        ctx.vm.push(Value::Boolean(true))
    } else {
        ctx.vm.push(Value::Boolean(false))
    }
}

pub fn group() -> native::NativeGroup {
    let mut group = native::NativeGroup::new("");

    group.add_native("into_exception", into_exception).unwrap();
    group
        .add_native("unwrap_exception", unwrap_exception)
        .unwrap();
    group.add_native("try", try_exception).unwrap();
    group.add_native("throw", throw_exception).unwrap();
    group.add_native("is_exception", is_exception).unwrap();

    group
}
