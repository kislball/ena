use flexstr::{local_fmt, local_str};
use crate::vm::{ir, machine};

pub fn into_string(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let val = vm.pop()?;

    let st = match val {
        ir::Value::Boolean(true) => local_str!("true"),
        ir::Value::Boolean(false) => local_str!("false"),
        ir::Value::String(st) => st,
        ir::Value::Null => local_str!("null"),
        ir::Value::Block(block_name) => local_fmt!("'{}", block_name),
        ir::Value::Number(num) => local_fmt!("{}", num),
        ir::Value::Pointer(pointer) => local_fmt!("{}->", pointer),
    };

    vm.push(ir::Value::String(st))
}

pub fn is_pointer(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let val = matches!(vm.pop()?, ir::Value::Pointer(_));
    vm.push(ir::Value::Boolean(val))
}

pub fn is_number(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let val = matches!(vm.pop()?, ir::Value::Number(_));
    vm.push(ir::Value::Boolean(val))
}

pub fn is_block(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let val = matches!(vm.pop()?, ir::Value::Block(_));
    vm.push(ir::Value::Boolean(val))
}

pub fn is_bool(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let val = matches!(vm.pop()?, ir::Value::Boolean(_));
    vm.push(ir::Value::Boolean(val))
}

pub fn is_string(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let val = matches!(vm.pop()?, ir::Value::String(_));
    vm.push(ir::Value::Boolean(val))
}

pub fn is_null(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let val = matches!(vm.pop()?, ir::Value::Null);
    vm.push(ir::Value::Boolean(val))
}

pub fn into_ptr(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
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

    group.add_native("unsafe_into_ptr", into_ptr).unwrap();
    group.add_native("into_string", into_string).unwrap();
    group.add_native("is_string", is_string).unwrap();
    group.add_native("is_null", is_null).unwrap();
    group.add_native("is_number", is_number).unwrap();
    group.add_native("is_pointer", is_pointer).unwrap();
    group.add_native("is_block", is_block).unwrap();
    group.add_native("is_bool", is_bool).unwrap();

    group
}
