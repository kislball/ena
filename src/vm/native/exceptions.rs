use std::collections::HashMap;

use flexstr::LocalStr;

use crate::vm::{ir, machine};

pub fn try_exception(
    vm: &mut machine::VM,
    ir: &ir::IR,
    locals: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    let block = if let ir::Value::Block(block_name) = vm.pop()? {
        block_name
    } else {
        return Err(machine::VMError::ExpectedBlock);
    };

    if let Err(err) = vm.run_block(block, ir, locals) {
        vm.push(ir::Value::VMError(Box::from(err)))?;
    }

    Ok(())
}

pub fn into_exception(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    let exception = ir::Value::VMError(Box::from(machine::VMError::RuntimeException(vm.pop()?)));
    vm.push(exception)
}

pub fn unwrap_exception(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let ir::Value::VMError(err) = vm.pop()? {
        if let machine::VMError::RuntimeException(real_err) = *err {
            vm.push(real_err)
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
