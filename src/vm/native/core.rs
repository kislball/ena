use crate::vm::{ir, machine};

pub fn drop_value(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    vm.pop()?;
    Ok(())
}

pub fn swap(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let one = vm.pop()?;
    let two = vm.pop()?;

    vm.stack.push(one);
    vm.stack.push(two);

    Ok(())
}

pub fn plus(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.stack.push(ir::Value::Number(a + b));
    } else {
        return Err(machine::VMError::ExpectedNumber("+".to_string()));
    }

    Ok(())
}

pub fn mul(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.stack.push(ir::Value::Number(a * b));
    } else {
        return Err(machine::VMError::ExpectedNumber("*".to_string()));
    }

    Ok(())
}

pub fn div(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.stack.push(ir::Value::Number(a / b));
    } else {
        return Err(machine::VMError::ExpectedNumber("/".to_string()));
    }

    Ok(())
}

pub fn subst(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.stack.push(ir::Value::Number(a - b));
    } else {
        return Err(machine::VMError::ExpectedNumber("-".to_string()));
    }

    Ok(())
}

pub fn pow(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.stack.push(ir::Value::Number(a.powf(b)));
    } else {
        return Err(machine::VMError::ExpectedNumber("pow".to_string()));
    }

    Ok(())
}

pub fn root(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    if let (ir::Value::Number(a), ir::Value::Number(b)) = (vm.pop()?, vm.pop()?) {
        vm.stack.push(ir::Value::Number(a.powf(1.0 / b)));
    } else {
        return Err(machine::VMError::ExpectedNumber("pow".to_string()));
    }

    Ok(())
}

pub fn dup(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let val = vm.pop()?;
    vm.stack.push(val);
    vm.stack.push(val);

    Ok(())
}

pub fn equal(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    match (vm.pop()?, vm.pop()?) {
        (ir::Value::Number(a), ir::Value::Number(b)) => {
            vm.stack.push(ir::Value::Boolean(a == b));
            Ok(())
        },
        _ => {
            Err(machine::VMError::CannotCompare("==".to_string()))
        },
    }
}

pub fn block_exists(vm: &mut machine::VM, ir: &ir::IR) -> Result<(), machine::VMError> {
    if let ir::Value::Block(name) = vm.pop()? {
        vm.stack.push(ir::Value::Boolean(ir.blocks.contains_key(name)));
        Ok(())
    } else {
        Err(machine::VMError::ExpectedBlock("block_exists?".to_string()))
    }
}

pub fn call<'a>(vm: &mut machine::VM<'a>, ir: &ir::IR<'a>) -> Result<(), machine::VMError> {
    if let ir::Value::Block(name) = vm.pop()? {
        vm.run_block(name, &ir)?;
        Ok(())
    } else {
        Err(machine::VMError::ExpectedBlock("call".to_string()))
    }
}

// pub fn run_thread<'a>(vm: &mut machine::VM<'a>, ir: &ir::IR<'a>) -> Result<(), machine::VMError> {
//     if let ir::Value::Block(name) = vm.pop()? {
//         let n_name = name.clone();
//         thread::spawn(|| {
//             let mut new_vm = machine::VM::new();
//             new_vm.run_block(n_name, &ir.clone());
//         });
//         Ok(())
//     } else {
//         Err(machine::VMError::ExpectedBlock("run_thread".to_string()))
//     }
// }

pub fn group<'a>() -> ir::NativeGroup<'a> {
    let mut group = ir::NativeGroup::new("");

    group.add_native("drop", drop_value).unwrap();
    group.add_native("swap", swap).unwrap();
    group.add_native("dup", dup).unwrap();
    group.add_native("+", plus).unwrap();
    group.add_native("*", mul).unwrap();
    group.add_native("/", div).unwrap();
    group.add_native("-", subst).unwrap();
    group.add_native("pow", pow).unwrap();
    group.add_native("root", root).unwrap();
    group.add_native("==", equal).unwrap();
    group.add_native("call", call).unwrap();
    group.add_native("block_exists?", block_exists).unwrap();

    group
}
