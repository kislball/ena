use crate::vm::{ir, machine};

pub fn try_exception(ctx: ir::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let block = if let ir::Value::Block(block_name) = ctx.vm.pop()? {
        block_name
    } else {
        return Err(machine::VMError::ExpectedBlock);
    };

    if let Err(err) = ctx
        .vm
        .run_block(block, ctx.ir, ctx.single_evals)
    {
        ctx.vm.push(ir::Value::VMError(Box::from(err)))?;
    }

    Ok(())
}

pub fn into_exception(ctx: ir::NativeHandlerCtx) -> Result<(), machine::VMError> {
    let exception =
        ir::Value::VMError(Box::from(machine::VMError::RuntimeException(ctx.vm.pop()?)));
    ctx.vm.push(exception)
}

pub fn unwrap_exception(ctx: ir::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::VMError(err) = ctx.vm.pop()? {
        if let machine::VMError::RuntimeException(real_err) = *err {
            ctx.vm.push(real_err)
        } else {
            Err(machine::VMError::ExpectedException)
        }
    } else {
        Err(machine::VMError::ExpectedException)
    }
}

pub fn group() -> ir::NativeGroup {
    let mut group = ir::NativeGroup::new("");

    group.add_native("into_exception", into_exception).unwrap();
    group
        .add_native("unwrap_exception", unwrap_exception)
        .unwrap();
    group.add_native("try", try_exception).unwrap();

    group
}
