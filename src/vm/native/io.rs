use crate::vm::{ir, machine};

pub fn print(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    if let ir::Value::String(st) = vm.pop()? {
        print!("{}", st);
    } else {
        return Err(machine::VMError::ExpectedString("print".to_string()));
    }

    Ok(())
}

pub fn group<'a>() -> ir::NativeGroup<'a> {
    let mut group = ir::NativeGroup::new("ena.io");

    group.add_native("print", print).unwrap();

    group
}
