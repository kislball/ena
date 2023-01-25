use crate::vm::{heap, ir, machine};

pub fn drop_value(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    if let ir::Value::Pointer(pointer_value) = vm.pop()? {
        heap::heap_result_into_vm(vm.heap.rc_minus(pointer_value))?;
    }

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
    let popped = (vm.pop()?, vm.pop()?);
    if let (ir::Value::Number(a), ir::Value::Number(b)) = popped {
        vm.stack.push(ir::Value::Number(a + b));
    } else if let (ir::Value::Pointer(a), ir::Value::Number(b)) = popped {
        let old_b = b;
        let b = b as usize;

        if old_b != b as f64 {
            return Err(machine::VMError::BadPointer(
                "+(expected integer)".to_string(),
            ));
        }

        {
            let new_ptr = a + b;

            heap::heap_result_into_vm(vm.heap.rc_plus(new_ptr))?;
            heap::heap_result_into_vm(vm.heap.rc_minus(a))?;
        }

        vm.stack.push(ir::Value::Pointer(a + b));
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
    let popped = (vm.pop()?, vm.pop()?);
    if let (ir::Value::Number(a), ir::Value::Number(b)) = popped {
        vm.stack.push(ir::Value::Number(a - b));
    } else if let (ir::Value::Pointer(a), ir::Value::Number(b)) = popped {
        let old_b = b;
        let b = b as usize;

        if old_b != b as f64 {
            return Err(machine::VMError::BadPointer(
                "-(expected integer)".to_string(),
            ));
        }

        if a < b {
            return Err(machine::VMError::BadPointer(
                "-(going negative)".to_string(),
            ));
        }

        {
            let new_ptr = a - b;
            heap::heap_result_into_vm(vm.heap.rc_plus(new_ptr))?;
            heap::heap_result_into_vm(vm.heap.rc_minus(a))?;
        }

        vm.stack.push(ir::Value::Pointer(a - b));
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

    if let ir::Value::Pointer(pointer) = val {
        heap::heap_result_into_vm(vm.heap.rc_plus(pointer))?;
    }

    vm.stack.push(val);
    vm.stack.push(val);

    Ok(())
}

pub fn equal(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    match (vm.pop()?, vm.pop()?) {
        (ir::Value::Number(a), ir::Value::Number(b)) => {
            vm.stack.push(ir::Value::Boolean(a == b));
            Ok(())
        }
        _ => Err(machine::VMError::CannotCompare("==".to_string())),
    }
}

pub fn block_exists(vm: &mut machine::VM, ir: &ir::IR) -> Result<(), machine::VMError> {
    if let ir::Value::Block(name) = vm.pop()? {
        vm.stack
            .push(ir::Value::Boolean(ir.blocks.contains_key(name)));
        Ok(())
    } else {
        Err(machine::VMError::ExpectedBlock("block_exists?".to_string()))
    }
}

pub fn alloc(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let size = vm.pop_usize()?;
    let block: heap::MemoryBlock;

    {
        block = heap::heap_result_into_vm(vm.heap.alloc(size))?;
    }

    vm.stack.push(ir::Value::Pointer(block.pointer));

    Ok(())
}

pub fn realloc(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let new_ptr: usize;

    if let (ir::Value::Pointer(pointer_value), i) = (vm.pop()?, vm.pop_usize()?) {
        new_ptr = heap::heap_result_into_vm(vm.heap.realloc(pointer_value, i))?;
    } else {
        return Err(machine::VMError::ExpectedPointer("realloc".to_string()));
    }

    vm.stack.push(ir::Value::Pointer(new_ptr));

    Ok(())
}

pub fn free(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let pointer = match vm.pop()? {
        ir::Value::Pointer(i) => i,
        _ => {
            return Err(machine::VMError::ExpectedPointer("free".to_string()));
        }
    };

    match vm.heap.free(pointer) {
        Err(_) => Err(machine::VMError::BadPointer("free".to_string())),
        _ => Ok(()),
    }
}

pub fn set_ref(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let ptr: usize;
    let val: ir::Value;

    if let (ir::Value::Pointer(value), b) = (vm.pop()?, vm.pop()?) {
        ptr = value;
        val = b;
    } else {
        return Err(machine::VMError::ExpectedTwo("expected pointer and value"));
    }

    vm.heap.set(ptr, val);

    Ok(())
}

pub fn deref(vm: &mut machine::VM, _: &ir::IR) -> Result<(), machine::VMError> {
    let ptrval = vm.pop()?;
    let val: ir::Value;

    if let ir::Value::Pointer(value) = ptrval {
        val = vm.heap.get(value).unwrap_or(ir::Value::Null);
        heap::heap_result_into_vm(vm.heap.rc_minus(value))?;
    } else {
        return Err(machine::VMError::ExpectedPointer("@".to_string()));
    }

    vm.stack.push(val);

    Ok(())
}

pub fn into_ptr<'a>(vm: &mut machine::VM<'a>, _: &ir::IR<'a>) -> Result<(), machine::VMError> {
    if let ir::Value::Number(num) = vm.pop()? {
        let ptr = num as usize;

        if num != ptr as f64 {
            return Err(machine::VMError::BadPointer("into_ptr".to_string()));
        }

        heap::heap_result_into_vm(vm.heap.rc_plus(ptr))?;

        vm.stack.push(ir::Value::Pointer(ptr));

        Ok(())
    } else {
        Err(machine::VMError::ExpectedNumber("into_ptr".to_string()))
    }
}

pub fn call<'a>(vm: &mut machine::VM<'a>, ir: &ir::IR<'a>) -> Result<(), machine::VMError> {
    if let ir::Value::Block(name) = vm.pop()? {
        vm.run_block(name, ir)?;
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
    group.add_native("@", deref).unwrap();
    group.add_native("=", set_ref).unwrap();
    group.add_native("into_ptr", into_ptr).unwrap();
    group.add_native("unsafe_alloc", alloc).unwrap();
    group.add_native("unsafe_realloc", realloc).unwrap();
    group.add_native("unsafe_free", free).unwrap();

    group
}
