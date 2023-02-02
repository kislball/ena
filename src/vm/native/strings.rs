use std::collections::HashMap;
use flexstr::{local_fmt, LocalStr, ToLocalStr};
use crate::vm::{ir, machine};

pub fn strlen(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let ir::Value::String(st) = vm.pop()? {
        vm.push(ir::Value::Number(st.len() as f64))
    } else {
        return Err(machine::VMError::ExpectedString);
    }
}

pub fn concat(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let (ir::Value::String(a), ir::Value::String(b)) = (vm.pop()?, vm.pop()?) {
        vm.push(ir::Value::String(local_fmt!("{a}{b}")))
    } else {
        return Err(machine::VMError::ExpectedString);
    }
}

pub fn split(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let (ir::Value::String(a), ir::Value::String(b)) = (vm.pop()?, vm.pop()?) {
        let vals: Vec<&str> = a.split(b.as_str()).collect();

        for val in &vals {
            vm.push(ir::Value::String(val.to_local_str()))?;
        }

        vm.push(ir::Value::Number(vals.len() as f64))?;

        Ok(())
    } else {
        return Err(machine::VMError::ExpectedString);
    }
}

pub fn contains(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let (ir::Value::String(a), ir::Value::String(b)) = (vm.pop()?, vm.pop()?) {
        vm.push(ir::Value::Boolean(a.to_string().contains(b.as_str())))
    } else {
        Err(machine::VMError::ExpectedString)
    }
}

pub fn chars(
    vm: &mut machine::VM,
    _: &ir::IR,
    _: &HashMap<LocalStr, ir::Value>,
) -> Result<(), machine::VMError> {
    if let ir::Value::String(a) = vm.pop()? {
        let chars: Vec<char> = a.chars().collect();

        for ch in &chars {
            vm.push(ir::Value::String(ch.to_local_str()))?;
        }

        vm.push(ir::Value::Number(chars.len() as f64))?;

        Ok(())
    } else {
        return Err(machine::VMError::ExpectedString);
    }
}

pub fn group() -> ir::NativeGroup {
    let mut group = ir::NativeGroup::new("string");

    group.add_native("len", strlen).unwrap();
    group.add_native("concat", concat).unwrap();
    group.add_native("split", split).unwrap();
    group.add_native("contains", contains).unwrap();
    group.add_native("chars", chars).unwrap();

    group
}
