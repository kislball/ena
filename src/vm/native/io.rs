use std::collections::HashMap;

use flexstr::LocalStr;

use crate::vm::{ir, machine};

pub fn print(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let ir::Value::String(st) = vm.pop()? {
        print!("{st}");
    } else {
        return Err(machine::VMError::ExpectedString);
    }

    Ok(())
}

pub fn group() -> ir::NativeGroup {
    let mut group = ir::NativeGroup::new("ena.vm.io");

    group.add_native("print", print).unwrap();

    group
}
