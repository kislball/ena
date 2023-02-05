use crate::vm::{machine};
use crate::ir;

pub fn print(ctx: ir::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::String(st) = ctx.vm.pop()? {
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
