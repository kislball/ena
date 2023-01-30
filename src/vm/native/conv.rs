use flexstr::{local_fmt, local_str};

use crate::vm::{ir, machine};

pub fn into_string<'a>(vm: &mut machine::VM, _: &ir::IR<'a>) -> Result<(), machine::VMError> {
    let val = vm.pop()?;

    let st = match val {
        ir::Value::Boolean(true) => local_str!("true"),
        ir::Value::Boolean(false) => local_str!("false"),
        ir::Value::String(st) => st,
        ir::Value::Null => local_str!("null"),
        ir::Value::Block(block_name) => block_name,
        ir::Value::Number(num) => local_fmt!("{}", num),
        ir::Value::Pointer(pointer) => local_fmt!("{}->", pointer),
    };

    vm.push(ir::Value::String(st))
}

pub fn into_ptr<'a>(vm: &mut machine::VM, _: &ir::IR<'a>) -> Result<(), machine::VMError> {
    if let ir::Value::Number(num) = vm.pop()? {
        let ptr = num as usize;

        if num != ptr as f64 {
            return Err(machine::VMError::BadPointer("into_ptr".to_string()));
        }

        vm.push(ir::Value::Pointer(ptr))?;

        Ok(())
    } else {
        Err(machine::VMError::ExpectedNumber("into_ptr".to_string()))
    }
}

pub fn group<'a>() -> ir::NativeGroup<'a> {
    let mut group = ir::NativeGroup::new("");

    group.add_native("into_ptr", into_ptr).unwrap();

    group
}
