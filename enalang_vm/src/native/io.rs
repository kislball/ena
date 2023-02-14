use crate::{machine, native};
use enalang_ir as ir;

pub fn print(ctx: native::NativeHandlerCtx) -> Result<(), machine::VMError> {
    if let ir::Value::String(st) = ctx.vm.pop()? {
        print!("{st}");
    } else {
        return Err(machine::VMError::ExpectedString);
    }

    Ok(())
}

pub fn group() -> native::NativeGroup {
    let mut group = native::NativeGroup::new("ena.vm.io");

    group.add_native("print", print).unwrap();

    group
}
